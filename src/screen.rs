use anyhow::{anyhow, Result};

use hidapi::{HidApi, HidDevice};

static APEX7_TKL: u16 = 0x1618;
static STEELSERIES_VENDOR_ID: u16 = 0x1038;

use bitvec::prelude::*;

pub struct Screen {
    device: HidDevice,
    frame_buffer: BitArray<[u8; 642], Msb0>,
}

impl Screen {
    pub fn open() -> Result<Self> {
        let api = match HidApi::new() {
            Ok(api) => api,
            Err(e) => panic!("err: {}", e),
        };

        let device = api
            .device_list()
            .find(|device| {
                device.vendor_id() == STEELSERIES_VENDOR_ID
                    && device.product_id() == APEX7_TKL
                    && device.interface_number() == 1
            })
            .ok_or(anyhow!("no device found"))?;

        let device = device.open_device(&api)?;

        let mut fb = BitArray::ZERO;
        fb.as_raw_mut_slice()[0] = 0x61;

        Ok(Screen {
            device: device,
            frame_buffer: fb,
        })
    }

    pub fn send_data(&self) -> Result<()> {
        self.device.send_feature_report(&self.frame_buffer.data)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.frame_buffer.fill(false);
        self.frame_buffer.as_raw_mut_slice()[0] = 0x61;
        Ok(())
    }

    pub fn draw(&mut self, data: &[u8]) -> Result<()> {
        for i in 2..640 {
            self.frame_buffer.as_raw_mut_slice()[i] = data[i];
        }

        self.send_data()
    }
}
