use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use alloc::{vec, vec::Vec};
use log::debug;
use mbarrier::smp_mb;
use spin::Mutex;

use crate::{Data, Transport, err::ScmiError};

const PROTOCOL_VERSION: u8 = 0;
const PROTOCOL_ATTRIBUTES: u8 = 0x1;

pub struct Protocal<T: Transport> {
    data: Data<T>,
    id: u8,
}

impl<T: Transport> Protocal<T> {
    pub(super) fn new(data: Data<T>, id: u8) -> Self {
        Self { data, id }
    }

    pub fn do_xfer<'a, R, F>(
        &'a mut self,
        mut xfer: Xfer,
        on_completed: F,
    ) -> XferFuture<'a, T, R, F>
    where
        F: Fn(&mut Xfer) -> Result<R, ScmiError>,
    {
        xfer.hdr.protocol_id = self.id;

        xfer.hdr.clear_status();
        xfer.status = XferStatus::Init;

        smp_mb();
        XferFuture {
            protocol: self,
            xfer,
            on_complete: on_completed,
        }
    }

    pub fn version(
        &mut self,
    ) -> XferFuture<'_, T, u32, impl Fn(&mut Xfer) -> Result<u32, ScmiError>> {
        let xfer = Xfer::new(PROTOCOL_VERSION, 0, 4);
        self.do_xfer(xfer, |xfer| {
            let version = u32::from_le_bytes([xfer.rx[0], xfer.rx[1], xfer.rx[2], xfer.rx[3]]);
            Ok(version)
        })
    }
}

pub struct XferFuture<'a, T: Transport, R, F: Fn(&mut Xfer) -> Result<R, ScmiError>> {
    protocol: &'a mut Protocal<T>,
    xfer: Xfer,
    on_complete: F,
}

impl<'a, T: Transport, R, F: Fn(&mut Xfer) -> Result<R, ScmiError>> XferFuture<'a, T, R, F> {
    pub fn poll_completion(&mut self) -> nb::Result<R, ScmiError> {
        debug!("Polling completion: xfer status={:?}", self.xfer.status);
        match self.xfer.status {
            XferStatus::Init => {
                self.protocol.data.lock().send_message(&mut self.xfer)?;
                self.xfer.status = XferStatus::SendOk;
                Err(nb::Error::WouldBlock)
            }
            XferStatus::SendOk => {
                self.protocol.data.lock().fetch_response(&mut self.xfer)?;
                self.xfer.status = XferStatus::RespOk;
                Err(nb::Error::WouldBlock)
            }
            XferStatus::RespOk => {
                let res = (self.on_complete)(&mut self.xfer)?;
                Ok(res)
            }
        }
    }
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmiStdProtocol {
    Base = 0x10,
    Power = 0x11,
    System = 0x12,
    Perf = 0x13,
    Clock = 0x14,
    Sensor = 0x15,
    Reset = 0x16,
    Voltage = 0x17,
    Powercap = 0x18,
}

#[allow(dead_code)]
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmiSystemEvents {
    Shutdown,
    Coldreset,
    Warmreset,
    Powerup,
    Suspend,
    Max,
}

// use core::sync::Mutex; // not available in no_std

const SCMI_XFER_FREE: i32 = 0;
#[allow(dead_code)]
const SCMI_XFER_BUSY: i32 = 1;
#[allow(dead_code)]
const SCMI_XFER_SENT_OK: i32 = 0;
#[allow(dead_code)]
const SCMI_XFER_RESP_OK: i32 = 1;
#[allow(dead_code)]
const SCMI_XFER_DRESP_OK: i32 = 2;

static TRANSFER_ID_COUNTER: AtomicI32 = AtomicI32::new(0);
static TOKEN_ALLOCATOR: Mutex<TokenTable> = Mutex::new(TokenTable::new());

const fn genmask(high: u32, low: u32) -> u32 {
    if high >= 32 || low >= 32 || high < low {
        0
    } else {
        let all = u32::MAX;
        let upper = all >> (31 - high);
        let lower = all << low;
        upper & lower
    }
}

const fn mask_to_max(mask: u32) -> u32 {
    if mask == 0 {
        0
    } else {
        mask >> mask.trailing_zeros()
    }
}

const MSG_ID_MASK: u32 = genmask(7, 0);
const MSG_TYPE_MASK: u32 = genmask(9, 8);
const MSG_PROTOCOL_ID_MASK: u32 = genmask(17, 10);
const MSG_TOKEN_ID_MASK: u32 = genmask(27, 18);
const MSG_TOKEN_MAX: usize = mask_to_max(MSG_TOKEN_ID_MASK) as usize + 1;

#[inline(always)]
fn field_prep(mask: u32, value: u32) -> u32 {
    let shift = mask.trailing_zeros();
    ((value & (mask >> shift)) << shift) & mask
}

/// Message(Tx/Rx) header
///
/// - id: The identifier of the message being sent
/// - protocol_id: The identifier of the protocol used to send id message
/// - type_: The SCMI type for this message
/// - seq: The token to identify the message. When a message returns, the
///   platform returns the whole message header unmodified including the
///   token
/// - status: Status of the transfer once it's complete
/// - poll_completion: Indicate if the transfer needs to be polled for
///   completion or interrupt mode is used
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ScmiMsgHdr {
    pub id: u8,
    pub protocol_id: u8,
    pub type_: MsgType,
    pub seq: u16,
    pub status: u32,
    pub poll_completion: bool,
}

impl ScmiMsgHdr {
    pub fn pack(&self) -> u32 {
        field_prep(MSG_ID_MASK, self.id.into())
            | field_prep(MSG_TYPE_MASK, self.type_ as u32)
            | field_prep(MSG_TOKEN_ID_MASK, self.seq.into())
            | field_prep(MSG_PROTOCOL_ID_MASK, self.protocol_id.into())
    }

    pub fn to_result(&self) -> Result<(), ScmiError> {
        ScmiError::from_status(self.status as i32)
    }

    pub fn clear_status(&mut self) {
        self.status = 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum MsgType {
    #[default]
    Command = 0,
    DelayedResponse = 2,
    Notification = 3,
}

#[allow(dead_code)]
#[repr(C)]
pub struct Completion {
    // TODO: define fields
}

#[allow(dead_code)]
#[repr(C)]
pub struct HlistNode {
    // TODO: define fields
}

#[allow(dead_code)]
type Refcount = i32;
#[allow(dead_code)]
type Spinlock = (); // placeholder, TODO: implement spinlock

pub struct Xfer {
    pub transfer_id: i32,
    pub hdr: ScmiMsgHdr,
    pub tx: Vec<u8>,
    pub rx: Vec<u8>,
    pub pending: bool,
    busy: AtomicBool,
    pub status: XferStatus,
}

impl Xfer {
    pub fn new(msg_id: u8, tx_size: usize, rx_size: usize) -> Self {
        let transfer_id = TRANSFER_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let token = TOKEN_ALLOCATOR.lock().alloc().expect("Alloc token fail");

        let hdr = ScmiMsgHdr {
            id: msg_id,
            seq: token,
            ..Default::default()
        };

        let tx = vec![0u8; tx_size];
        let rx = vec![0u8; rx_size];

        Self {
            transfer_id,
            hdr,
            tx,
            rx,
            pending: false,
            busy: AtomicBool::new(false),
            status: XferStatus::SendOk,
        }
    }

    pub fn token(&self) -> u16 {
        self.hdr.seq
    }
}

impl Drop for Xfer {
    fn drop(&mut self) {
        TOKEN_ALLOCATOR.lock().release(self.hdr.seq);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum XferStatus {
    #[default]
    Init = 0,
    SendOk = 1,
    RespOk = 2,
}

const TOKENS_PER_WORD: usize = 32;
#[allow(clippy::manual_div_ceil)]
const TOKEN_TABLE_WORDS: usize = (MSG_TOKEN_MAX + TOKENS_PER_WORD - 1) / TOKENS_PER_WORD;

const fn token_table_init() -> [u32; TOKEN_TABLE_WORDS] {
    [0; TOKEN_TABLE_WORDS]
}

struct TokenTable {
    bitmap: [u32; TOKEN_TABLE_WORDS],
    next_hint: usize,
}

impl TokenTable {
    const fn new() -> Self {
        TokenTable {
            bitmap: token_table_init(),
            next_hint: 0,
        }
    }

    fn alloc(&mut self) -> Option<u16> {
        for offset in 0..MSG_TOKEN_MAX {
            let token = (self.next_hint + offset) % MSG_TOKEN_MAX;
            let word_idx = token / TOKENS_PER_WORD;
            let bit_idx = token % TOKENS_PER_WORD;
            let mask = 1u32 << bit_idx;
            if self.bitmap[word_idx] & mask == 0 {
                self.bitmap[word_idx] |= mask;
                self.next_hint = (token + 1) % MSG_TOKEN_MAX;
                return Some(token as u16);
            }
        }

        None
    }

    fn release(&mut self, token: u16) {
        let token = token as usize;
        if token >= MSG_TOKEN_MAX {
            return;
        }

        let word_idx = token / TOKENS_PER_WORD;
        let bit_idx = token % TOKENS_PER_WORD;
        let mask = 1u32 << bit_idx;
        self.bitmap[word_idx] &= !mask;
        self.next_hint = token;
    }
}
