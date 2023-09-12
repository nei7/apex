use tokio::{
    self, pin,
    time::{sleep, Duration},
};
use tokio_stream::StreamExt;

mod notification;
mod screen;
mod sysinfo;

#[tokio::main]
async fn main() {
    let mut screen = screen::Screen::open().unwrap();
    let notification_stream = notification::read_notifications::<640>().await.unwrap();
    let system_info_stream = sysinfo::read_sys_info::<640>().await.unwrap();

    pin!(notification_stream, system_info_stream);

    loop {
        tokio::select! {
            Some(Ok(data)) = notification_stream.next() =>{
                screen.draw(&data).unwrap();
                sleep(Duration::from_millis(5000)).await;
            },
            Some(Ok(data)) = system_info_stream.next() => screen.draw(&data).unwrap(),
        }
    }
}
