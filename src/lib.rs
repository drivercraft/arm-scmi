#![no_std]

use core::{
    fmt,
    marker::PhantomData,
    mem,
    ptr::{NonNull, copy_nonoverlapping},
    sync::atomic::{AtomicU16, Ordering, compiler_fence},
};

pub use crate::{channel::ChannelInfo, protocol::ScmiXfer, shmem::Shmem};

mod channel;
mod err;
mod protocol;
mod shmem;
mod transport;

pub use transport::Transport;

pub struct Scmi<T: Transport> {
    transport: T,
    shmem: Shmem,
}
