mod common;

use std::time::Duration;

use subtunnel::client::connector::{spawn_control_task, ClientControlConfig};
use subtunnel::client::{connect_with_config, Client, ConnectTlsOptions, EstablishedConnection};
use subtunnel::protocol::codec::{read_message, write_message};
use subtunnel::protocol::ControlMessage;
use subtunnel::server::auth::Authenticator;
use subtunnel::server::handler::{handle_agent_connection_with_config, HeartbeatConfig};
use subtunnel::server::tunnel_mgr::TunnelManager;
use subtunnel::server::{Server, ServerConfig};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn miss_limit_one_sends_one_heartbeat_before_unregistering_dead_agent() {
    let (server_io, agent_io) = tokio::io::duplex(64 * 1024);
    let tunnel_mgr = TunnelManager::new();
    let mut handler = tokio::spawn(handle_agent_connection_with_config(
        server_io,
        tunnel_mgr.clone(),
        Authenticator::new("test-token".into()),
        "example.test".into(),
        HeartbeatConfig {
            interval: Duration::from_millis(20),
            miss_limit: 1,
            write_timeout: Duration::from_millis(100),
        },
    ));

    let common::FakeAgent {
        mux: _mux,
        mut control,
    } = common::FakeAgent::connect(agent_io, "myapp")
        .await
        .expect("fake agent handshake failed");
    assert_eq!(tunnel_mgr.tunnel_count().await, 1);

    let heartbeat = tokio::time::timeout(Duration::from_millis(200), read_message(&mut control))
        .await
        .expect("server did not send a heartbeat before the miss limit")
        .expect("failed to read server heartbeat")
        .expect("server disconnected before sending a heartbeat");
    assert_eq!(heartbeat, ControlMessage::Heartbeat);

    let result = tokio::time::timeout(Duration::from_millis(500), &mut handler).await;
    assert!(
        result.is_ok(),
        "agent handler did not exit after the heartbeat miss limit"
    );
    result.unwrap().unwrap().unwrap();

    let next_message = tokio::time::timeout(Duration::from_millis(200), read_message(&mut control))
        .await
        .expect("agent control stream did not close after disconnect")
        .expect("failed to read agent control stream after disconnect");
    assert_eq!(next_message, None, "server sent more than one heartbeat");

    assert_eq!(tunnel_mgr.tunnel_count().await, 0);
    let replacement = tunnel_mgr
        .register("replacement-agent", "tcp", Some("myapp"))
        .await;
    assert!(
        replacement.is_ok(),
        "subdomain remained reserved after dead agent"
    );
}

#[tokio::test]
async fn healthy_agent_stays_connected() {
    let (server_io, agent_io) = tokio::io::duplex(64 * 1024);
    let tunnel_mgr = TunnelManager::new();
    let handler = tokio::spawn(handle_agent_connection_with_config(
        server_io,
        tunnel_mgr.clone(),
        Authenticator::new("test-token".into()),
        "example.test".into(),
        HeartbeatConfig {
            interval: Duration::from_millis(10),
            miss_limit: 3,
            write_timeout: Duration::from_millis(100),
        },
    ));

    let fake_agent = common::FakeAgent::connect(agent_io, "healthy")
        .await
        .expect("fake agent handshake failed");
    let _mux = fake_agent.mux;
    let mut control = fake_agent.control;
    let (ack_tx, mut ack_rx) = tokio::sync::mpsc::channel(32);
    let responder = tokio::spawn(async move {
        loop {
            match read_message(&mut control).await {
                Ok(Some(ControlMessage::Heartbeat)) => {
                    write_message(&mut control, &ControlMessage::HeartbeatAck)
                        .await
                        .unwrap();
                    ack_tx.send(()).await.unwrap();
                }
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => return,
            }
        }
    });

    for _ in 0..20 {
        tokio::time::timeout(Duration::from_secs(1), ack_rx.recv())
            .await
            .expect("heartbeat acknowledgement timed out")
            .expect("heartbeat responder stopped");
    }

    assert!(!handler.is_finished(), "healthy agent was disconnected");
    assert_eq!(tunnel_mgr.tunnel_count().await, 1);
    responder.abort();
    handler.abort();
}

#[tokio::test]
async fn partial_frame_across_ticks_not_corrupted() {
    let (client_io, server_io) = tokio::io::duplex(64 * 1024);
    let slow_client = common::SlowStream::new(client_io, Duration::from_millis(30));
    let (observed_tx, mut observed_rx) = tokio::sync::mpsc::unbounded_channel();
    let (alive, control_handle) = spawn_control_task(
        slow_client,
        ClientControlConfig {
            heartbeat_interval: Duration::from_millis(20),
            heartbeat_timeout: Duration::from_secs(3),
            write_timeout: Duration::from_millis(100),
        },
        Some(observed_tx),
    );

    let (mut server_read, mut server_write) = tokio::io::split(server_io);
    write_message(&mut server_write, &ControlMessage::Heartbeat)
        .await
        .unwrap();

    let (heartbeat_tx, mut heartbeat_rx) = tokio::sync::mpsc::channel(64);
    let drain = tokio::spawn(async move {
        while let Ok(Some(message)) = read_message(&mut server_read).await {
            if message == ControlMessage::Heartbeat && heartbeat_tx.send(()).await.is_err() {
                return;
            }
        }
    });

    for received in 0..50 {
        if tokio::time::timeout(Duration::from_secs(2), heartbeat_rx.recv())
            .await
            .ok()
            .flatten()
            .is_none()
        {
            panic!(
                "client stopped after {received} heartbeat ticks; alive={}",
                *alive.borrow()
            );
        }
    }

    assert_eq!(
        tokio::time::timeout(Duration::from_secs(1), observed_rx.recv())
            .await
            .unwrap(),
        Some(ControlMessage::Heartbeat)
    );
    assert!(
        *alive.borrow(),
        "slow valid frame marked the connection dead"
    );

    control_handle.abort();
    drain.abort();
}

#[tokio::test]
async fn interleaved_messages_under_tick_pressure() {
    let (client_io, server_io) = tokio::io::duplex(64 * 1024);
    let (observed_tx, mut observed_rx) = tokio::sync::mpsc::unbounded_channel();
    let (alive, control_handle) = spawn_control_task(
        client_io,
        ClientControlConfig {
            heartbeat_interval: Duration::from_millis(2),
            heartbeat_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_millis(100),
        },
        Some(observed_tx),
    );

    let (mut server_read, mut server_write) = tokio::io::split(server_io);
    let drain = tokio::spawn(async move {
        while read_message(&mut server_read)
            .await
            .ok()
            .flatten()
            .is_some()
        {}
    });
    let sender = tokio::spawn(async move {
        for message_index in 0..100 {
            let frame =
                subtunnel::protocol::codec::encode_message(&ControlMessage::Heartbeat).unwrap();
            for (byte_index, byte) in frame.into_iter().enumerate() {
                server_write.write_all(&[byte]).await.unwrap();
                if (message_index + byte_index) % 7 == 0 {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                } else {
                    tokio::task::yield_now().await;
                }
            }
        }
    });

    for parsed in 0..100 {
        let message = tokio::time::timeout(Duration::from_secs(5), observed_rx.recv())
            .await
            .unwrap_or_else(|_| panic!("only parsed {parsed} of 100 messages"))
            .expect("control observer closed early");
        assert_eq!(message, ControlMessage::Heartbeat);
    }
    sender.await.unwrap();
    assert!(
        *alive.borrow(),
        "valid interleaved messages marked connection dead"
    );

    control_handle.abort();
    drain.abort();
}

#[tokio::test]
async fn client_write_stall_triggers_dead_detection() {
    let (client_io, _undrained_server_io) = tokio::io::duplex(64);
    let (mut alive, control_handle) = spawn_control_task(
        client_io,
        ClientControlConfig {
            heartbeat_interval: Duration::from_millis(10),
            heartbeat_timeout: Duration::from_millis(80),
            write_timeout: Duration::from_millis(100),
        },
        None,
    );

    tokio::time::timeout(Duration::from_millis(500), async {
        while *alive.borrow() {
            alive
                .changed()
                .await
                .expect("control task dropped alive sender");
        }
    })
    .await
    .expect("stalled control write prevented dead-connection detection");

    control_handle.abort();
}

#[tokio::test]
async fn server_write_stall_disconnects_agent() {
    let (server_io, agent_io) = tokio::io::duplex(64);
    let (gated_agent_io, read_enabled) = common::ReadGate::new(agent_io);
    let tunnel_mgr = TunnelManager::new();
    let mut handler = tokio::spawn(handle_agent_connection_with_config(
        server_io,
        tunnel_mgr.clone(),
        Authenticator::new("test-token".into()),
        "example.test".into(),
        HeartbeatConfig {
            interval: Duration::from_millis(1),
            miss_limit: u32::MAX,
            write_timeout: Duration::from_millis(50),
        },
    ));

    let _stalled_agent = common::FakeAgent::connect(gated_agent_io, "stalled")
        .await
        .expect("fake agent handshake failed");
    read_enabled.store(false, std::sync::atomic::Ordering::SeqCst);

    tokio::time::timeout(Duration::from_secs(5), &mut handler)
        .await
        .expect("server handler remained wedged on a stalled control write")
        .unwrap()
        .unwrap();
    assert_eq!(tunnel_mgr.tunnel_count().await, 0);
}

async fn unused_tcp_port() -> u16 {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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
                "test-token",
                Some("reconnect"),
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
    .expect("client did not establish within the retry bound")
}

#[tokio::test]
async fn reconnect_same_subdomain_after_drop() {
    let control_port = unused_tcp_port().await;
    let mut http_port = unused_tcp_port().await;
    while http_port == control_port {
        http_port = unused_tcp_port().await;
    }
    let server = Server::new_with_timing(
        ServerConfig {
            control_port,
            http_port,
            auth_token: Some("test-token".into()),
            host: "127.0.0.1".into(),
            domain: "tunnel.example.test".into(),
            ..ServerConfig::default()
        },
        HeartbeatConfig {
            interval: Duration::from_millis(50),
            miss_limit: 3,
            write_timeout: Duration::from_millis(100),
        },
        subtunnel::server::listener::ListenerConfig::default(),
    );
    let tunnel_mgr = server.tunnel_manager().clone();
    let server_task = tokio::spawn(async move { server.run().await });
    let address = format!("127.0.0.1:{control_port}");
    let client_config = ClientControlConfig {
        heartbeat_interval: Duration::from_millis(20),
        heartbeat_timeout: Duration::from_millis(200),
        write_timeout: Duration::from_millis(100),
    };

    let first = connect_eventually(&address, client_config).await;
    assert_eq!(first.tunnel_info.subdomain, "reconnect");
    assert_eq!(tunnel_mgr.tunnel_count().await, 1);

    let killed_client = tokio::spawn(async move {
        std::future::pending::<()>().await;
        drop(first);
    });
    killed_client.abort();
    let _ = killed_client.await;

    let second = connect_eventually(&address, client_config).await;
    assert_eq!(second.tunnel_info.subdomain, "reconnect");
    assert_eq!(tunnel_mgr.tunnel_count().await, 1);

    drop(second);
    server_task.abort();
}

#[tokio::test]
async fn client_run_keeps_control_task_alive_while_proxying() {
    let local_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_port = local_listener.local_addr().unwrap().port();
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

    let control_port = unused_tcp_port().await;
    let mut http_port = unused_tcp_port().await;
    while http_port == control_port {
        http_port = unused_tcp_port().await;
    }
    let server = Server::new_with_timing(
        ServerConfig {
            control_port,
            http_port,
            auth_token: Some("test-token".into()),
            host: "127.0.0.1".into(),
            domain: "tunnel.example.test".into(),
            ..ServerConfig::default()
        },
        HeartbeatConfig {
            interval: Duration::from_millis(25),
            miss_limit: 3,
            write_timeout: Duration::from_millis(100),
        },
        subtunnel::server::listener::ListenerConfig {
            initial_read_timeout: Duration::from_secs(1),
            open_stream_timeout: Duration::from_secs(1),
        },
    );
    let tunnel_mgr = server.tunnel_manager().clone();
    let server_task = tokio::spawn(async move { server.run().await });

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let client = Client::new(
        format!("127.0.0.1:{control_port}"),
        "test-token".into(),
        local_port,
        Some("client-run".into()),
        ConnectTlsOptions {
            verify: false,
            ca_path: None,
        },
    );
    let mut client_task = tokio::spawn(client.run(shutdown_rx));

    tokio::time::timeout(Duration::from_secs(5), async {
        while tunnel_mgr.tunnel_count().await != 1 {
            assert!(
                !client_task.is_finished(),
                "Client::run stopped before registering"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("Client::run did not register a tunnel");

    let request =
        b"GET / HTTP/1.1\r\nHost: client-run.tunnel.example.test\r\nConnection: close\r\n\r\n";
    let echoed = tokio::time::timeout(Duration::from_secs(2), async {
        let mut visitor = loop {
            match tokio::net::TcpStream::connect(("127.0.0.1", http_port)).await {
                Ok(visitor) => break visitor,
                Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
            }
        };
        visitor.write_all(request).await?;
        visitor.shutdown().await?;
        let mut echoed = Vec::new();
        visitor.read_to_end(&mut echoed).await?;
        Ok::<_, std::io::Error>(echoed)
    })
    .await
    .expect("request through Client::run timed out")
    .expect("request through Client::run failed");
    assert_eq!(echoed, request);

    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        tunnel_mgr.tunnel_count().await,
        1,
        "Client::run dropped the control task while proxying"
    );
    assert!(
        !client_task.is_finished(),
        "Client::run stopped while connected"
    );

    shutdown_tx.send(true).unwrap();
    tokio::time::timeout(Duration::from_secs(2), &mut client_task)
        .await
        .expect("Client::run did not stop after shutdown")
        .unwrap()
        .unwrap();

    server_task.abort();
    echo_task.abort();
}
