# agent-pontifex-daemon.rs — agent instructions

Follow the org policy in `agent-pontifex/.github` and `agent-pontifex/ARCHITECTURE.md`.
Repository-specific rules:

1. **Do not add contract types here.** They live in `agent-pontifex-interfaces`. A duplicate
   contract home is the DEN-3048 defect.
2. **Keep `src/lifecycle.rs` and `src/config.rs` dependency-free.** Their value is that they
   are pure `std` and can be reasoned about without a runtime. Adapters may take deps.
3. **Never accept a credential as a CLI flag.** argv is visible in process listings.
   `.cli-flags.toml` is the only option schema; do not add a second parser.
4. **Any change to the shutdown state machine needs a test that pins the invariant**, not
   just one that exercises the happy path. The existing tests cover: readiness dropped
   before drain, idempotent terminate, no readiness regain while draining, and clean-vs-
   timeout stop reasons.
5. If `shutdown-grace-seconds` changes, update `TimeoutStopSec`, `ExitTimeOut` and the
   Windows SCM note together. They are one constraint expressed in three places.
