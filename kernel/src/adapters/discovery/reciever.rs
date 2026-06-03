#[cfg(target_os = "linux")]
use std::os::unix::ffi::OsStrExt;

use gethostname::gethostname;
use tokio::time;
use tokio::{io, net::UdpSocket};

pub async fn broadcast_send_msg() -> Result<(), io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    let message = gethostname();
    println!("started to send with name: {:?}", message);

    loop {
        #[cfg(target_os = "linux")]
        {
            socket
                .send_to(message.as_bytes(), "255.255.255.255:9999")
                .await?;
        }

        #[cfg(target_os = "windows")]
        {
            socket
                .send_to(message.as_encoded_bytes(), "255.255.255.255:9999")
                .await?;
        }

        time::sleep(time::Duration::from_secs(5)).await
    }
}
