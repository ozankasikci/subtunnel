mod common;

use std::collections::BTreeMap;
use std::time::Duration;

use subtunnel::config::{select_tunnels, Config, TunnelConfig};
use subtunnel::runner::run_tunnels;
use subtunnel::server::auth::Authenticator;
use subtunnel::server::handler::handle_agent_connection;
use subtunnel::server::tunnel_mgr::TunnelManager;
use subtunnel::server::{Server, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const DOMAIN: &str = "tunnel.example.test";
const TOKEN: &str = "test-token";

async fn unused_tcp_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

async fn start_server() -> (
    u16,
    u16,
    TunnelManager,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let control_port = unused_tcp_port().await;
    let mut http_port = unused_tcp_port().await;
    while http_port == control_port {
        http_port = unused_tcp_port().await;
    }

    let server = Server::new(ServerConfig {
        control_port,
        http_port,
        auth_token: Some(TOKEN.into()),
        host: "127.0.0.1".into(),
        domain: DOMAIN.into(),
        ..ServerConfig::default()
    });
    let tunnel_manager = server.tunnel_manager().clone();
    let server_task = tokio::spawn(async move { server.run().await });

    (control_port, http_port, tunnel_manager, server_task)
}

async fn start_local_http(body: &'static str) -> (u16, tokio::task::JoinHandle<()>, Vec<u8>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes();
    let response_for_task = response.clone();
    let task = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let response = response_for_task.clone();
            tokio::spawn(async move {
                let mut request = Vec::new();
                stream.read_to_end(&mut request).await.unwrap();
                stream.write_all(&response).await.unwrap();
                stream.shutdown().await.unwrap();
            });
        }
    });

    (port, task, response)
}

fn config_with_tunnels(
    control_port: u16,
    tunnels: impl IntoIterator<Item = (&'static str, u16, &'static str)>,
) -> Config {
    Config {
        server: format!("127.0.0.1:{control_port}"),
        token: TOKEN.into(),
        tls_verify: false,
        tls_ca: None,
        tunnels: tunnels
            .into_iter()
            .map(|(name, local_port, subdomain)| {
                (
                    name.to_string(),
                    TunnelConfig {
                        local_port,
                        subdomain: Some(subdomain.to_string()),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>(),
    }
}

async fn request_once(http_port: u16, subdomain: &str) -> std::io::Result<Vec<u8>> {
    let request =
        format!("GET / HTTP/1.1\r\nHost: {subdomain}.{DOMAIN}\r\nConnection: close\r\n\r\n");
    let mut visitor = TcpStream::connect(("127.0.0.1", http_port)).await?;
    visitor.write_all(request.as_bytes()).await?;
    visitor.shutdown().await?;
    let mut response = Vec::new();
    visitor.read_to_end(&mut response).await?;
    Ok(response)
}

async fn wait_for_route(
    http_port: u16,
    subdomain: &str,
    expected_response: &[u8],
    runner_task: &tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            assert!(
                !runner_task.is_finished(),
                "run_tunnels stopped before {subdomain} served traffic"
            );
            if let Ok(response) = request_once(http_port, subdomain).await {
                if response == expected_response {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("{subdomain} did not serve traffic before the timeout"));
}

#[tokio::test]
async fn two_tunnels_route_independently_and_shutdown_cleanly() {
    let (first_port, first_local, first_response) = start_local_http("first").await;
    let (second_port, second_local, second_response) = start_local_http("second").await;
    let (control_port, http_port, tunnel_manager, server_task) = start_server().await;
    let config = config_with_tunnels(
        control_port,
        [
            ("first", first_port, "first"),
            ("second", second_port, "second"),
        ],
    );
    let tunnels = select_tunnels(&config, true, &[]).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut runner_task =
        tokio::spawn(async move { run_tunnels(&config, tunnels, shutdown_rx).await });

    wait_for_route(http_port, "first", &first_response, &runner_task).await;
    wait_for_route(http_port, "second", &second_response, &runner_task).await;
    assert_eq!(tunnel_manager.tunnel_count().await, 2);

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(2), &mut runner_task)
        .await
        .expect("run_tunnels did not stop after shutdown")
        .unwrap()
        .expect("run_tunnels returned an error after clean shutdown");

    server_task.abort();
    first_local.abort();
    second_local.abort();
}

#[tokio::test]
async fn hard_registration_error_does_not_stop_surviving_tunnel() {
    let (rejected_port, rejected_local, _) = start_local_http("rejected").await;
    let (survivor_port, survivor_local, survivor_response) = start_local_http("survivor").await;
    let (control_port, http_port, tunnel_manager, server_task) = start_server().await;

    let (reservation_server, reservation_agent) = tokio::io::duplex(64 * 1024);
    let reservation_handler = tokio::spawn(handle_agent_connection(
        reservation_server,
        tunnel_manager.clone(),
        Authenticator::new(TOKEN.into()),
        DOMAIN.into(),
    ));
    let reservation = common::FakeAgent::connect(reservation_agent, "taken")
        .await
        .expect("failed to reserve the rejected subdomain");
    assert_eq!(tunnel_manager.tunnel_count().await, 1);

    let config = config_with_tunnels(
        control_port,
        [
            ("a-rejected", rejected_port, "taken"),
            ("z-survivor", survivor_port, "survivor"),
        ],
    );
    let tunnels = select_tunnels(&config, true, &[]).unwrap();
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut runner_task =
        tokio::spawn(async move { run_tunnels(&config, tunnels, shutdown_rx).await });

    wait_for_route(http_port, "survivor", &survivor_response, &runner_task).await;
    assert_eq!(tunnel_manager.tunnel_count().await, 2);

    shutdown_tx.send(true).unwrap();
    let error = tokio::time::timeout(Duration::from_secs(2), &mut runner_task)
        .await
        .expect("run_tunnels did not stop after shutdown")
        .unwrap()
        .expect_err("run_tunnels did not report the rejected tunnel");
    assert!(
        format!("{error:#}").contains("1 tunnel task(s) failed"),
        "unexpected run_tunnels error: {error:#}"
    );

    drop(reservation);
    reservation_handler.abort();
    server_task.abort();
    rejected_local.abort();
    survivor_local.abort();
}
