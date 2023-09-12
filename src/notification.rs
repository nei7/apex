use tokio;

use dbus::{
    channel::MatchingReceiver,
    message::MatchRule,
    nonblock,
    strings::{Interface, Member},
    Message,
};
use dbus_tokio::connection;
use std::time::Duration;

use tokio_stream::Stream;

use async_stream::stream;

use anyhow::Result;
use tokio::sync::mpsc;

use embedded_graphics::{
    framebuffer::Framebuffer,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X12},
        MonoTextStyle,
    },
    pixelcolor::{
        raw::{LittleEndian, RawU1},
        BinaryColor,
    },
    prelude::DrawTarget,
    prelude::Point,
    text::Text,
    Drawable,
};

#[derive(Debug)]
pub struct Notification {
    pub title: String,
    pub content: String,
}

impl TryFrom<Message> for Notification {
    type Error = anyhow::Error;

    fn try_from(value: Message) -> Result<Self> {
        let (_, _, _, title, content) = value.read5::<String, u32, String, String, String>()?;

        Ok(Notification { title, content })
    }
}

impl Drawable for Notification {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        Text::new(
            &self.title,
            Point::new(10, 10),
            MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
        )
        .draw(target)?;

        Text::new(
            &self.content,
            Point::new(10, 20),
            MonoTextStyle::new(&FONT_6X12, BinaryColor::On),
        )
        .draw(target)?;

        Ok(())
    }
}

pub async fn read_notifications<const SIZE: usize>(
) -> Result<impl Stream<Item = Result<[u8; SIZE]>>> {
    let mut frame_buffer: Framebuffer<BinaryColor, RawU1, LittleEndian, 128, 40, SIZE> =
        Framebuffer::new();

    let (resource, conn) = connection::new_session_sync()?;
    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let proxy = nonblock::Proxy::new(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(500),
        conn.clone(),
    );

    let mut rule = MatchRule::new();
    rule.interface = Some(Interface::from("org.freedesktop.Notifications"));
    rule.member = Some(Member::from("Notify"));

    proxy
        .method_call(
            "org.freedesktop.DBus.Monitoring",
            "BecomeMonitor",
            (vec![rule.match_str()], 0u32),
        )
        .await?;

    let (tx, mut rx) = mpsc::channel::<Message>(10);

    conn.start_receive(rule, Box::new(move |msg, _| tx.try_send(msg).is_ok()));

    Ok(stream! {
         while let Some(msg) = rx.recv().await {
            frame_buffer.clear(BinaryColor::Off)?;
            let notif = Notification::try_from(msg)?;
            notif.draw(&mut frame_buffer)?;
            yield Ok(frame_buffer.data().clone());
        }
    })
}
