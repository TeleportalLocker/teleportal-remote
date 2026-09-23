//! Traitement des messages signaling et décision de forward.

use std::net::IpAddr;

use teleportal_protocol::{Message, PeerId, Role, SessionId};

use crate::rate_limit::RateLimiter;
use crate::state::{RegistryError, SessionRegistry};
use bytes::Bytes;
use tokio::sync::mpsc;

/// État d’une connexion WS côté handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnPhase {
    /// En attente du `Hello`.
    ExpectHello,
    /// Hello reçu, en attente Create/Join.
    Negotiating {
        /// Rôle annoncé.
        role: Role,
    },
    /// Attaché à une session.
    InSession {
        /// Session.
        session_id: SessionId,
        /// Identifiant local.
        peer_id: PeerId,
        /// Rôle effectif.
        role: Role,
    },
}

/// Réponse locale à renvoyer sur la même connexion.
#[derive(Debug)]
pub enum HandlerOutput {
    /// Message à envoyer au client local.
    Reply(Message),
    /// Fermer après envoi optionnel.
    Close(Option<Message>),
}

/// Résultat du traitement d’un message.
pub struct HandleResult {
    /// Nouvelle phase éventuelle.
    pub phase: ConnPhase,
    /// Sorties locales (ordered).
    pub outputs: Vec<HandlerOutput>,
}

/// Traite un message entrant pour une connexion.
pub async fn handle_message(
    registry: &SessionRegistry,
    limiter: &RateLimiter,
    peer_ip: IpAddr,
    phase: ConnPhase,
    outbound: &mpsc::Sender<Bytes>,
    message: Message,
) -> HandleResult {
    match phase {
        ConnPhase::ExpectHello => handle_hello(message),
        ConnPhase::Negotiating { role } => {
            handle_negotiate(registry, limiter, peer_ip, role, outbound, message).await
        }
        ConnPhase::InSession {
            session_id,
            peer_id,
            role,
        } => handle_in_session(registry, session_id, peer_id, role, message).await,
    }
}

fn handle_hello(message: Message) -> HandleResult {
    match message {
        Message::Hello {
            protocol_version: _,
            role,
        } => HandleResult {
            // validate_message already enforced version on decode
            phase: ConnPhase::Negotiating { role },
            outputs: vec![],
        },
        other => HandleResult {
            phase: ConnPhase::ExpectHello,
            outputs: vec![HandlerOutput::Close(Some(Message::Error {
                code: "expected_hello".into(),
                message: format!("expected Hello, got {other:?}"),
            }))],
        },
    }
}

async fn handle_negotiate(
    registry: &SessionRegistry,
    limiter: &RateLimiter,
    peer_ip: IpAddr,
    role: Role,
    outbound: &mpsc::Sender<Bytes>,
    message: Message,
) -> HandleResult {
    match (role, message) {
        (Role::Host, Message::CreateSession) => {
            if !limiter.check_and_record(peer_ip) {
                return HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Close(Some(Message::Error {
                        code: "rate_limited".into(),
                        message: "too many create/join attempts".into(),
                    }))],
                };
            }
            match registry.create_session(outbound.clone()).await {
                Ok((session_id, code, peer_id)) => HandleResult {
                    phase: ConnPhase::InSession {
                        session_id,
                        peer_id,
                        role: Role::Host,
                    },
                    outputs: vec![HandlerOutput::Reply(Message::SessionCreated {
                        session_id,
                        code,
                    })],
                },
                Err(e) => HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Close(Some(Message::Error {
                        code: "create_failed".into(),
                        message: format!("{e:?}"),
                    }))],
                },
            }
        }
        (Role::Guest, Message::JoinSession { code }) => {
            if !limiter.check_and_record(peer_ip) {
                return HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Close(Some(Message::Error {
                        code: "rate_limited".into(),
                        message: "too many create/join attempts".into(),
                    }))],
                };
            }
            match registry.join_session(&code, outbound.clone()).await {
                Ok((session_id, peer_id, host_id)) => {
                    let _ = registry
                        .send_to_peer(
                            session_id,
                            host_id,
                            &Message::PeerJoined {
                                peer_id,
                                role: Role::Guest,
                            },
                        )
                        .await;
                    HandleResult {
                        phase: ConnPhase::InSession {
                            session_id,
                            peer_id,
                            role: Role::Guest,
                        },
                        outputs: vec![HandlerOutput::Reply(Message::JoinAccepted {
                            session_id,
                            peer_id,
                            role: Role::Guest,
                        })],
                    }
                }
                Err(RegistryError::NotFound) => HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Reply(Message::JoinRejected {
                        reason: "session not found".into(),
                    })],
                },
                Err(RegistryError::SessionFull) => HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Reply(Message::JoinRejected {
                        reason: "session full".into(),
                    })],
                },
                Err(e) => HandleResult {
                    phase: ConnPhase::Negotiating { role },
                    outputs: vec![HandlerOutput::Close(Some(Message::Error {
                        code: "join_failed".into(),
                        message: format!("{e:?}"),
                    }))],
                },
            }
        }
        (role, other) => HandleResult {
            phase: ConnPhase::Negotiating { role },
            outputs: vec![HandlerOutput::Close(Some(Message::Error {
                code: "invalid_negotiate".into(),
                message: format!("unexpected message for {role:?}: {other:?}"),
            }))],
        },
    }
}

fn is_forwardable(message: &Message) -> bool {
    matches!(
        message,
        Message::CursorMove { .. }
            | Message::MouseMove { .. }
            | Message::MouseButton { .. }
            | Message::MouseScroll { .. }
            | Message::KeyEvent { .. }
            | Message::VideoFrame { .. }
            | Message::Heartbeat { .. }
    )
}

async fn handle_in_session(
    registry: &SessionRegistry,
    session_id: SessionId,
    peer_id: PeerId,
    role: Role,
    message: Message,
) -> HandleResult {
    match message {
        Message::LeaveSession => {
            registry.detach_peer(session_id, peer_id).await;
            HandleResult {
                phase: ConnPhase::ExpectHello,
                outputs: vec![HandlerOutput::Close(None)],
            }
        }
        ref msg if is_forwardable(msg) => {
            registry.touch(session_id).await;
            let _ = registry.forward_to_other(session_id, peer_id, msg).await;
            HandleResult {
                phase: ConnPhase::InSession {
                    session_id,
                    peer_id,
                    role,
                },
                outputs: vec![],
            }
        }
        other => HandleResult {
            phase: ConnPhase::InSession {
                session_id,
                peer_id,
                role,
            },
            outputs: vec![HandlerOutput::Reply(Message::Error {
                code: "unexpected".into(),
                message: format!("message not allowed in session: {other:?}"),
            })],
        },
    }
}
