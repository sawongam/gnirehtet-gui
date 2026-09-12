# RELEASE_PLAN.md

| Field | Value |
|-------|-------|
| Status | Draft v0.1 |
| Date | 2026-09-12 |
| Project | gnirehtet desktop GUI |
| Upstream | Genymobile/gnirehtet @ 1eb2e58 / v2.5.1, Apache-2.0 |
| Related | [TEST_STRATEGY.md](./TEST_STRATEGY.md), [TEST_MATRIX.md](./TEST_MATRIX.md), [FAILURE_SCENARIOS.md](./FAILURE_SCENARIOS.md), [CROSS_PLATFORM.md](./CROSS_PLATFORM.md) |

---

## 1. Versioning

| Component | Scheme | Notes |
|-----------|--------|-------|
| Desktop GUI / orchestrator | SemVer `MAJOR.MINOR.PATCH` | GUI product version is the **release version** |
| Bundled relay | Track upstream rust relay + local patch suffix if any | Record git pin `1eb2e58` until forked |
| Bundled APK | Upstream `v2.5.1` unless rebuilt | Mismatch tests (M-24) compare this pin |
| Tag | `vX.Y.Z` on product repo | Changelog must list upstream pin |

Pre-release: `vX.Y.Z-rc.N`. Nightlies: date stamp, not support-tagged.

**Compatibility rule:** MVP does not rewrite APK/relay; bumping either is a **release-blocking** compat retest (Android 10–16 sample + happy path USB).

---

## 2. Artifact set

| Artifact | Platform | Required for MVP GA? |
|----------|----------|----------------------|
| Linux package (AppImage and/or `.deb` — pick in packaging RFC) | Linux x64 (arm64 optional) | **Yes** (at least one format) |
| Windows installer (MSI or NSIS) + optional portable zip | Windows x64 | **Yes** |
| macOS `.dmg` / `.app` | macOS | Best-effort; label if unsigned |
| Bundled `gnirehtet` relay sidecar (per-OS) | all | **Yes** |
| Bundled APK (`GNIREHTET_APK` default) | all | **Yes** |
| Checksums (`SHA256SUMS`) | all | **Yes** |
| SBOM / dependency list (best-effort) | all | Recommended |
| LICENSE + NOTICE + upstream Apache-2.0 attribution | all | **Yes** |

**Not** in default MVP artifacts: full Android SDK / platform-tools. Optional “bundled adb” extra is a **packaging risk** — if added, version-pin, license-review, and TEST_MATRIX P-08.

Honor env: `ADB`, `GNIREHTET_APK`.

---

## 3. Release checklist

### 3.1 Preflight (eng + release)

- [ ] Version bump + changelog with upstream pin `1eb2e58` / APK `v2.5.1`
- [ ] LICENSE, NOTICE, third-party crate/npm licenses present
- [ ] No secrets in artifacts; no accidental SDK dump
- [ ] Relay + APK hashes recorded
- [ ] Default port 31416 documented; conflict UX implemented
- [ ] Logging fields + diagnostics pack implemented (TEST_STRATEGY §8)
- [ ] CI green: unit + mocked adb integration
- [ ] Signed Windows build **or** explicit SmartScreen known-limitation
- [ ] macOS: notarize **or** mark best-effort unsigned

### 3.2 Smoke (lab, P0 themes)

Use TEST_MATRIX IDs:

- [ ] **Happy path USB** Linux (M-01)
- [ ] **Happy path USB** Windows (M-02)
- [ ] **Port conflict** (M-17)
- [ ] **Relay killed externally** ≤3s (M-18)
- [ ] **Quit while running** — port free (M-23)
- [ ] **APK version mismatch** (M-24)
- [ ] **Multi-device require selection** (M-11)
- [ ] adb missing / unauthorized (M-06, M-10)
- [ ] VPN deny + revoke (M-15, M-16)
- [ ] PATH adb + `ADB` override (P-05, P-06)
- [ ] Fresh install Linux + Windows (P-01, P-02)
- [ ] Diagnostics pack contents complete

### 3.3 Sign-off

| Role | Sign |
|------|------|
| QA | P0 matrix + failure SLAs |
| Eng | Known bugs triaged; no Sev-1/2 open without mitigation |
| Release | Artifacts, checksums, signing, LICENSE |
| Product | Known-limitations published (macOS, no multi-tether, IPv4, no root) |

### 3.4 Soak

- [ ] ≥4h continuous tether on Linux **or** Windows (prefer both for GA)
- [ ] Mid-soak: unplug/replug once; relay kill once; confirm recovery SLAs
- [ ] No silent session death; memory leak “no unbounded growth” eyeball + RSS note

---

## 4. Rollback

| Situation | Action |
|-----------|--------|
| Bad APK/relay pin | Re-release with last known hashes; document `GNIREHTET_APK` override |
| GUI crash on start | Yank tag from download page; keep git tag; publish hotfix |
| Signing/notarization fail | Hold channel; do not ship unsigned as GA (lab OK) |
| Critical tether blackhole | Stop distribution; known-limitation + hotfix; users Stop + uninstall APK if needed |

Rollback artifact: previous `vX.Y.Z` installer + SHA256. State whether settings format changed (breaking upgrades need migration note).

---

## 5. Known-limitations template

Publish with every release:

```
## Known limitations — vX.Y.Z
- Platforms: Linux + Windows supported; macOS best-effort ([status]).
- One active tether session; multiple devices require selection (no simultaneous multi-tether).
- USB debugging primary; wireless adb not a supported primary path.
- IPv4 only; no root; VPN permission is an on-device user grant.
- adb is user-provided (PATH or configured). Full SDK is not vendored.
- Default relay port 31416; conflicts must be resolved by user/settings.
- OEM Android VPN/USB quirks: [list].
- Signing: Windows [signed|SmartScreen]; macOS [notarized|unsigned lab].
- Upstream: Genymobile/gnirehtet v2.5.1 @ 1eb2e58 (Apache-2.0).
```

---

## 6. Attribution / LICENSE checks (gate)

- [ ] Apache-2.0 of upstream reproduced
- [ ] NOTICE lists Genymobile gnirehtet + rust/JS deps as required
- [ ] GUI license compatible (do not relicense upstream APK/relay more restrictively)
- [ ] No GPL platform-tools accidentally bundled without review
- [ ] About screen shows versions + licenses link

---

## 7. Security / signing gates

| Gate | MVP GA |
|------|--------|
| Dependency audit (cargo/npm high CVEs) | Block or waive with ticket |
| Windows Authenticode | Required for “supported” Windows **or** limitation banner |
| macOS notarization + staple | Required only if claiming macOS supported; else best-effort |
| Linux: no suid; no root requirement | Required |
| Sidecar only spawned from app-controlled path | Required |
| adb never auto-downloaded from random URL | Required |
| Code signing identity documented in release notes | Required |

Permissions: app must not require host root. Linux udev is user/setup, not the binary.

---

## 8. Recovery & diagnostics (release quality bar)

Ship notes must tell users how to export diagnostics pack:

- logs (`timestamp`, `serial`, `adb_exit_code`, `port`, `stage`, `error_code`)
- `adb devices -l`, version strings, OS, port listen state

Support will reject tickets without pack when issue is device-specific.

SLAs to advertise internally (not necessarily user-facing copy): relay death UI ≤3s; device loss ≤5s.

---

## 9. Test / failure / matrix pointers

- Strategy, levels, CI vs lab: TEST_STRATEGY.md  
- What must be green: TEST_MATRIX.md P0  
- How failures recover: FAILURE_SCENARIOS.md  
- OS-specific ship notes: CROSS_PLATFORM.md  

---

## 10. Post-MVP release expansions

When enabling simultaneous multi-device tether or wireless-adb-primary: new SemVer **MINOR** at least; expand matrix X-01/X-02; extra soak; do not slip into patch.

---

## 11. Release day runbook (short)

1. Freeze pin; build signed artifacts; write SHA256SUMS  
2. Run §3.2 smoke on RC bits (not dev tree)  
3. Attach limitations + LICENSE  
4. Tag + publish  
5. Monitor first 48h issues for RELAY_DEAD / PORT_IN_USE / ADB_MISSING clusters  

