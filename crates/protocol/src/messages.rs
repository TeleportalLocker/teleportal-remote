//! Catalogue des messages du protocole Teleportal.

use serde::{Deserialize, Serialize};

use crate::ids::{CursorId, PeerId, SessionCode, SessionId};
use crate::role::Role;

/// Version du protocole négociée dans [`Message::Hello`].
pub const PROTOCOL_VERSION: u16 = 1;

/// Codec vidéo supporté (MVP : H264 uniquement à la validation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    /// H.264 / AVC (MVP).
    H264,
    /// H.265 / HEVC (prévu, rejeté en validation MVP).
    H265,
}

/// Bouton de souris.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    /// Clic gauche.
    Left,
    /// Clic droit.
    Right,
    /// Clic milieu.
    Middle,
}

/// Modificateurs clavier (bitflags simples).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    /// Shift.
    pub shift: bool,
    /// Control / Ctrl.
    pub ctrl: bool,
    /// Alt / Option.
    pub alt: bool,
    /// Meta / Command / Win.
    pub meta: bool,
}

/// Message protocolaire (signaling, curseur, input, vidéo).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Handshake initial.
    Hello {
        /// Doit égaler [`PROTOCOL_VERSION`].
        protocol_version: u16,
        /// Rôle annoncé par le pair.
        role: Role,
    },
    /// Demande de création de session (hôte).
    CreateSession,
    /// Session créée avec code à 6 chiffres.
    SessionCreated {
        /// Identifiant de session.
        session_id: SessionId,
        /// Code de connexion.
        code: SessionCode,
    },
    /// Demande de jonction via code.
    JoinSession {
        /// Code à 6 chiffres.
        code: SessionCode,
    },
    /// Jonction acceptée.
    JoinAccepted {
        /// Session rejointe.
        session_id: SessionId,
        /// Identifiant attribué au pair.
        peer_id: PeerId,
        /// Rôle effectif.
        role: Role,
    },
    /// Jonction refusée.
    JoinRejected {
        /// Motif lisible.
        reason: String,
    },
    /// Départ volontaire de la session.
    LeaveSession,
    /// Un pair a rejoint.
    PeerJoined {
        /// Pair arrivé.
        peer_id: PeerId,
        /// Rôle du pair.
        role: Role,
    },
    /// Un pair a quitté.
    PeerLeft {
        /// Pair parti.
        peer_id: PeerId,
    },
    /// Keep-alive.
    Heartbeat {
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Erreur protocolaire applicative.
    Error {
        /// Code machine.
        code: String,
        /// Message humain.
        message: String,
    },
    /// Session expirée côté relay.
    SessionExpired,
    /// Position du curseur collaboratif (coords normalisées 0.0–1.0).
    CursorMove {
        /// Curseur concerné.
        cursor_id: CursorId,
        /// Abscisse normalisée.
        x: f32,
        /// Ordonnée normalisée.
        y: f32,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Déplacement souris (contrôle distant).
    MouseMove {
        /// Abscisse normalisée 0.0–1.0.
        x: f32,
        /// Ordonnée normalisée 0.0–1.0.
        y: f32,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Bouton souris.
    MouseButton {
        /// Bouton.
        button: MouseButton,
        /// `true` = pressé, `false` = relâché.
        pressed: bool,
        /// Abscisse normalisée.
        x: f32,
        /// Ordonnée normalisée.
        y: f32,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Molette / scroll.
    MouseScroll {
        /// Delta horizontal.
        dx: f32,
        /// Delta vertical.
        dy: f32,
        /// Abscisse normalisée.
        x: f32,
        /// Ordonnée normalisée.
        y: f32,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Touche clavier.
    KeyEvent {
        /// Identifiant de touche (nom logique ou code).
        key: String,
        /// `true` = pressé, `false` = relâché.
        pressed: bool,
        /// Modificateurs.
        modifiers: KeyModifiers,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
    },
    /// Trame vidéo encodée.
    VideoFrame {
        /// Identifiant monotone de frame.
        frame_id: u64,
        /// Horodatage Unix ms.
        timestamp_ms: u64,
        /// Largeur en pixels.
        width: u32,
        /// Hauteur en pixels.
        height: u32,
        /// Codec.
        codec: VideoCodec,
        /// Payload encodé.
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}
