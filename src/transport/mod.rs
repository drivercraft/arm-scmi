use crate::{Shmem, channel::ChannelInfo, protocol::Xfer};

mod smc;

pub use smc::Smc;

pub trait Transport {
    const MAX_MSG: usize;
    const MAX_MSG_SIZE: usize;
    const SYNC_CMDS_COMPLETED_ON_RET: bool;

    fn chan_available(&self, idx: usize) -> bool;
    fn no_completion_irq(&self) -> bool;
    // fn chan_setup(&mut self, info: ChannelInfo);
    // fn chan_free(&mut self, idx: usize);
    fn send_message(
        &mut self,
        info: &ChannelInfo,
        shmem: &mut Shmem,
        xfer: Xfer,
    ) -> Result<(), crate::err::ScmiError>;
}
