use crate::{channel::ChannelInfo, protocol::ScmiXfer};

mod smc;

pub use smc::Smc;

pub trait Transport {
    fn chan_available(&self, idx: usize) -> bool;
    fn no_completion_irq(&self) -> bool;
    // fn chan_setup(&mut self, info: ChannelInfo);
    // fn chan_free(&mut self, idx: usize);
    // fn send_message(&self, info: &ChannelInfo, xfer: ScmiXfer);
}
