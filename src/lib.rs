//! Agent Pontifex cross-platform background daemon.
//!
//! Runs under systemd (Linux), launchd (macOS) and the Windows Service Control
//! Manager, and in the foreground for development. The daemon claims work from
//! an Agent Pontifex coordinator, renews the lease while working, and drains
//! cleanly on shutdown.
//!
//! Contract types come from `agent-pontifex-interfaces`; this crate holds no
//! duplicate contract definitions.

#![forbid(unsafe_code)]

pub mod config;
pub mod lifecycle;

pub use config::{ConfigError, DaemonConfig, Supervisor};
pub use lifecycle::{heartbeat_for_lease, Lifecycle, Signal, State, StopReason};
