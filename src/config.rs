//! Runtime configuration for the daemon.
//!
//! The public contract lives in `.cli-flags.toml` at the repository root and is
//! parsed through `flags-2-env` at the argv boundary. This module holds only the
//! *resolved, immutable* configuration that is passed inward — it never reads
//! argv or the environment itself, and it never carries credentials. Secrets stay
//! in the environment/secret store; argv is visible in process listings.

use std::time::Duration;

/// Which service manager is supervising this process.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Supervisor {
    /// Linux systemd. Readiness and liveness are reported over `sd_notify`.
    Systemd,
    /// macOS launchd. KeepAlive is handled by the plist; exit code drives restart.
    Launchd,
    /// Windows Service Control Manager.
    WindowsService,
    /// No supervisor — foreground run, typically a developer shell.
    Foreground,
}

impl Supervisor {
    /// Detect the supervisor from the environment the process was started in.
    ///
    /// Detection is *observational only*: it reads well-known variables set by
    /// each manager and never mutates them. `Foreground` is the safe default —
    /// an unrecognised environment must not be treated as supervised, or the
    /// daemon would suppress logs nobody is collecting.
    pub fn detect(
        invocation_id: Option<&str>,
        launchd_socket: Option<&str>,
        windows_service: bool,
    ) -> Self {
        if windows_service {
            Self::WindowsService
        } else if invocation_id.is_some() {
            Self::Systemd
        } else if launchd_socket.is_some() {
            Self::Launchd
        } else {
            Self::Foreground
        }
    }

    /// Whether the supervisor restarts us, which decides if a fatal error should
    /// exit non-zero (let the supervisor restart) or loop internally.
    pub const fn restarts_on_exit(self) -> bool {
        matches!(self, Self::Systemd | Self::Launchd | Self::WindowsService)
    }
}

/// Resolved, immutable daemon configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DaemonConfig {
    pub supervisor: Supervisor,
    pub coordinator_url: String,
    pub worker_id: String,
    /// Lease renewal interval. Must be strictly less than `lease_duration`.
    pub heartbeat_interval: Duration,
    pub lease_duration: Duration,
    pub shutdown_grace: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ConfigError {}

impl DaemonConfig {
    /// Validate the invariants that make lease-based work safe.
    ///
    /// A parser/config failure is a **startup failure**. There is no silent
    /// fallback to defaults: a daemon that renews a lease too slowly will have
    /// its work stolen mid-flight while still believing it holds the lease.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.worker_id.trim().is_empty() {
            return Err(ConfigError("worker_id must not be empty".into()));
        }
        if !self.coordinator_url.starts_with("https://")
            && !self.coordinator_url.starts_with("http://")
        {
            return Err(ConfigError(
                "coordinator_url must be an absolute http(s) URL".into(),
            ));
        }
        if self.lease_duration.is_zero() {
            return Err(ConfigError("lease_duration must be non-zero".into()));
        }
        if self.heartbeat_interval >= self.lease_duration {
            return Err(ConfigError(
                "heartbeat_interval must be strictly less than lease_duration, \
                 otherwise the lease can expire before it is renewed"
                    .into(),
            ));
        }
        Ok(())
    }
}
