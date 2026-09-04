# agent-pontifex-daemon.rs

Cross-platform Agent Pontifex background daemon. Runs under **systemd** (Linux),
**launchd** (macOS) and the **Windows Service Control Manager**, and in the foreground for
development.

The daemon claims work from an Agent Pontifex coordinator, renews the lease while working,
and drains cleanly on shutdown.

## Shutdown contract

Per the fleet contract (DEN-3175), `SIGTERM` **leaves readiness first**, then drains:

```
Starting ──DependenciesReady──▶ Ready ──Terminate──▶ Draining ──▶ Stopped
                                                        │
                                     DrainComplete ─────┴───── GraceExpired
```

Deregistering before draining is the whole point. Draining first means new work keeps
arriving into a process that is trying to leave.

`Stopped` records **why**: `Drained` (clean) or `GraceExpired` (timed out). A log that only
says "stopped" cannot tell those apart, and they mean very different things during an incident.

Two properties the tests pin down:

- **Readiness cannot be regained while draining.** A late `DependenciesReady` must not pull
  the daemon back into rotation.
- **Repeated `SIGTERM` is idempotent.** A supervisor that signals twice must not shorten the
  grace period.

## Lease renewal

`heartbeat_interval` **must be strictly less than** `lease_duration`, and this is enforced at
startup — a config failure is a startup failure, never a silent default. A daemon that renews
too slowly has its work stolen mid-flight while still believing it holds the lease.

The default is a **third** of the lease. Renewing at exactly half leaves no room for a single
dropped request; a third gives two chances before expiry.

## Service manager timeouts

Every manager can kill an active drain. Each of these must exceed `shutdown-grace-seconds`:

| Manager | Setting | Shipped value |
|---|---|---|
| systemd | `TimeoutStopSec` | 45 s |
| launchd | `ExitTimeOut` | 45 s |
| Windows SCM | `WaitToKillServiceTimeout` | registry, default 30 s — **raise it** |

## Configuration

The public contract is `.cli-flags.toml`, parsed by
[flags-2-env](https://github.com/flags-2-env/flags-2-env) at the argv boundary. That file is
the single authority for commands, aliases, types, defaults, env keys and precedence.

**No credentials are accepted as flags.** argv is visible in process listings and shell
history; secrets come from the environment or secret store only.

## Dependencies

Deliberately none at this layer. Lifecycle and configuration are pure `std` so they can be
property-tested and formally reviewed without a runtime. Transport, serde and the async
runtime belong in the adapter layer, resolved through zed-pkg. Contract types come from
`agent-pontifex-interfaces` — this crate defines no duplicate contract types.

## Verify

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
