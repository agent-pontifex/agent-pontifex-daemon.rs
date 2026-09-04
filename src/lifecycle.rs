//! Supervised process lifecycle: readiness, graceful drain, and forced stop.
//!
//! The shutdown contract is shared across the fleet (DEN-3175): a stop signal
//! moves the daemon out of readiness FIRST, so the supervisor and any load
//! balancer stop sending work, and only then drains in-flight work. Draining
//! before deregistering is the classic bug — new work keeps arriving into a
//! process that is trying to leave.

use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Process started, dependencies not yet proven.
    Starting,
    /// Dependencies proven; accepting work.
    Ready,
    /// Deregistered from work intake; finishing in-flight jobs.
    Draining,
    /// All work settled or grace expired.
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Signal {
    DependenciesReady,
    Terminate,
    DrainComplete,
    GraceExpired,
}

/// Why the daemon stopped. Distinguishing these is what makes an incident
/// readable: a clean drain and a timed-out drain look identical in a log that
/// only records "stopped".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Drained,
    GraceExpired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lifecycle {
    state: State,
    stop_reason: Option<StopReason>,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Lifecycle {
    pub const fn new() -> Self {
        Self {
            state: State::Starting,
            stop_reason: None,
        }
    }

    pub const fn state(&self) -> State {
        self.state
    }

    pub const fn stop_reason(&self) -> Option<StopReason> {
        self.stop_reason
    }

    /// Readiness as reported to systemd / the load balancer.
    /// Draining is deliberately NOT ready.
    pub const fn is_ready(&self) -> bool {
        matches!(self.state, State::Ready)
    }

    /// Apply a signal. Returns the new state.
    ///
    /// Transitions are total and explicit: every (state, signal) pair either
    /// advances or is ignored. Illegal transitions are impossible rather than
    /// merely unhandled — e.g. `DependenciesReady` arriving while draining must
    /// never pull the daemon back into `Ready`.
    pub fn apply(&mut self, signal: Signal) -> State {
        self.state = match (self.state, signal) {
            (State::Starting, Signal::DependenciesReady) => State::Ready,
            (State::Starting | State::Ready, Signal::Terminate) => State::Draining,
            (State::Draining, Signal::DrainComplete) => {
                self.stop_reason = Some(StopReason::Drained);
                State::Stopped
            }
            (State::Draining, Signal::GraceExpired) => {
                self.stop_reason = Some(StopReason::GraceExpired);
                State::Stopped
            }
            // Terminate while already draining is idempotent: a supervisor that
            // sends SIGTERM twice must not shorten the grace period.
            (current, _) => current,
        };
        self.state
    }
}

/// Heartbeat cadence that keeps a lease alive with margin for one lost renewal.
///
/// Renewing at exactly half the lease leaves no room for a single dropped
/// request; a third gives two chances before expiry.
pub fn heartbeat_for_lease(lease: Duration) -> Duration {
    lease / 3
}
