//! Duplex in-memory pour tests et développement local hors réseau.

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::mpsc;

use crate::connection::Connection;
use crate::error::TransportError;

/// Une extrémité d’un canal mémoire bidirectionnel.
pub struct InMemoryConnection {
    tx: Option<mpsc::Sender<Bytes>>,
    rx: mpsc::Receiver<Bytes>,
}

impl InMemoryConnection {
    /// Crée une paire de connexions reliées (A ↔ B).
    #[must_use]
    pub fn pair(buffer: usize) -> (Self, Self) {
        let (a_to_b_tx, a_to_b_rx) = mpsc::channel(buffer);
        let (b_to_a_tx, b_to_a_rx) = mpsc::channel(buffer);
        (
            Self {
                tx: Some(a_to_b_tx),
                rx: b_to_a_rx,
            },
            Self {
                tx: Some(b_to_a_tx),
                rx: a_to_b_rx,
            },
        )
    }
}

#[async_trait]
impl Connection for InMemoryConnection {
    async fn send(&mut self, frame: Bytes) -> Result<(), TransportError> {
        let Some(tx) = &self.tx else {
            return Err(TransportError::Closed);
        };
        tx.send(frame).await.map_err(|_| TransportError::Closed)
    }

    async fn recv(&mut self) -> Result<Bytes, TransportError> {
        self.rx.recv().await.ok_or(TransportError::Closed)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        self.tx.take();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{recv_message, send_message};
    use teleportal_protocol::{CursorId, Message, Role, PROTOCOL_VERSION};

    #[tokio::test]
    async fn cursor_move_round_trip() {
        let (mut a, mut b) = InMemoryConnection::pair(8);
        let msg = Message::CursorMove {
            cursor_id: CursorId::new(),
            x: 0.33,
            y: 0.66,
            timestamp_ms: 99,
        };
        send_message(&mut a, &msg).await.expect("send");
        let got = recv_message(&mut b).await.expect("recv");
        assert_eq!(got, msg);
    }

    #[tokio::test]
    async fn hello_round_trip() {
        let (mut a, mut b) = InMemoryConnection::pair(4);
        let msg = Message::Hello {
            protocol_version: PROTOCOL_VERSION,
            role: Role::Guest,
        };
        send_message(&mut a, &msg).await.unwrap();
        assert_eq!(recv_message(&mut b).await.unwrap(), msg);
    }

    #[tokio::test]
    async fn close_stops_recv() {
        let (mut a, mut b) = InMemoryConnection::pair(1);
        a.close().await.unwrap();
        // Drop sender side fully by closing a; b recv should eventually see closed
        // after a is dropped — close only drops tx on a, so b's rx from a closes.
        drop(a);
        let err = b.recv().await.expect_err("closed");
        assert!(matches!(err, TransportError::Closed));
    }
}
