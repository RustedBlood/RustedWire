use std::time::Duration;
use tokio::io;
use tokio::net::UdpSocket;

#[derive(PartialEq, Clone)]
pub struct HostInfo {
    pub ip: String,
    pub name: String,
}

pub async fn broadcast_get_recievers() -> Result<Vec<HostInfo>, io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:9999").await?;
    let mut result: Vec<HostInfo> = Vec::new();
    let mut buf = [0u8; 1024];

    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => break,
            recv = socket.recv_from(&mut buf) => {
                let (size, addr) = recv?;
                if let Ok(hostname) = String::from_utf8(buf[..size].to_vec()) {
                    let host = HostInfo {
                        ip: addr.ip().to_string(),
                        name: hostname
                    };
                    if !buf.is_empty() && !result.contains(&host) {
                        result.push(host);
                        println!("Found node at {}", addr.ip());
                    }
                }
            }
        }
    }
    Ok(result)
}
