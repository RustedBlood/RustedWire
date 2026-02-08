use gethostname::gethostname;
use std::os::unix::ffi::OsStrExt;
use tokio::time;
use tokio::{io, net::UdpSocket};
pub async fn start_to_recieve() -> Result<(), io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.set_broadcast(true)?;
    let message = gethostname();
    println!("started to recieve with name: {:?}", message);
    loop {
        socket
            .send_to(message.as_bytes(), "255.255.255.255:9999")
            .await?;
        time::sleep(time::Duration::from_secs(5)).await
    }
}
