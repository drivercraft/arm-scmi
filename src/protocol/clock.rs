use nb::block;

use crate::{Transport, err::ScmiError};

pub struct Clock<T: Transport> {
    protocol: super::Protocal<T>,
    attributes: Option<ClockAttributes>,
}

impl<T: Transport> Clock<T> {
    pub const PROTOCOL_ID: u8 = 0x14;

    pub(crate) fn new(protocol: super::Protocal<T>) -> Self {
        Self {
            protocol,
            attributes: None,
        }
    }

    pub(crate) fn init(&mut self) {
        {
            // Initialization code if needed
            let mut version_fur = self.protocol.version();
            let version = block!(version_fur.poll_completion()).unwrap();
            debug!("Clock Protocol version: {:#x}", version);
        }
        self.attributes().unwrap();
    }

    fn attributes(&mut self) -> Result<(), ScmiError> {
        let xfer = super::Xfer::new(super::PROTOCOL_ATTRIBUTES, 0, 4);
        let mut res = self.protocol.do_xfer(xfer, |xfer| {
            let mut buff = [0u8; 4];
            buff[..4].copy_from_slice(&xfer.rx[..4]);
            Ok(buff)
        });
        let res = block!(res.poll_completion())?;
        let num_clocks = u16::from_le_bytes([res[0], res[1]]);
        let max_async_req = res[2];
        self.attributes = Some(ClockAttributes {
            num_clocks,
            max_async_req,
            reserved: res[3],
        });
        debug!(
            "Clock Protocol Attributes: num_clocks={}, max_async_req={}",
            num_clocks, max_async_req
        );
        Ok(())
    }
}

#[derive(Debug)]
#[repr(C)]
struct ClockAttributes {
    pub num_clocks: u16,
    pub max_async_req: u8,
    pub reserved: u8,
}
