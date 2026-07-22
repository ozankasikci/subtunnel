use std::time::Duration;

use subtunnel::client::connector::{connect_with_config, ClientControlConfig};
use subtunnel::client::{run_proxy, ConnectTlsOptions, EstablishedConnection};
use subtunnel::server::handler::HeartbeatConfig;
use subtunnel::server::listener::ListenerConfig;
use subtunnel::server::{Server, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn unused_tcp_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

async fn connect_eventually(
    server_addr: &str,
    config: ClientControlConfig,
) -> EstablishedConnection {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match connect_with_config(
                server_addr,
                "soak-token",
                Some("soak"),
                &ConnectTlsOptions {
                    verify: false,
                    ca_path: None,
                },
                config,
            )
            .await
            {
                Ok(connection) => return connection,
                Err(_) => tokio::task::yield_now().await,
            }
        }
    })
    .await
    .expect("soak client could not connect")
}

#[tokio::test]
#[ignore = "15-second end-to-end reliability soak"]
async fn e2e_tunnel_survives_traffic_and_heartbeats() {
    let control_port = unused_tcp_port().await;
    let mut http_port = unused_tcp_port().await;
    while http_port == control_port {
        http_port = unused_tcp_port().await;
    }
    let local_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_address = local_listener.local_addr().unwrap();
    let echo_task = tokio::spawn(async move {
        loop {
            let (mut stream, _) = local_listener.accept().await.unwrap();
            tokio::spawn(async move {
                let mut request = Vec::new();
                stream.read_to_end(&mut request).await.unwrap();
                stream.write_all(&request).await.unwrap();
                stream.shutdown().await.unwrap();
            });
        }
    });

    let server = Server::new_with_timing(
        ServerConfig {
            control_port,
            http_port,
            auth_token: Some("soak-token".into()),
            host: "127.0.0.1".into(),
            domain: "tunnel.example.test".into(),
            ..ServerConfig::default()
        },
        HeartbeatConfig {
            interval: Duration::from_millis(100),
            miss_limit: 3,
            write_timeout: Duration::from_secs(1),
        },
        ListenerConfig {
            initial_read_timeout: Duration::from_secs(1),
            open_stream_timeout: Duration::from_secs(1),
        },
    );
    let tunnel_mgr = server.tunnel_manager().clone();
    let server_task = tokio::spawn(async move { server.run().await });
    let connection = connect_eventually(
        &format!("127.0.0.1:{control_port}"),
        ClientControlConfig {
            heartbeat_interval: Duration::from_millis(50),
            heartbeat_timeout: Duration::from_millis(500),
            write_timeout: Duration::from_secs(1),
        },
    )
    .await;

    let EstablishedConnection {
        mux,
        tunnel_info: _,
        alive,
        _control_handle: control_guard,
    } = connection;
    let alive_for_assertions = alive.clone();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let local_address = local_address.to_string();
    let proxy_task =
        tokio::spawn(async move { run_proxy(mux, &local_address, shutdown_rx, alive).await });

    for request_index in 0..200 {
        let request = format!(
            "POST /{request_index} HTTP/1.1\r\nHost: soak.tunnel.example.test\r\nContent-Length: 11\r\n\r\npayload-{request_index:03}"
        )
        .into_bytes();
        let echoed = tokio::time::timeout(Duration::from_secs(2), async {
            let mut visitor = TcpStream::connect(("127.0.0.1", http_port)).await?;
            visitor.write_all(&request).await?;
            visitor.shutdown().await?;
            let mut echoed = Vec::new();
            visitor.read_to_end(&mut echoed).await?;
            Ok::<_, std::io::Error>(echoed)
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(echoed, request, "request {request_index} failed");
        assert!(
            *alive_for_assertions.borrow(),
            "control task marked the only connection dead at request {request_index}"
        );
        tokio::time::sleep(Duration::from_millis(75)).await;
    }

    assert_eq!(tunnel_mgr.tunnel_count().await, 1);
    assert!(*alive_for_assertions.borrow());
    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(2), proxy_task)
        .await
        .expect("client proxy did not stop")
        .unwrap()
        .unwrap();

    drop(control_guard);
    server_task.abort();
    echo_task.abort();
}
