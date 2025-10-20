use core::fmt::Display;

use smccc::Call;

use crate::{Shmem, Transport, err::ScmiError};

pub struct Smc {
    func_id: u32,
    irq: Option<u32>,
}

impl Smc {
    pub fn new(func_id: u32, irq: Option<u32>) -> Self {
        Smc { func_id, irq }
    }
}

impl Transport for Smc {
    fn chan_available(&self, _idx: usize) -> bool {
        true
    }

    fn no_completion_irq(&self) -> bool {
        self.irq.is_none()
    }

    fn send_message(
        &mut self,
        info: &crate::ChannelInfo,
        shmem: &mut Shmem,
        xfer: crate::Xfer,
    ) -> Result<(), ScmiError> {
        shmem.tx_prepare(&xfer);

        let ret = smccc::Smc::call32(self.func_id, [0, 0, 0, 0, 0, 0, 0]);
        if ret[0] != 0 {
            return Err(ScmiError::NotSupported);
        }
        Ok(())
    }

    const MAX_MSG: usize = 20;

    const MAX_MSG_SIZE: usize = 128;

    const SYNC_CMDS_COMPLETED_ON_RET: bool = true;
}
