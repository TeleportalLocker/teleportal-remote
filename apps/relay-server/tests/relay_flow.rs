//! Test d’intégration : Create → Join → forward CursorMove.

use std::sync::Arc;
use std::time::Duration;

use teleportal_protocol::{CursorId, Message, Role, SessionCode, PROTOCOL_VERSION};
use teleportal_relay_server::config::Config;
use teleportal_relay_server::rate_limit::RateLimiter;
use teleportal_relay_server::ws::{serve_ephemeral, AppState};
use teleportal_relay_server::SessionRegistry;
use teleportal_transport::{recv_message, send_message, Connection, WebsocketConnection};

fn test_config() -> Config {
    Config {
        bind: "127.0.0.1:0".parse().unwrap(),
        session_ttl: Duration::from_secs(600),
        cleanup_interval: Duration::from_secs(30),
        rate_limit_per_min: 0,
    }
}

fn test_state(config: Config) -> AppState {
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_per_min));
    AppState {
        registry: SessionRegistry::new(),
        config: Arc::new(config),
        rate_limiter,
    }
}

#[tokio::test]
async fn create_join_forward_cursor() {
    let (addr, _handle) = serve_ephemeral(test_state(test_config()))
        .await
        .expect("bind");
    let url = format!("ws://{addr}/ws");

    let mut host = WebsocketConnection::connect(&url)
        .await
        .expect("host connect");
    let mut guest = WebsocketConnection::connect(&url)
        .await
        .expect("guest connect");

    send_message(
        &mut host,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Host,
        },
    )
    .await
    .unwrap();
    send_message(&mut host, &Message::CreateSession)
        .await
        .unwrap();

    let created = recv_message(&mut host).await.expect("session created");
    let code = match created {
        Message::SessionCreated { code, .. } => code,
        other => panic!("expected SessionCreated, got {other:?}"),
    };

    send_message(
        &mut guest,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Guest,
        },
    )
    .await
    .unwrap();
    send_message(&mut guest, &Message::JoinSession { code: code.clone() })
        .await
        .unwrap();

    let accepted = recv_message(&mut guest).await.expect("join accepted");
    assert!(matches!(accepted, Message::JoinAccepted { .. }));

    let peer_joined = recv_message(&mut host).await.expect("peer joined");
    assert!(matches!(peer_joined, Message::PeerJoined { .. }));

    let cursor = Message::CursorMove {
        cursor_id: CursorId::new(),
        x: 0.25,
        y: 0.75,
        timestamp_ms: 123,
    };
    send_message(&mut host, &cursor).await.unwrap();
    let forwarded = recv_message(&mut guest).await.expect("cursor forward");
    assert_eq!(forwarded, cursor);

    host.close().await.ok();
    guest.close().await.ok();
}

#[tokio::test]
async fn join_rejects_unknown_code() {
    let (addr, _handle) = serve_ephemeral(test_state(test_config())).await.unwrap();
    let url = format!("ws://{addr}/ws");

    let mut guest = WebsocketConnection::connect(&url).await.unwrap();
    send_message(
        &mut guest,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Guest,
        },
    )
    .await
    .unwrap();
    send_message(
        &mut guest,
        &Message::JoinSession {
            code: SessionCode::parse("999999").unwrap(),
        },
    )
    .await
    .unwrap();

    let rejected = recv_message(&mut guest).await.unwrap();
    assert!(matches!(rejected, Message::JoinRejected { .. }));
}

#[tokio::test]
async fn rate_limit_blocks_excess_joins() {
    let mut config = test_config();
    config.rate_limit_per_min = 2;
    let (addr, _handle) = serve_ephemeral(test_state(config)).await.unwrap();
    let url = format!("ws://{addr}/ws");

    for i in 0..2 {
        let mut guest = WebsocketConnection::connect(&url).await.unwrap();
        send_message(
            &mut guest,
            &Message::Hello {
                protocol_version: PROTOCOL_VERSION,
                role: Role::Guest,
            },
        )
        .await
        .unwrap();
        send_message(
            &mut guest,
            &Message::JoinSession {
                code: SessionCode::parse("999999").unwrap(),
            },
        )
        .await
        .unwrap();
        let msg = recv_message(&mut guest).await.unwrap();
        assert!(
            matches!(msg, Message::JoinRejected { .. }),
            "attempt {i}: expected JoinRejected, got {msg:?}"
        );
        guest.close().await.ok();
    }

    let mut guest = WebsocketConnection::connect(&url).await.unwrap();
    send_message(
        &mut guest,
        &Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Guest,
        },
    )
    .await
    .unwrap();
    send_message(
        &mut guest,
        &Message::JoinSession {
            code: SessionCode::parse("999999").unwrap(),
        },
    )
    .await
    .unwrap();
    let limited = recv_message(&mut guest).await.unwrap();
    match limited {
        Message::Error { code, .. } => assert_eq!(code, "rate_limited"),
        other => panic!("expected rate_limited error, got {other:?}"),
    }
}

#[tokio::test]
async fn health_ok() {
    let (addr, _handle) = serve_ephemeral(test_state(test_config())).await.unwrap();
    let body = tokio::task::spawn_blocking(move || {
        use std::io::{Read, Write};
        use std::net::TcpStream;
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut buf = String::new();
        stream.read_to_string(&mut buf).unwrap();
        buf
    })
    .await
    .unwrap();
    assert!(body.contains("200"));
    assert!(body.contains("ok"));
}
