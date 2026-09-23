//! Serveur relay Teleportal Remote.
//!
//! Sessions en mémoire, code 6 chiffres, WebSocket `/ws`, routage host↔guest.

#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod cleanup;
pub mod code;
pub mod config;
pub mod handler;
pub mod rate_limit;
pub mod session;
pub mod state;
pub mod ws;

pub use config::Config;
pub use state::SessionRegistry;
pub use ws::{app_router, serve, serve_ephemeral, AppState};
