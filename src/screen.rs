use anyhow::{anyhow, Result};

use embedded_graphics::{
    framebuffer::{buffer_size, Framebuffer},
    pixelcolor::BinaryColor,
    pixelcolor::{raw::LittleEndian, raw::RawU1},
    prelude::*,
};
use hidapi::{HidApi, HidDevice, HidError};
use sysinfo::{System, SystemExt};
use tokio::time::{sleep, Duration};

static APEX7_TKL: u16 = 0x1618;
static STEELSERIES_VENDOR_ID: u16 = 0x1038;

use bitvec::prelude::*;

pub struct Screen {
    device: HidDevice,
    frame_buffer: BitArray<[u8; 642], Msb0>,
}

impl DrawTarget for Screen {
    type Color = BinaryColor;
    type Error = anyhow::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<()>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            if let (x @ 0..=127, y @ 0..=39) = (coord.x, coord.y) {
                // Calculate the index in the framebuffer.
                let index: i32 = x + y * 128;
                self.frame_buffer.set(index as usize, color.is_on());
            }
        }

        Ok(())
    }
}

impl OriginDimensions for Screen {
    fn size(&self) -> Size {
        Size::new(128, 40)
    }
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

    pub fn flush(&self) -> Result<()> {
        self.device.send_feature_report(&self.frame_buffer.data)?;
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        self.frame_buffer.fill(false);
        self.frame_buffer.as_raw_mut_slice()[0] = 0x61;
        self.flush()?;
        Ok(())
    }
}

// fn render_sys_info(&mut self) {
//     self.sys.refresh_memory();
//     let mem_used = self.sys.used_memory() as f64 / u64::pow(1024, 3) as f64;
//     Text::new(
//         format!("M: {:>4.1}G", mem_used).as_str(),
//         Point::new(27, 20),
//         MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
//     )
//     .draw(&mut self.frame_buffer)
//     .unwrap();

//     self.device
//         .send_feature_report(self.frame_buffer.data())
//         .unwrap();
// }
