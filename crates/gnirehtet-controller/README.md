# gnirehtet-controller

Phase 2 orchestration library for Desktop / shared CLI policy.

- Depends on **`gnirehtet-adb` only** (no `gnirehtet-relay` / relaylib).
- Manages a **session-owned** stock `gnirehtet relay` child process.
- Default listen port **31416**; no mid-session listen-port hot-swap.
- Probes bind before claiming ownership (`PORT_IN_USE` on conflict).
- `Drop` / `clear_owned_relay` kills only a relay this session started.

See `docs/architecture/rust/CONTROLLER_API_DRAFT.md` and `REFACTOR_PLAN.md` Phase 2.
