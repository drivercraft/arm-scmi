use crate::Transport;

pub struct Clock<T: Transport> {
    protocol: super::Protocal<T>,
}

impl<T: Transport> Clock<T> {
    pub const PROTOCOL_ID: u8 = 0x14;

    pub(crate) fn new(protocol: super::Protocal<T>) -> Self {
        Self { protocol }
    }

    pub(crate) fn init(&mut self) {
        // Initialization code if needed

    }
 
}
