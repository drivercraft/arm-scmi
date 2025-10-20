use core::ptr::NonNull;

use mbarrier::{rmb, wmb};
use tock_registers::{interfaces::*, registers::*};

use crate::{Xfer, protocol::ScmiMsgHdr};

tock_registers::register_structs! {
    pub ShmemHeader {
        (0x00 => reserved: u32),
        (0x04 => channel_status: ReadWrite<u32,  ChannelStatus::Register>),
        (0x08 => reserved1: [u32; 2]),
        (0x10 => flags: ReadWrite<u32, ShmemFlags::Register>),
        (0x14 => length: ReadWrite<u32>),
        (0x18 => msg_header: ReadWrite<u32>),
        (0x1C => @END),
    }
}

tock_registers::register_bitfields![
    u32,
    ChannelStatus [
        STATUS OFFSET(0) NUMBITS(2) [
            FREE = 0,
            ERROR = 0b10,
        ]
    ],
    ShmemFlags [
        INTR_ENABLED OFFSET(0) NUMBITS(1) [],
    ],
];

pub struct Shmem {
    pub address: NonNull<u8>,
    pub bus_address: usize,
    pub size: usize,
}

impl Shmem {
    fn header(&mut self) -> &mut ShmemHeader {
        unsafe { &mut *(self.address.as_ptr() as *mut ShmemHeader) }
    }
    pub fn tx_prepare(&mut self, xfer: &Xfer) {
        match self
            .header()
            .channel_status
            .read_as_enum(ChannelStatus::STATUS)
        {
            Some(ChannelStatus::STATUS::Value::FREE) => {}
            _ => {
                panic!("Timeout waiting for channel response");
            }
        }

        /* Mark channel busy + clear error */

        self.header().channel_status.set(0);

        if xfer.hdr.poll_completion {
            self.header().flags.modify(ShmemFlags::INTR_ENABLED::CLEAR);
        } else {
            self.header().flags.modify(ShmemFlags::INTR_ENABLED::SET);
        }

        self.header()
            .length
            .set(size_of::<u32>() as u32 + xfer.tx.len() as u32);

        self.header().msg_header.set(xfer.hdr.pack());

        /* Copy TX payload */
        if !xfer.tx.is_empty() {
            unsafe {
                let dest = self.address.as_ptr().add(size_of::<ShmemHeader>());
                core::ptr::copy_nonoverlapping(xfer.tx.as_ptr(), dest, xfer.tx.len());
            }
        }
        wmb();
    }
}

impl Shmem {
    const COMPATIBLE: &str = "arm,scmi-shmem";
}
