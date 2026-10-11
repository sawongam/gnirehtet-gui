# Roadmap

**Status:** Draft v0.1  
**Principle:** GUI and host UX first; networking core reused; rewrites only when evidence demands.

---

## Phase 0 — Evidence & spikes (1–2 weeks)

**Exit criteria:** TECH_DECISIONS gates for Tauri (or documented fallback) pass on ≥2 OS targets.

- [ ] Vendor or submodule upstream @ pinned revision; record hash in docs
- [ ] Spike A: Tauri 2 sidecar spawn/stop of `gnirehtet relay` + log stream
- [ ] Spike B: Orchestrator wraps `install` / `start` / `stop` / `tunnel` against a real device
- [ ] Spike C: Bundle APK as resource; verify version code `9` path
- [ ] Device matrix notes: Android versions available to maintainers
- [ ] Decision meeting: confirm or reject Tauri+Svelte provisional stack

**Deliverables:** spike notes, updated `TECH_DECISIONS.md` (provisional → accepted/rejected)

---

## Phase 1 — MVP desktop GUI (see `MVP_SPEC.md`)

**Exit criteria:** MVP acceptance tests green on Linux + Windows; macOS best-effort.

- [ ] `host-orchestrator` crate mirroring CLI semantics
- [ ] Svelte UI: devices, run/stop, logs, settings
- [ ] Packaging: app + sidecar + apk + NOTICE
- [ ] Contributor docs: build, run, attribution
- [ ] Public repo hygiene: issue templates, CODE_OF_CONDUCT, LICENSE

**Non-goals remain in force:** no relay rewrite, no client rewrite.

---

## Phase 2 — Reliability & multi-device

- [ ] Autorun / multi-device session model (upstream `autorun` / `autostart` behaviors)
- [ ] Stronger process supervisor (restart policies, port conflict recovery)
- [ ] Structured events (not only log scrape): relay up, client id assigned, adb lost
- [ ] macOS signing + notarization pipeline if macOS is first-class
- [ ] Automated integration tests with emulators where possible

---

## Phase 3 — Relay modernization (only with evidence)

Trigger examples: CVEs in frozen deps, inability to build on supported Rust toolchains, need for graceful in-process shutdown.

- [ ] ADR: mio upgrade vs tokio migration vs remain-sidecar-forever
- [ ] Protocol conformance test suite before/after
- [ ] Optional in-process `relaylib` with cooperative shutdown
- [ ] Keep Java relay out of default product path unless a platform requires it

---

## Phase 4 — Product depth (optional)

- [ ] Richer traffic/session stats (without breaking packet path)
- [ ] Wireless debugging guided setup
- [ ] IPv6 exploration (major protocol work — separate ADR)
- [ ] Optional minimal on-device status UI (still host-driven)

---

## Priority order (always)

1. Correctness of tether path  
2. Safe lifecycle (no orphan relays)  
3. Clear failure UX (adb/VPN/port)  
4. Cross-platform packaging  
5. Feature breadth  

Anything that threatens (1) or (2) for the sake of (5) is out of order.

---

## Coordination with other agents

| Agent topic | Architect expectation |
|-------------|----------------------|
| UI/UX research | Must not assume Android client redesign |
| Performance claims | Measure against stock gnirehtet CLI baseline |
| Stack proposals | Update `TECH_DECISIONS.md`; do not silently fork architecture |
| “Rewrite relay in X” | Requires Phase 3 trigger evidence + conformance plan |
