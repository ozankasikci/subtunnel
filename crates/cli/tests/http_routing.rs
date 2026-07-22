use std::time::Duration;

use subtunnel::server::listener::{
    proxy_connections_with_config, serve_http_listener, ListenerConfig,
};
use subtunnel::server::tunnel_mgr::TunnelManager;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn start_http_listener(
    tunnel_mgr: TunnelManager,
    config: ListenerConfig,
) -> (std::net::SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        serve_http_listener(
            listener,
            vec!["tunnel.example.test".into()],
            tunnel_mgr,
            config,
        )
        .await
        .unwrap();
    });
    (address, task)
}

#[tokio::test]
async fn unknown_subdomain_gets_http_404() {
    let (address, listener) =
        start_http_listener(TunnelManager::new(), ListenerConfig::default()).await;
    let mut client = TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: missing.tunnel.example.test\r\n\r\n")
        .await
        .unwrap();

    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(2), client.read_to_end(&mut response))
        .await
        .expect("HTTP listener did not close the failed request")
        .unwrap();
    assert!(
        response.starts_with(b"HTTP/1.1 404 Not Found\r\n"),
        "unexpected response: {:?}",
        String::from_utf8_lossy(&response)
    );
    listener.abort();
}

#[tokio::test]
async fn binary_post_routes_to_tunnel() {
    let tunnel_mgr = TunnelManager::new();
    let mut registered = tunnel_mgr
        .register("agent", "tcp", Some("binary"))
        .await
        .unwrap();
    let (address, listener) = start_http_listener(tunnel_mgr, ListenerConfig::default()).await;

    let body: Vec<u8> = (0..=255).collect();
    let mut request = format!(
        "POST /upload HTTP/1.1\r\nHost: binary.tunnel.example.test\r\nContent-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    request.extend_from_slice(&body);

    let mut client = TcpStream::connect(address).await.unwrap();
    client.write_all(&request).await.unwrap();

    let (_stream, preread) =
        tokio::time::timeout(Duration::from_secs(2), registered.conn_rx.recv())
            .await
            .expect("binary request was not routed")
            .expect("tunnel route channel closed");
    assert_eq!(preread, request);
    listener.abort();
}

#[tokio::test]
async fn slow_client_initial_read_times_out() {
    let (address, listener) = start_http_listener(
        TunnelManager::new(),
        ListenerConfig {
            initial_read_timeout: Duration::from_millis(100),
            open_stream_timeout: Duration::from_secs(1),
        },
    )
    .await;
    let mut client = TcpStream::connect(address).await.unwrap();

    let mut byte = [0u8; 1];
    let read = tokio::time::timeout(Duration::from_secs(2), client.read(&mut byte))
        .await
        .expect("idle HTTP connection did not time out")
        .unwrap();
    assert_eq!(read, 0, "idle HTTP connection remained open");
    listener.abort();
}

#[tokio::test]
async fn closed_tunnel_channel_gets_http_502() {
    let tunnel_mgr = TunnelManager::new();
    let registered = tunnel_mgr
        .register("agent", "tcp", Some("closed"))
        .await
        .unwrap();
    drop(registered.conn_rx);
    let (address, listener) = start_http_listener(tunnel_mgr, ListenerConfig::default()).await;
    let mut client = TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: closed.tunnel.example.test\r\n\r\n")
        .await
        .unwrap();

    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(2), client.read_to_end(&mut response))
        .await
        .expect("HTTP listener did not close the failed request")
        .unwrap();
    assert!(response.starts_with(b"HTTP/1.1 502 Bad Gateway\r\n"));
    listener.abort();
}

#[tokio::test]
async fn hung_open_does_not_block_other_visitors() {
    let (conn_tx, conn_rx) = tokio::sync::mpsc::channel(4);
    let (first_started_tx, first_started_rx) = tokio::sync::oneshot::channel();
    let first_started_tx = std::sync::Arc::new(std::sync::Mutex::new(Some(first_started_tx)));
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (opened_tx, mut opened_rx) = tokio::sync::mpsc::unbounded_channel();

    let proxy = tokio::spawn(proxy_connections_with_config(
        "t_hol".into(),
        conn_rx,
        {
            let calls = calls.clone();
            let first_started_tx = first_started_tx.clone();
            move || {
                let call = calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let first_started_tx = first_started_tx.clone();
                let opened_tx = opened_tx.clone();
                async move {
                    if call == 0 {
                        if let Some(tx) = first_started_tx.lock().unwrap().take() {
                            let _ = tx.send(());
                        }
                        std::future::pending::<()>().await;
                        unreachable!()
                    }
                    let (proxy_side, observer_side) = tokio::io::duplex(64);
                    opened_tx.send(observer_side).unwrap();
                    Ok(proxy_side)
                }
            }
        },
        ListenerConfig {
            initial_read_timeout: Duration::from_secs(1),
            open_stream_timeout: Duration::from_millis(200),
        },
    ));

    let (mut visitor_a, server_a) = tokio::io::duplex(256);
    conn_tx.send((server_a, b"A".to_vec())).await.unwrap();
    tokio::time::timeout(Duration::from_secs(1), first_started_rx)
        .await
        .expect("first open did not start")
        .unwrap();

    let (_visitor_b, server_b) = tokio::io::duplex(64);
    conn_tx.send((server_b, b"B".to_vec())).await.unwrap();
    let mut observer = tokio::time::timeout(Duration::from_secs(1), opened_rx.recv())
        .await
        .expect("visitor B starved behind visitor A")
        .expect("open observer closed");
    let mut byte = [0u8; 1];
    tokio::time::timeout(Duration::from_secs(1), observer.read_exact(&mut byte))
        .await
        .expect("visitor B preread was not proxied")
        .unwrap();
    assert_eq!(&byte, b"B");

    let mut response = Vec::new();
    tokio::time::timeout(Duration::from_secs(1), visitor_a.read_to_end(&mut response))
        .await
        .expect("hung visitor did not receive a bounded failure")
        .unwrap();
    assert!(response.starts_with(b"HTTP/1.1 502 Bad Gateway\r\n"));

    proxy.abort();
}
