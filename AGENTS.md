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

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.
