#![no_std]

pub use crate::{channel::ChannelInfo, protocol::ScmiXfer, shmem::Shmem};

mod channel;
mod err;
mod protocol;
mod shmem;
mod transport;

pub use transport::Smc;
pub use transport::Transport;

pub struct Scmi<T: Transport> {
    transport: T,
    shmem: Shmem,
}

impl<T: Transport> Scmi<T> {
    pub fn new(kind: T, shmem: Shmem) -> Self {
        Self {
            transport: kind,
            shmem,
        }
    }
}
