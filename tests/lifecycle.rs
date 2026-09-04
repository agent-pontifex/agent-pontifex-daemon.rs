use agent_pontifex_daemon::*;
use std::time::Duration;

fn cfg(hb: u64, lease: u64) -> DaemonConfig {
    DaemonConfig {
        supervisor: Supervisor::Foreground,
        coordinator_url: "https://coordinator.example".into(),
        worker_id: "worker-1".into(),
        heartbeat_interval: Duration::from_secs(hb),
        lease_duration: Duration::from_secs(lease),
        shutdown_grace: Duration::from_secs(30),
    }
}

#[test]
fn heartbeat_must_be_shorter_than_the_lease_it_renews() {
    cfg(40, 120)
        .validate()
        .expect("40s renew under a 120s lease is valid");
    // Equal is not enough: the renewal races the expiry.
    assert!(cfg(120, 120).validate().is_err());
    assert!(cfg(121, 120).validate().is_err());
}

#[test]
fn config_failures_are_startup_failures_not_defaults() {
    let mut c = cfg(40, 120);
    c.worker_id = "  ".into();
    assert!(c.validate().is_err());

    let mut c = cfg(40, 120);
    c.coordinator_url = "coordinator.example".into();
    assert!(c.validate().is_err(), "a relative URL must not be accepted");
}

#[test]
fn terminate_leaves_readiness_before_draining() {
    let mut l = Lifecycle::new();
    assert!(!l.is_ready());
    l.apply(Signal::DependenciesReady);
    assert!(l.is_ready());

    l.apply(Signal::Terminate);
    assert_eq!(l.state(), State::Draining);
    assert!(
        !l.is_ready(),
        "a draining daemon must not advertise readiness"
    );
}

#[test]
fn repeated_terminate_does_not_shorten_the_grace_period() {
    let mut l = Lifecycle::new();
    l.apply(Signal::DependenciesReady);
    l.apply(Signal::Terminate);
    l.apply(Signal::Terminate);
    assert_eq!(l.state(), State::Draining);
    assert_eq!(l.stop_reason(), None);
}

#[test]
fn readiness_cannot_be_regained_while_draining() {
    let mut l = Lifecycle::new();
    l.apply(Signal::DependenciesReady);
    l.apply(Signal::Terminate);
    l.apply(Signal::DependenciesReady);
    assert_eq!(l.state(), State::Draining);
    assert!(!l.is_ready());
}

#[test]
fn clean_drain_and_timeout_are_distinguishable() {
    let mut a = Lifecycle::new();
    a.apply(Signal::DependenciesReady);
    a.apply(Signal::Terminate);
    a.apply(Signal::DrainComplete);
    assert_eq!(a.stop_reason(), Some(StopReason::Drained));

    let mut b = Lifecycle::new();
    b.apply(Signal::DependenciesReady);
    b.apply(Signal::Terminate);
    b.apply(Signal::GraceExpired);
    assert_eq!(b.stop_reason(), Some(StopReason::GraceExpired));
}

#[test]
fn terminate_before_ready_still_drains() {
    let mut l = Lifecycle::new();
    l.apply(Signal::Terminate);
    assert_eq!(
        l.state(),
        State::Draining,
        "SIGTERM during startup must not be lost"
    );
}

#[test]
fn supervisor_detection_defaults_to_foreground() {
    assert_eq!(
        Supervisor::detect(None, None, false),
        Supervisor::Foreground
    );
    assert_eq!(
        Supervisor::detect(Some("abc"), None, false),
        Supervisor::Systemd
    );
    assert_eq!(
        Supervisor::detect(None, Some("/sock"), false),
        Supervisor::Launchd
    );
    assert_eq!(
        Supervisor::detect(Some("abc"), None, true),
        Supervisor::WindowsService
    );
    assert!(!Supervisor::Foreground.restarts_on_exit());
    assert!(Supervisor::Systemd.restarts_on_exit());
}

#[test]
fn heartbeat_leaves_room_for_one_lost_renewal() {
    let lease = Duration::from_secs(120);
    let hb = heartbeat_for_lease(lease);
    assert_eq!(hb, Duration::from_secs(40));
    assert!(hb * 2 < lease, "two renewals must fit inside one lease");
}
