use core::ptr::NonNull;

use aarch64_cpu_ext::{
    asm::cache,
    cache::{CacheOp, dcache_range},
};
use log::debug;
use mbarrier::{rmb, wmb};
use tock_registers::{interfaces::*, registers::*};

use crate::Xfer;

tock_registers::register_structs! {
    pub ShmemHeader {
        (0x00 => reserved: u32),
        (0x04 => channel_status: ReadWrite<u32,  ChannelStatus::Register>),
        (0x08 => reserved1: [u32; 2]),
        (0x10 => flags: ReadWrite<u32, ShmemFlags::Register>),
        (0x14 => pub length: ReadWrite<u32>),
        (0x18 => pub msg_header: ReadWrite<u32>),
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
    pub fn init(&mut self) {
        debug!("Initializing SHMEM at {:p}", self.address);
        self.header().channel_status.set(0);
        self.header().flags.set(0);
        self.header().length.set(0);
        self.header().msg_header.set(0);
    }

    pub(crate) fn header(&mut self) -> &mut ShmemHeader {
        unsafe { &mut *(self.address.as_ptr() as *mut ShmemHeader) }
    }
    pub fn tx_prepare(&mut self, xfer: &Xfer) {
        // loop {
        //     match self
        //         .header()
        //         .channel_status
        //         .read_as_enum(ChannelStatus::STATUS)
        //     {
        //         Some(ChannelStatus::STATUS::Value::FREE) => {
        //             break;
        //         }
        //         _ => {
        //             // panic!("Channel not free: {:?}", e);
        //         }
        //     }
        // }
        /* Mark channel busy + clear error */

        self.header().channel_status.set(0);
        self.header().flags.set(0);
        // if xfer.hdr.poll_completion {
        //     self.header().flags.modify(ShmemFlags::INTR_ENABLED::CLEAR);
        // } else {
        //     self.header().flags.modify(ShmemFlags::INTR_ENABLED::SET);
        // }
        let len = size_of::<u32>() as u32 + xfer.tx.len() as u32;
        self.header().length.set(len);
        self.header().msg_header.set(xfer.hdr.pack());

        debug!(
            "Preparing TX: hdr={:?}, tx_len={}, all_len={len}",
            xfer.hdr,
            xfer.tx.len()
        );
        /* Copy TX payload */
        if !xfer.tx.is_empty() {
            self.write_payload(&xfer.tx);
        }
        let size = size_of::<ShmemHeader>() + xfer.tx.len();
        dcache_range(CacheOp::Clean, self.address.as_ptr() as usize, size);
    }

    pub fn rx_prepare_hdr(&mut self) {
        let size = size_of::<ShmemHeader>();
        dcache_range(CacheOp::Invalidate, self.address.as_ptr() as usize, size);
    }

    pub fn rx_prepare_payload(&mut self, xfer: &mut Xfer) {
        dcache_range(
            CacheOp::Invalidate,
            self.address.as_ptr() as usize + size_of::<ShmemHeader>(),
            xfer.rx.len(),
        );
    }

    pub fn payload_ptr(&mut self) -> *mut u8 {
        unsafe { self.address.as_ptr().add(size_of::<ShmemHeader>()) }
    }

    pub fn write_payload(&mut self, buff: &[u8]) {
        unsafe {
            let dest = self.address.as_ptr().add(size_of::<ShmemHeader>());
            for (i, &b) in buff.iter().enumerate() {
                dest.add(i).write_volatile(b);
            }
        }
        wmb();
    }

    pub fn read_payload(&mut self, buff: &mut [u8], skip: usize) {
        unsafe {
            let src = self.payload_ptr();
            for (i, b) in buff.iter_mut().enumerate() {
                *b = src.add(skip + i).read_volatile();
            }
        }
        rmb();
    }
}

impl Shmem {
    const COMPATIBLE: &str = "arm,scmi-shmem";
}
