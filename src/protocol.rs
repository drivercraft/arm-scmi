use core::ffi::c_void;
use core::sync::atomic::AtomicI32;

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
    pub type_: u8,
    pub seq: u16,
    pub status: u32,
    pub poll_completion: bool,
}

#[repr(C)]
pub struct Completion {
    // TODO: define fields
}

#[repr(C)]
pub struct HlistNode {
    // TODO: define fields
}

/// Message(Tx/Rx) structure
///
/// - buf: Buffer pointer
/// - len: Length of data in the Buffer
#[repr(C)]
pub struct ScmiMsg {}
type Refcount = i32;
type Spinlock = (); // placeholder, TODO: implement spinlock

#[repr(C)]
pub struct ScmiXfer {
    pub transfer_id: i32,
    pub hdr: ScmiMsgHdr,
    pub tx: ScmiMsg,
    pub rx: ScmiMsg,
}
