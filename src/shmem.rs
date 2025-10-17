use core::ptr::NonNull;

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub struct ChannelStatus: u32 {
        const FREE = 0;
        const ERROR = 1 << 1;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
     pub struct ShmemFlags: u32 {
        const INTR_ENABLED = 1 << 0;
    }
}

#[repr(C)]
pub struct SharedMem {
    reserved: u32,
    channel_status: ChannelStatus,
    reserved1: [u32; 2],
    flags: ShmemFlags,
    length: u32,
    msg_header: u32,
    // msg_payload: [u32],
}

pub struct Shmem {
    pub address: NonNull<u8>,
    pub bus_address: usize,
    pub size: usize,
}

impl Shmem {
    const COMPATIBLE: &str = "arm,scmi-shmem";
}
