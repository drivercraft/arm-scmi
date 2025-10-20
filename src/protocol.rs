use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

use alloc::vec::Vec;
use spin::Mutex;

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
const SCMI_XFER_BUSY: i32 = 1;
const SCMI_XFER_SENT_OK: i32 = 0;
const SCMI_XFER_RESP_OK: i32 = 1;
const SCMI_XFER_DRESP_OK: i32 = 2;

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

const MSG_ID_MASK: u32 = genmask(7, 0);
const MSG_TYPE_MASK: u32 = genmask(9, 8);
const MSG_PROTOCOL_ID_MASK: u32 = genmask(17, 10);
const MSG_TOKEN_ID_MASK: u32 = genmask(27, 18);

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Command = 0,
    DelayedResponse = 2,
    Notification = 3,
}

#[repr(C)]
pub struct Completion {
    // TODO: define fields
}

#[repr(C)]
pub struct HlistNode {
    // TODO: define fields
}

type Refcount = i32;
type Spinlock = (); // placeholder, TODO: implement spinlock

#[repr(C)]
pub struct Xfer {
    pub transfer_id: i32,
    pub hdr: ScmiMsgHdr,
    pub tx: Vec<u8>,
    pub rx: Vec<u8>,
}

impl Xfer {
    pub fn new(msg_id: u8, tx_size: usize, rx_size: usize, set_pending: bool) -> Self {
        static TRANSFER_ID_COUNTER: AtomicI32 = AtomicI32::new(0);
        static TOKEN_ALLOC: Mutex<TokenTable> = Mutex::new(TokenTable::new());

        let transfer_id = TRANSFER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        if set_pending {
            
        }

        Xfer {
            transfer_id,
            hdr,
            tx,
            rx,
        }
    }

    fn token_alloc(&mut self) {}
}

struct TokenTable {}

impl TokenTable {
    const fn new() -> Self {
        TokenTable {}
    }
}
