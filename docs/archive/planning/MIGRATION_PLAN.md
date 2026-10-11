# Migration Plan (Reuse → Wrap → Refactor → Rewrite)

**Status:** Draft v0.1  
**Upstream pin:** `Genymobile/gnirehtet` @ `1eb2e58` / v2.5.1  
**Policy:** Prevent unnecessary rewrites. Every move up the rewrite ladder needs a written reason.

---

## 1. Inventory & disposition

| Upstream asset | Disposition | Rationale |
|----------------|-------------|-----------|
| `app/` Android VPN client | **Reuse** (ship APK; fork only if Android forces it) | Hardest correctness surface; already intent-controllable |
| `relay-rust/` networking | **Reuse** as MVP engine (binary/sidecar) | Upstream-recommended; contains TCP semantics |
| `relay-rust` CLI (`main.rs`, `adb_monitor.rs`) | **Wrap**, then **extract** shared lib | Encodes known-good adb flows |
| `relay-java/` | **Keep available**, not default | Fallback only; JRE cost |
| Root Gradle multi-project | **Do not require** for desktop-first repo | Desktop build should not depend on full Android Studio unless building APK |
| Shell helpers / `gnirehtet-run.cmd` | **Replace** with GUI (+ optional thin CLI) | UX successor |
| Docs (`README`, `DEVELOP`) | **Cite + supersede** with our docs | Preserve protocol truth from DEVELOP.md |
| License headers / NOTICE | **Retain** | Apache-2.0 obligation |

---

## 2. Phased migration

### Stage M0 — Consume without modifying

- Pin upstream commit/tag.
- CI downloads or builds release APK + rust binary **or** builds from submodule.
- GUI shells out to identical command semantics as upstream CLI.

**Done when:** GUI Run/Stop matches CLI `run` behavior on a test device.

### Stage M1 — Vendor + thin adapters

- Vendor `relay-rust` and `app` (or git submodule) into successor repo.
- Add adapter layer (`host-orchestrator`) that is the **only** place that knows adb CLI details.
- UI never constructs raw adb commands ad hoc.

**Done when:** no UI code string-builds `adb reverse` except via orchestrator tests.

### Stage M2 — Extract libraries (refactor, not rewrite)

- Split “relay event loop” vs “adb orchestration” vs “CLI argv parser”.
- Publish internal crates: `gnirehtet-relay`, `gnirehtet-adb`, `gnirehtet-cli`.
- GUI and CLI both call the same libraries.

**Done when:** deleting GUI still leaves a working CLI with shared code.

### Stage M3 — Targeted fixes

Allowed examples:

- Android manifest / foreground service updates for new API levels
- Port conflict diagnostics
- Cooperative relay shutdown API
- Dependency bumps with conformance tests

Disallowed without new ADR:

- Replacing userspace TCP implementation wholesale
- Replacing VpnService with another capture mechanism
- Changing client id / packet framing unilaterally

### Stage M4 — Rewrite (last resort)

Only if Stage M3 cannot meet a hard requirement (build break on all supported toolchains, unpatchable security issue, protocol extension that cannot dual-run).

Rewrite checklist (all required):

1. Written failure analysis of reuse path  
2. Protocol conformance suite green on old and new  
3. Side-by-side traffic test plan (TCP+UDP, lossy link optional)  
4. Rollback: ship previous relay binary for one major release  

---

## 3. Compatibility strategy

### Wire compatibility

- MVP **must** speak the existing client↔relay protocol (client id `u32` + raw IPv4 packets).
- Any protocol change requires version negotiation or coordinated APK+relay release.

### APK compatibility

- Honor upstream version code checks or replace with an explicit compatibility table in orchestrator.
- Bundle matching APK with each app release; document mismatch errors.

### CLI compatibility (nice-to-have)

- Preserve subcommand names where practical for muscle memory (`relay`, `start`, `stop`, …).
- GUI can be primary; CLI remains for automation.

---

## 4. What we deliberately remove

| Item | When | Why |
|------|------|-----|
| Java relay from default downloads | MVP packaging | Upstream already recommends Rust; smaller support matrix |
| Assumption that users run terminal `gnirehtet run` | MVP UX | Product is the GUI; CLI becomes optional |
| Dead Gradle tasks for desktop contributors | After APK build path is documented | Reduce onboarding confusion |

Removal ≠ deletion from history: Java relay can remain in vendor tree or docs as fallback.

---

## 5. Risk controls

- **Golden log tests:** record adb command sequences from upstream CLI; orchestrator must match.
- **Packet fixtures:** reuse/port Java/Rust unit tests for headers/packetizer where feasible.
- **Manual device checklist** per release (VPN prompt, DNS override, reconnect after unplug + `tunnel`).

---

## 6. Immediate next actions

1. Confirm product name / repo location with Sangam.  
2. Run Phase 0 spikes (`ROADMAP.md`).  
3. Freeze MVP scope to `MVP_SPEC.md` unless new evidence forces a cut.  
4. Other agents: submit findings as facts/assumptions/risks against these docs; architect merges via ADRs.
