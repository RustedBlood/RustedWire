use std::net::SocketAddr;
use std::time::Duration;
use tokio::io;
use tokio::net::UdpSocket;
pub async fn search_recivers() -> Result<SocketAddr, io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:9999").await?;

    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        if buf.is_empty() {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }
        println!(
            "Found node {} at {}",
            String::from_utf8_lossy(&buf[..len]),
            addr.ip()
        );
        return Ok(addr);
    }
}
