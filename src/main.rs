use anyhow::Result;
use tracing::{info, warn};
use wtransport::{Endpoint, Identity, ServerConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting WebTransport server...");

    // Create server configuration
    let config = ServerConfig::builder()
        .with_bind_default(8765)
        .with_identity(
            Identity::load_pemfiles("cert.pem", "key.pem")
                .await
                .expect("Failed to load certificates. Run: openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -sha256 -days 365 -nodes -subj '/CN=localhost'")
        )
        .build();

    let server = Endpoint::server(config)?;
    info!("WebTransport server listening on https://localhost:8765");

    // Also start a simple HTTP server for serving the client HTML
    tokio::spawn(async {
        if let Err(e) = start_http_server().await {
            warn!("HTTP server error: {}", e);
        }
    });

    // Accept connections
    loop {
        let incoming_session = server.accept().await;

        tokio::spawn(async move {
            match incoming_session.await {
                Ok(incoming_request) => {
                    info!("New session request from: {:?}", incoming_request.origin());

                    match incoming_request.accept().await {
                        Ok(connection) => {
                            info!("Connection accepted");
                            handle_connection(connection).await;
                        }
                        Err(e) => warn!("Failed to accept connection: {}", e),
                    }
                }
                Err(e) => warn!("Session error: {}", e),
            }
        });
    }
}

async fn handle_connection(connection: wtransport::Connection) {
    info!("Handling connection");

    loop {
        tokio::select! {
            // Handle incoming bidirectional streams
            stream = connection.accept_bi() => {
                match stream {
                    Ok((mut send, mut recv)) => {
                        info!("New bidirectional stream opened");

                        tokio::spawn(async move {
                            // Read data from the stream
                            let mut buffer = vec![0u8; 1024];
                            loop {
                                match recv.read(&mut buffer).await {
                                    Ok(Some(bytes_read)) => {
                                        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                                        info!("Received: {}", message);

                                        // Echo back
                                        let response = format!("Server echo: {}", message);
                                        if let Err(e) = send.write_all(response.as_bytes()).await {
                                            warn!("Failed to send response: {}", e);
                                            break;
                                        }
                                    }
                                    Ok(None) => {
                                        info!("Stream finished");
                                        break;
                                    }
                                    Err(e) => {
                                        warn!("Error reading from stream: {}", e);
                                        break;
                                    }
                                }
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to accept stream: {}", e);
                        break;
                    }
                }
            }

            // Handle incoming datagrams
            datagram = connection.receive_datagram() => {
                match datagram {
                    Ok(data) => {
                        let message = String::from_utf8_lossy(&data);
                        info!("Received datagram: {}", message);

                        // Echo back via datagram
                        let response = format!("Server datagram echo: {}", message);
                        if let Err(e) = connection.send_datagram(response.as_bytes()) {
                            warn!("Failed to send datagram: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Error receiving datagram: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

async fn start_http_server() -> Result<()> {
    use std::net::SocketAddr;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    let addr: SocketAddr = "127.0.0.1:7654".parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("HTTP server listening on http://{}", addr);
    info!("Open http://127.0.0.1:7654 in your browser to test");

    loop {
        let (mut stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let html = include_str!("../client.html");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                html.len(),
                html
            );

            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}
