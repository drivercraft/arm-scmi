use core::ptr::NonNull;

use crate::{Transport, shmem::SharedMem};

pub struct Smc {
    shmem: NonNull<SharedMem>,
    func_id: u32,
    irq: Option<u32>,
}

impl Smc {
    pub fn new(shmem: NonNull<SharedMem>, func_id: u32, irq: Option<u32>) -> Self {
        Smc {
            shmem,
            func_id,
            irq,
        }
    }
}

impl Transport for Smc {
    fn chan_available(&self, _idx: usize) -> bool {
        true
    }

    fn no_completion_irq(&self) -> bool {
        self.irq.is_none()
    }
}
