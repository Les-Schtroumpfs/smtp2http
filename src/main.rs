use smtp_proto::request::receiver::RequestReceiver;
use std::env;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream) {
    let peer = stream
        .peer_addr()
        .map(|a| a.to_string())
        .unwrap_or_else(|_| "unknown".into());
    eprintln!("[conn] accepted from {peer}");

    let mut receiver = RequestReceiver::default();
    let mut read_buf = [0u8; 4096];
    // Scratch buffer for the parser
    let buf = [0u8; 8192];

    loop {
        match stream.read(&mut read_buf).await {
            Ok(0) => {
                eprintln!("[conn] {peer} closed");
                break;
            }
            Ok(n) => {
                let mut it = read_buf[..n].iter();
                match receiver.ingest(&mut it, &buf) {
                    Ok(req) => {
                        eprintln!("[conn {peer}] parsed request: {req:?}");
                    }
                    Err(err) => {
                        eprintln!("[conn {peer}] parse error: {err:?}");
                    }
                }
            }
            Err(err) => {
                eprintln!("[conn {peer}] read error: {err:?}");
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = env::var("SMTP_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:2525".to_string());
    let listener = TcpListener::bind(&addr).await?;
    eprintln!("listening on {addr}");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    handle_client(stream).await;
                });
            }
            Err(err) => {
                eprintln!("accept error: {err:?}");
            }
        }
    }
}
