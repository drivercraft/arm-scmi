use crate::Transport;
use core::ffi::c_void;

pub struct Channel<T: Transport> {
    inner: T,
}

pub struct ChannelInfo {}

