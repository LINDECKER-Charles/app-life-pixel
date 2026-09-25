//! `healthcheck`: `GET /healthz` on the local listener, for the images, which have no shell and
//! no `curl`.

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// How long the server has to answer.
const HEALTHCHECK_TIMEOUT: Duration = Duration::from_secs(5);
/// The start of a healthy answer.
const HEALTHY_STATUS_LINE: &[u8] = b"HTTP/1.1 200 ";

/// Whether the server listening on `address` answers `/healthz` with `200` within five
/// seconds. An unspecified address, such as a container's `0.0.0.0`, is asked on loopback.
pub async fn healthcheck(address: SocketAddr) -> bool {
    let target = local_target(address);
    let answer = tokio::time::timeout(HEALTHCHECK_TIMEOUT, request_health(target)).await;
    matches!(answer, Ok(Ok(true)))
}

fn local_target(address: SocketAddr) -> SocketAddr {
    let ip = match address.ip() {
        IpAddr::V4(ip) if ip.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(ip) if ip.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
        ip => ip,
    };
    SocketAddr::new(ip, address.port())
}

async fn request_health(target: SocketAddr) -> io::Result<bool> {
    let mut stream = TcpStream::connect(target).await?;
    let request = format!("GET /healthz HTTP/1.1\r\nHost: {target}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await?;
    let mut status_line = [0; HEALTHY_STATUS_LINE.len()];
    stream.read_exact(&mut status_line).await?;
    Ok(status_line == HEALTHY_STATUS_LINE)
}
