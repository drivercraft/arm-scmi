use smccc::Call;

use crate::Transport;

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
    
    fn send_message(&mut self, info: &crate::ChannelInfo, xfer: crate::ScmiXfer) {
        // smccc::Smc::call64(self.func_id, 0, 0, 0, 0, 0, 0, )
    }
}
