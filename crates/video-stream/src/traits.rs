//! Traits communs d’encodage / décodage vidéo.

use teleportal_capture::Frame;

use crate::error::{DecodeError, EncodeError};
use crate::types::{DecodeConfig, DecodedFrame, EncodeConfig, EncodedFrame};

/// Encodeur de frames écran → bitstream codec.
///
/// Abstraction prête pour des backends hardware (Media Foundation /
/// VideoToolbox) en remplacement de OpenH264.
pub trait VideoEncoder: Send {
    /// Démarre l’encodeur selon `config`.
    ///
    /// # Errors
    ///
    /// Config invalide ou échec d’initialisation native.
    fn start(config: EncodeConfig) -> Result<Self, EncodeError>
    where
        Self: Sized;

    /// Encode une frame BGRA8.
    ///
    /// Retourne `Ok(None)` si l’encodeur a sauté la frame (rate control).
    ///
    /// # Errors
    ///
    /// Frame invalide ou échec natif.
    fn encode(&mut self, frame: &Frame) -> Result<Option<EncodedFrame>, EncodeError>;

    /// Demande une keyframe (IDR) pour la prochaine frame.
    fn force_keyframe(&mut self);

    /// Arrête l’encodeur et libère les ressources.
    ///
    /// # Errors
    ///
    /// Échec de libération native (rare).
    fn stop(self) -> Result<(), EncodeError>
    where
        Self: Sized;
}

/// Décodeur bitstream → RGBA8.
///
/// Abstraction prête pour backends hardware futurs.
pub trait VideoDecoder: Send {
    /// Démarre le décodeur selon `config`.
    ///
    /// # Errors
    ///
    /// Config invalide ou échec d’initialisation native.
    fn start(config: DecodeConfig) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Décode un payload Annex-B (une ou plusieurs NAL).
    ///
    /// Retourne `Ok(None)` si davantage de données sont nécessaires (ex. avant IDR).
    ///
    /// # Errors
    ///
    /// Bitstream corrompu ou échec natif.
    fn decode(
        &mut self,
        frame_id: u64,
        timestamp_ms: u64,
        annex_b: &[u8],
    ) -> Result<Option<DecodedFrame>, DecodeError>;

    /// Arrête le décodeur.
    ///
    /// # Errors
    ///
    /// Échec de libération native (rare).
    fn stop(self) -> Result<(), DecodeError>
    where
        Self: Sized;
}
