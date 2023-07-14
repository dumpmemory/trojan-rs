use std::{net::SocketAddr, time::Duration};

use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    spawn,
};

use tokio_rustls::TlsServerStream;

use crate::{config::OPTIONS, types::Result};

pub async fn start_tcp(
    mut source: TlsServerStream,
    target_addr: SocketAddr,
    mut buffer: BytesMut,
    src_addr: SocketAddr,
) -> Result<()> {
    if target_addr == *OPTIONS.back_addr.as_ref().unwrap() {
        let mut proxy_added = false;
        for _ in 0..10 {
            let mut headers = [httparse::EMPTY_HEADER; 100];
            let mut request = httparse::Request::new(&mut headers);
            match request.parse(buffer.as_ref()) {
                Ok(httparse::Status::Complete(offset)) => {
                    log::error!("X-Forwarded-For: {}", src_addr);
                    let data = buffer.split_off(offset - 2);
                    buffer.extend_from_slice(b"X-Forwarded-For: ");
                    buffer.extend_from_slice(src_addr.ip().to_string().as_bytes());
                    buffer.extend_from_slice(b"\r\n");
                    buffer.unsplit(data);
                    proxy_added = true;
                    break;
                }
                _ => {
                    if tokio::time::timeout(Duration::from_secs(10), source.read_buf(&mut buffer))
                        .await??
                        == 0
                    {
                        log::error!("read http header failed");
                        return Ok(());
                    }
                }
            }
        }
        if !proxy_added {
            log::error!(
                "header not completed after 10 retries:{}",
                String::from_utf8_lossy(buffer.as_ref())
            );
        }
    }

    log::info!("tcp backend:{}", target_addr);
    let mut target = TcpStream::connect(target_addr).await?;
    if let Err(err) = target.write_all(buffer.as_ref()).await {
        let _ = target.shutdown().await;
        let _ = source.shutdown().await;
        log::error!("tcp send data to target:{} failed:{}", target_addr, err);
        return Ok(());
    }
    let (source_read, source_write) = source.into_split();
    let (target_read, target_write) = target.into_split();
    spawn(copy(
        source_read,
        target_write,
        format!("tcp {} to {}", src_addr, target_addr),
        OPTIONS.tcp_idle_timeout,
    ));
    spawn(copy(
        target_read,
        source_write,
        format!("tcp {} to {}", target_addr, src_addr),
        OPTIONS.tcp_idle_timeout,
    ));
    Ok(())
}

async fn copy<R: AsyncReadExt + Unpin, W: AsyncWriteExt + Unpin>(
    mut read: R,
    mut write: W,
    message: String,
    timeout: u64,
) {
    let mut buffer = vec![0u8; 4096];
    loop {
        if let Ok(Ok(n)) = tokio::time::timeout(
            Duration::from_secs(timeout),
            read.read(buffer.as_mut_slice()),
        )
        .await
        {
            if n > 0 {
                if let Err(err) = write.write_all(&buffer.as_slice()[..n]).await {
                    log::warn!("{} write failed:{}", message, err);
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    log::warn!("{} read failed", message);
    let _ = write.shutdown().await;
}
