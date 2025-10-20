use log::debug;
use smccc::{error::success_or_error_32, smc32};
use tock_registers::interfaces::Readable;

use crate::{Shmem, Transport, Xfer, err::ScmiError};

pub struct Smc {
    func_id: u32,
    irq: Option<u32>,
}

impl Smc {
    pub fn new(func_id: u32, irq: Option<u32>) -> Self {
        Smc { func_id, irq }
    }

    fn call(&self) -> Result<(), smccc::psci::Error> {
        success_or_error_32(smc32(self.func_id, [0; 7])[0])
    }
}

impl Transport for Smc {
    fn chan_available(&self, _idx: usize) -> bool {
        true
    }

    fn no_completion_irq(&self) -> bool {
        self.irq.is_none()
    }

    fn send_message(&mut self, shmem: &mut Shmem, xfer: &Xfer) -> Result<(), ScmiError> {
        shmem.tx_prepare(xfer);
        debug!("Sending SMC message {:?}", xfer.hdr);
        self.call().unwrap();

        Ok(())
    }

    const MAX_MSG: usize = 20;

    const MAX_MSG_SIZE: usize = 128;

    const SYNC_CMDS_COMPLETED_ON_RET: bool = true;

    fn fetch_response(&mut self, shmem: &mut Shmem, xfer: &mut Xfer) -> Result<(), ScmiError> {
        shmem.rx_prepare(xfer);
        let len = shmem.header().length.get() as usize;
        xfer.hdr.status = unsafe { (shmem.payload_ptr() as *const u32).read_volatile() };
        debug!("Fetched SMC response len = {len}, header: {:?}", xfer.hdr);
        xfer.hdr.to_result()?;
        let rx_len = len.saturating_sub(8).min(xfer.rx.len());
        if rx_len > 0 {
            shmem.read_payload(&mut xfer.rx[..rx_len], 4);
        }
        xfer.rx.resize(rx_len, 0);
        debug!(
            "Fetched response: hdr={:?}, rx_len={}, buff={:?}",
            xfer.hdr,
            xfer.rx.len(),
            xfer.rx
        );

        Ok(())
    }
}
