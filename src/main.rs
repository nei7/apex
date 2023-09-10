use std::time::Duration;

use tokio::{self};

use tokio::pin;
use tokio_stream::StreamExt;

use embedded_graphics::{
    mono_font::{
        ascii::{FONT_10X20, FONT_6X12},
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    prelude::Point,
    text::Text,
};

use embedded_graphics::Drawable;
use tokio::time::{self};

mod notification;
mod screen;

#[tokio::main]
async fn main() {
    let mut screen = screen::Screen::open().unwrap();

    let mut notifications = notification::read_notifications().await.unwrap();
    pin!(notifications);

    loop {
        tokio::select! {
            val = notifications.next() => {
               if let Some(Ok(notification)) = val {
                    screen.clear();
                    notification.draw(&mut screen);
                    screen.flush();

               }
            }

        };
    }
}
