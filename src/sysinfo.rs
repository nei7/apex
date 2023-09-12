use async_stream::stream;
use embedded_graphics::framebuffer::Framebuffer;
use embedded_graphics::pixelcolor::BinaryColor;

use embedded_graphics::pixelcolor::raw::{LittleEndian, RawU1};
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::DrawTarget,
    prelude::Point,
    text::Text,
    Drawable,
};

use sysinfo::{CpuExt, System, SystemExt};
use tokio_stream::Stream;

use anyhow::Result;
use tokio::time::{self, Duration};

pub struct SysInfo {
    mem_used: f64,
    cpu_used: f64,
    sys: System,
}

impl Drawable for SysInfo {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::new(
            format!("M: {:>4.1}G", self.mem_used).as_str(),
            Point::new(10, 12),
            MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
        )
        .draw(target)?;

        Text::new(
            format!("C: {:>4.1}%", self.cpu_used).as_str(),
            Point::new(10, 35),
            MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
        )
        .draw(target)?;

        Ok(())
    }
}

impl SysInfo {
    pub fn new() -> Self {
        Self {
            mem_used: 0_f64,
            cpu_used: 0_f64,
            sys: System::new_all(),
        }
    }

    fn poll(&mut self) {
        self.sys.refresh_memory();
        self.sys.refresh_cpu();

        let mem_used = self.sys.used_memory() as f64 / u64::pow(1024, 3) as f64;

        let cpu = self.sys.global_cpu_info();
        let cpu_used = cpu.cpu_usage() as f64;

        self.cpu_used = cpu_used;
        self.mem_used = mem_used
    }
}

pub async fn read_sys_info<const SIZE: usize>() -> Result<impl Stream<Item = Result<[u8; SIZE]>>> {
    let mut frame_buffer: Framebuffer<BinaryColor, RawU1, LittleEndian, 128, 40, SIZE> =
        Framebuffer::new();

    let mut interval = time::interval(Duration::from_millis(1000));
    let mut sys = SysInfo::new();

    Ok(stream! {
        loop {
            interval.tick().await;
            sys.poll();
            frame_buffer.clear(BinaryColor::Off)?;
            sys.draw(&mut frame_buffer)?;
            yield Ok(frame_buffer.data().clone());
        }

    })
}
