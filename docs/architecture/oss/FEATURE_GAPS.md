# Feature Gaps: Reverse-Tether Desktop Apps

**Research date:** 2026-09-12  
**Lens:** User needs for a modern gnirehtet-based desktop GUI, mapped against real projects.  
**Legend:** ● = strong/covered · ◐ = partial · ○ = absent/weak · — = N/A

---

## 1. User-need clusters

Reverse tether users typically want:

1. **Get online fast** after plugging a phone (or headset).  
2. **Stay online** across unplug/replug, sleep, ADB blips.  
3. **Understand failures** without reading `adb` man pages.  
4. **Run on their OS** (Windows is the largest pain point; macOS/Linux matter for labs).  
5. **Trust the tool** (no root; clear VPN permission; no surprise network exfil).  
6. **Use real apps** (Play Store, banking, YouTube)—where VPN-based reverse tether often fails detection.

Gaps below are ordered by impact for a gnirehtet GUI successor.

---

## 2. Gap matrix (projects × capabilities)

| Need | gnirehtet CLI | Wirebound | YunuUSBNet | phone-inet-mgr | phone-net | RT-RP | AndroidBuddy | SimpleRT | re-Link* | Tetrd* |
|---|---|---|---|---|---|---|---|---|---|---|
| One-click start/stop | ◐ | ● | ● | ● | ● | ● | ● | ◐ | ● | ● |
| Device discovery UI | ○ | ● | ◐ | ◐ | ○ | ◐ | ● | ○ | ● | ● |
| Multi-device | ● `autorun` | ● | ◐ | ○ | ○ | ○ | ◐ | ● | ● | ● |
| USB support | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Wi‑Fi/TCP ADB path | ◐ possible | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ USB focus | ◐ |
| Windows | ● | ● | ● | ○ | ● | ● | ○ | ○ | ● | ● |
| macOS | ● (Rust lag) | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ● |
| Linux | ● | ○ | ○ | ● | ○ | ○ | ● | ● | ● | ● |
| Tray / status icon | ○ | ◐ | ◐ | ◐ | ○ | ○ | ○ | ○ | ● | ● |
| Auto-reconnect / hotplug | ◐ `autorun`/`tunnel` | ● | ● | ● | ○ | ◐ | ● udev | ◐ | ● | ◐ |
| DNS configuration UI | ◐ CLI | ● | ○ | ○ | ○ | ○ | ○ | ● CLI | ● | ◐ |
| Custom routes | ● `-r` | ◐ | ○ | ○ | ○ | ○ | ○ | ○ | ◐ | ○ |
| IPv6 | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ? | ? |
| Live throughput UI | ○ | ● | ● | ○ | ○ | ○ | ○ | ○ | ◐ | ◐ |
| Actionable diagnostics | ○ logs | ◐ logs | ◐ | ◐ | ◐ logs | ◐ | ◐ | ◐ | ● | ◐ |
| Bundled ADB + engine | ◐ releases | ● | ● | ● | ● auto-DL | ● | ◐ compile | build | ● | ● |
| Portable / no-admin install | ◐ | ◐ | ● claim | ○ systemd | ● | ● | ◐ | ○ root | ● | ◐ |
| No ADB required | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● | ◐ |
| No root (device) | ● | ● | ● | ● | ● | ● | ● | ● | ● | ● |
| Per-app VPN control | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ○ |
| App-compat guidance | ○ | ○ | ○ | ● README | ○ | ○ | ○ | ● README | ◐ | ◐ |
| Headless / lab automation | ● CLI | ○ | ○ | systemd | scripts | ○ | ◐ | CLI | ◐ | ○ |
| Security / trust UX | ○ | ○ | ○ | ○ | ◐ notes | ○ | ○ | ○ | ● marketing | ◐ |
| Cross-platform single binary story | ◐ | ○ | ○ | ○ | ○ | ○ | ○ | ○ | ● | ● |

\*Commercial — included as coverage benchmarks, not OSS peers.

---

## 3. Detailed gaps vs user needs

### 3.1 Discovery & connect

**Need:** See connected devices, serial/model, ADB auth state; one button to install APK + start VPN + open relay.

| Gap | Evidence | Who covers it |
|---|---|---|
| CLI users must know `run`/`install`/`start` | gnirehtet README | Wirebound, Easy Manager, phone-net scripts |
| First-run VPN permission popup is unexplained | Universal Android VpnService UX | Commercial apps explain better; OSS rarely coaches |
| Unauthorized ADB silent failure | Common issue threads | AndroidBuddy shows enable-debug instructions — **good pattern** |

**Opportunity:** Onboarding wizard: detect ADB → list devices → fix unauthorized → install client → request VPN → verify ICMP/HTTP probe.

### 3.2 Multi-device

**Need:** Lab of N phones; per-device toggle; shared or per-device relays.

| Gap | Notes |
|---|---|
| gnirehtet `autorun` exists but is invisible | No UI for “which devices are reverse-tethered” |
| Wrappers are mostly single-device mentally | Wirebound claims multi via autorun; RT-RP is headset-focused |
| SimpleRT multi-LAN is unique but stale | Interesting for device↔device file exchange over USB net |

**Opportunity:** Device table with status chips (ADB / APK / VPN / relay / last bytes).

### 3.3 Wi‑Fi vs USB transport

**Need:** Use reverse tether over TCP/IP ADB when cable is inconvenient (same LAN), or document that VPN-over-`adb reverse` works wirelessly if ADB is networked.

| Gap | Notes |
|---|---|
| Almost all GUIs assume USB only | scrcpy ecosystem has mature wireless pairing UX — reverse tether GUIs do not |
| Wireless ADB security risk underdocumented | Pairing codes, untrusted networks |

**Opportunity:** Optional “Wireless ADB” path with scary-but-clear warnings; reuse ScrcpyGUI pairing UI patterns.

### 3.4 OS parity (Windows / macOS / Linux)

| OS | Reality |
|---|---|
| **Windows** | Highest demand; most new wrappers (Wirebound, YunuUSBNet, phone-net, RT-RP). Driver/ADB hell remains. |
| **Linux** | Best for SimpleRT/AndroidBuddy/phone-internet-manager; packaging via apt/AppImage underused. |
| **macOS** | gnirehtet works; Rust release lagged (v2.2.1 in upstream notes); only stale Swift tray app (ryankwirth). **Largest OSS GUI hole.** |

**Opportunity:** Ship one codebase (Tauri/Qt) with signed macOS app + menu bar — white space vs all current OSS wrappers.

### 3.5 Permission UX

**Need:** Explain USB debugging, VPN consent, battery optimization exemptions, OEM quirks (Xiaomi, Samsung).

| Gap | Coverage |
|---|---|
| Battery killers stop VpnService | Mentioned in phone-internet-manager troubleshooting |
| OEM “restricted settings” blocking sideload | Rarely handled in GUIs |
| Host admin myths | gnirehtet issue #579 discussions confuse admin needs; clarify: gnirehtet does **not** need host root (unlike SimpleRT), but ADB/USB access matters |

### 3.6 Tray / status / notifications

**Need:** Know at a glance if reverse tether is up; pause from tray; OS notification on disconnect.

| Project | Status UX |
|---|---|
| ryankwirth | Menu bar list of active devices (stale but right idea) |
| Wirebound / Yunu | In-window status + charts |
| Commercial | Stronger always-visible status |

**Opportunity:** Cross-platform tray: green=connected, yellow=ADB only, red=down; click to reconnect.

### 3.7 Auto-reconnect

**Need:** Unplug/replug, cable shake, host sleep → restore tunnel without manual `gnirehtet tunnel`.

| Project | Approach |
|---|---|
| gnirehtet | `autorun` + manual `tunnel` |
| YunuUSBNet | PnP / WMI-driven reconnect (claimed differentiator) |
| phone-internet-manager | systemd + wrapper detect loop |
| AndroidBuddy | udev phone detection |

**Opportunity:** First-class reconnect state machine; expose metrics (reconnect count, last failure reason).

### 3.8 DNS

**Need:** Corporate DNS, DoH bypass, custom resolvers when host DNS is broken for the relayed path.

| Coverage | |
|---|---|
| gnirehtet | Limited; DNS goes through relayed UDP |
| Wirebound | Presets (Google/Cloudflare) + custom — **best OSS GUI DNS UX** |
| SimpleRT | `-n` nameserver / `local` |
| gnirehtetx | Custom DNS on client |
| re-Link | Custom DNS + proxy |

**Opportunity:** DNS settings panel + “test resolve” button; optional DNS override in enhanced APK.

### 3.9 IPv6

**Gap:** gnirehtet explicitly **does not support IPv6**. No surveyed OSS reverse-tether GUI adds it. Dual-stack hosts/networks can break assumptions.

**Opportunity:** Medium-term engine work (not GUI-only). Document IPv4-only clearly in UI. hev-socks5-tunnel shows dual-stack is feasible in tun2socks designs if architecture shifts.

### 3.10 Performance & throughput visibility

| Need | Coverage |
|---|---|
| Live speed | Wirebound charts; Yunu dual-unit speedometer |
| Latency/probe | Rarely implemented |
| CPU of relay | Rust better than Java; GUIs don’t expose |

**Opportunity:** Sparkline + optional detailed graph; “speed test” through tunnel; warn if Java relay selected.

### 3.11 Packaging

| Gap | Detail |
|---|---|
| Dependency hell | JRE, ADB, APK path, PATH issues — dominant Windows support burden |
| Bundling done well by | Wirebound releases, phone-net auto-download, RT-RP installer, scrcpy-style zips |
| Update channel | ScrcpyGUI auto-checks scrcpy releases — **reverse tether GUIs almost never auto-update the engine** |

**Opportunity:** Bundle platform-tools + gnirehtet; update checker for engine **and** GUI; portable zip + MSI/DMG/AppImage.

### 3.12 Security & trust

| Topic | Gap |
|---|---|
| ADB = full device control | GUIs under-explain blast radius |
| VPN API perception | Users fear “VPN = spyware”; re-Link addresses with policy language |
| Sideloaded APK trust | Prefer reproducible builds / signed APK matching upstream hashes |
| No-ADB desire | Only SimpleRT (stale) and commercials truly deliver |

### 3.13 No-root constraints & app compatibility

**Industry-wide gap:** Apps that check for “real” Wi‑Fi/cellular via `ConnectivityManager` fail on VpnService-only internet (Play Store, some social/video apps). Documented by SimpleRT, ReverseTethering NoRoot, phone-internet-manager.

| Mitigation ideas | Status in landscape |
|---|---|
| Leave Wi‑Fi/data on; VPN prioritized | Documented workaround |
| Per-app bypass/block | gnirehtetx only (client fork) |
| Always-on Wi‑Fi assist | Not productized in OSS GUIs |
| Honest compatibility list | Rare |

**Opportunity:** In-app compatibility advisor + link to known-broken apps; optional enhanced client with per-app rules.

### 3.14 Headless / automation / CI labs

CLI gnirehtet is fine for scripts; GUIs ignore `--json` / IPC. Labs need: list devices, start serial X, healthcheck, exit codes.

**Opportunity:** Keep a stable CLI/`gnirehtet`-compatible interface **and** GUI; add machine-readable status socket for orchestration.

---

## 4. Gap priority for a modern gnirehtet desktop GUI

| Priority | Gap | Rationale |
|---|---|---|
| P0 | Cross-platform packaging + bundled ADB/engine | Unlocks non-expert users; Windows+macOS hole |
| P0 | Connect wizard + failure diagnosis | #1 support cost |
| P0 | Hotplug auto-reconnect | Expected of any 2026 network tool |
| P1 | Tray status + multi-device table | Matches scrcpy/commercial UX bar |
| P1 | DNS UI + connectivity probe | Wirebound proves demand |
| P1 | App-compat education | Reduces 1-star “doesn’t work” reviews |
| P2 | Wireless ADB path | Parity with scrcpy GUIs |
| P2 | Engine update channel / Android 15+ stewardship | Survival if upstream stays cold |
| P3 | IPv6 / alternate SOCKS engine | Strategic, not day-one |
| P3 | ADB-free transport | Compete with re-Link long-term |

---

## 5. Mapping: which OSS projects teach which gap fixes

| Learn from | Steal (legally/ideas) |
|---|---|
| **Wirebound** | DNS presets, traffic charts, Electron IPC process management, autorun framing |
| **YunuUSBNet** | Aggressive reconnect, low-footprint packaging narrative |
| **phone-internet-manager** | systemd persistence; frank app-compat section |
| **phone-net** | Auto-download deps; clean teardown of VPN+adb |
| **ryankwirth** | macOS menu bar device list |
| **AndroidBuddy** | udev autostart; “enable USB debugging” instructional UX; suite concept |
| **RT-RP** | Domain-specific profiles (VR); installer bundling |
| **gnirehtetx** | Per-app block; custom DNS; stop-on-disconnect |
| **SimpleRT** | No-ADB ambition; multi-device virtual net; DNS `-n` |
| **scrcpy / QtScrcpy / ScrcpyGUI** | Wireless pairing, multi-device, auto binary updates, polish |
| **re-Link / Tetrd** | Benchmark for status UX and (re-Link) no-ADB — compete or roadmap |

---

## 6. Non-gaps (already “good enough” in baseline)

- Core PC→phone IPv4 TCP/UDP reverse tether without root (**gnirehtet**).  
- Multi-device capability at CLI level (`autorun`).  
- Dual relay implementations (Rust preferred, Java fallback).  
- Apache-2.0 licensing for commercial-friendly GUIs wrapping the engine.

