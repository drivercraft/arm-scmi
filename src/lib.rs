#![no_std]

extern crate alloc;

pub use crate::{channel::ChannelInfo, protocol::Xfer, shmem::Shmem};

mod channel;
mod err;
mod protocol;
mod shmem;
mod transport;

use alloc::sync::Arc;
use spin::Mutex;
pub use transport::Smc;
pub use transport::Transport;

type Data<T> = Arc<Mutex<ScmiData<T>>>;

pub struct Scmi<T: Transport> {
    data: Data<T>,
}

impl<T: Transport> Scmi<T> {
    pub fn new(kind: T, mut shmem: Shmem) -> Self {
        shmem.init();
        let data = ScmiData {
            transport: kind,
            shmem,
        };
        Scmi {
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn protocol(&self, id: u8) -> protocol::Protocal<T> {
        let data = self.data.clone();
        protocol::Protocal::new(data, id)
    }
}

struct ScmiData<T: Transport> {
    transport: T,
    shmem: Shmem,
}

impl<T: Transport> ScmiData<T> {
    pub fn send_message(&mut self, xfer: &mut Xfer) -> Result<(), crate::err::ScmiError> {
        self.transport.send_message(&mut self.shmem, xfer)
    }

    pub fn fetch_response(&mut self, xfer: &mut Xfer) -> Result<(), crate::err::ScmiError> {
        self.transport.fetch_response(&mut self.shmem, xfer)
    }
}
