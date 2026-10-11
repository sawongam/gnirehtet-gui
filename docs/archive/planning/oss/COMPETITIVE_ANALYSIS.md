# Competitive Analysis: Desktop GUI Successor for gnirehtet

**Research date:** 2026-09-12  
**Baseline:** [Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet) (Apache-2.0; ~7.9k★; last push 2024-08-11; **not actively maintained** beyond build blockers)  
**Scope:** Evidence-based positioning for a modern desktop GUI / front-end built around gnirehtet (or an equivalent reverse-tether engine).

---

## 1. Executive summary

Reverse tethering (PC → Android over USB) remains a **niche but sticky** need: developers, QA labs, VR headset users, repair desks, and environments where Wi‑Fi is unavailable, untrusted, or restricted. The technical baseline is mature—**VpnService + USB transport + host NAT/relay**—but the **product layer is fragmented and weak**.

Key findings:

1. **gnirehtet is still the de facto OSS engine**, but it is CLI-only, IPv4-only, and explicitly maintenance-mode. Users hit Android 15+ edge cases and packaging friction (esp. Windows + Rust/Java choice).
2. **There is no dominant modern OSS desktop GUI.** 2024–2026 produced several small wrappers (Wirebound, phone-internet-manager, YunuUSBNet, phone-net, RT-RP, Gnirehtet Easy Manager). Most are single-OS, low-star, early-stage, and wrap stock gnirehtet rather than extending the protocol.
3. **Commercial products own the “no ADB / polished UX” white space** (re-Link, Tetrd, ReverseTethering NoRoot). PdaNet+/EasyTether are **not** reverse-tether competitors—they primarily share phone→PC.
4. **SimpleRT** is the only notable OSS alternative architecture (AOA + TUN/NAT, no ADB), but it is stale (~2022 last push), Linux/macOS-only, and GPL-3.0.
5. **Opportunity:** a cross-platform, scrcpy-grade GUI that treats gnirehtet as the engine (with clear upgrade path for DNS/IPv6/reconnect/diagnostics), ships portable binaries, and matches commercial UX without requiring ADB-free magic on day one.

---

## 2. Competitor matrix

| Project | Type | GUI | Host OS | Reverse tether | Engine | Maint. (as of 2026-09) | Stars (approx.) | License |
|---|---|---|---|---|---|---|---|---|
| **gnirehtet** | Baseline CLI | No | Win/Linux/macOS | **Yes** (full, IPv4 TCP/UDP) | Own VPN+relay | Maintenance-only (push 2024-08) | ~7.9k | Apache-2.0 |
| **Wirebound** | OSS GUI wrapper | Yes (Electron) | Windows | **Yes** (via gnirehtet) | gnirehtet | Active early (created 2026-05) | ~3 | Apache-2.0 |
| **YunuUSBNet** | OSS GUI wrapper | Yes (AutoIt) | Windows | **Yes** (via gnirehtet) | gnirehtet | Active (push 2026-08) | ~1 | Unclear / NOASSERTION |
| **phone-internet-manager** | OSS GUI + systemd | Yes (Tkinter) | Linux (Debian-family) | **Yes** (via gnirehtet) | gnirehtet | Early (created 2026-02) | ~0 | MIT |
| **phone-net** | OSS scripts | Batch “one-click” | Windows | **Yes** (via gnirehtet) | gnirehtet | Early (2026-01) | ~0 | — |
| **Gnirehtet Easy Manager** | OSS GUI | Yes (WinForms) | Windows | **Yes** (via gnirehtet) | gnirehtet | Low activity (2025) | ~1 | — |
| **RT-RP** | Niche wrapper | Installer/scripts | Windows | **Yes** (via gnirehtet) | gnirehtet Java | Active niche VR (push 2026-07) | ~97 | GPL-3.0 |
| **ryankwirth/gnirehtet** | macOS menu-bar | Yes (tray) | macOS | **Yes** (via gnirehtet) | gnirehtet v2.4 | **Stale** (2020) | ~25 | Apache-2.0 |
| **AndroidBuddy** | Linux phone suite | Yes (yad) | Linux | **Partial** (launches gnirehtet) | gnirehtet | Active (2026-04) | ~26 | — |
| **SimpleRT** | Alt. OSS engine | No (service) | Linux/macOS | **Yes** (AOA, no ADB) | Own AOA+TUN | **Stale** (push 2022-12) | ~927 | GPL-3.0 |
| **google/vpn-reverse-tether** | Historical OSS | No | Linux | **Yes** (VpnService) | Own | **Archived** (2016) | ~191 | Apache-2.0 |
| **gnirehtetx** | Android APK fork | On-device UI extras | — | **Yes** (client fork) | gnirehtet-derived | Low (2025-03) | ~15 | Apache-2.0 |
| **re-Link** | Commercial | Yes | Win/macOS/Linux | **Yes** | Proprietary | Active product | n/a | Proprietary |
| **Tetrd** | Commercial | Yes (server+app) | Win/macOS/Linux | **Yes** (bidirectional claim) | Proprietary | Active (Play updates into 2026) | n/a | Proprietary |
| **ReverseTethering NoRoot** | Commercial | Yes (Java server) | Win/macOS/Linux | **Yes** | Proprietary | Still on Play | n/a | Proprietary |
| **PdaNet+ / EasyTether** | Commercial | Yes | Win/macOS | **No** (forward tether) | Proprietary | Active | n/a | Proprietary |

**Classification key**

- **Direct competitors (OSS):** projects whose primary purpose is reverse tethering with a user-facing desktop experience, or a viable alternate reverse-tether engine.
- **Adjacent:** phone management / mirroring tools that *include* reverse tether as one feature, or client-side forks.
- **Commercial competitors:** paid/no-ADB reverse tether products.
- **Non-competitors (exclude from deep dive):** PdaNet+/EasyTether (wrong direction); pure scrcpy GUIs (no networking); VPN clients (architecture only).

---

## 3. Deep dives — direct competitors only

### 3.1 Genymobile/gnirehtet (baseline engine, not a GUI)

- **URL:** https://github.com/Genymobile/gnirehtet  
- **Languages:** Java (relay + APK), Rust (preferred relay)  
- **Architecture:** Android `VpnService` captures device traffic → tunnel over `adb reverse` (localabstract ↔ TCP relay on host, default port 31416) → host relay performs TCP/UDP IPv4 forwarding/NAT. Commands: `run`, `autorun`, `relay`, `install`, `start/stop`, `tunnel`.  
- **Host OS:** Linux, Windows, macOS (macOS Rust build lagged at v2.2.1 in release notes).  
- **Android:** API 21+ (Android 5.0+).  
- **Reverse tether:** Yes — full for IPv4 TCP/UDP; **no IPv6**.  
- **GUI:** None (by design; “intended to be controlled from the computer only”).  
- **Status:** README states not actively maintained; last push ~2024-08-11; ~7.9k★; 300+ open issues; users still report Android 15 connectivity quirks (Rust vs Java).  
- **License:** Apache-2.0.  
- **Strengths:** Proven, cross-platform, no root, multi-device via `autorun`, Homebrew packaging, dual implementations.  
- **Weaknesses:** No GUI; IPv4-only; maintenance mode; opaque diagnostics; ConnectivityManager “fake Wi‑Fi” recognition issues for some apps (shared industry problem); Windows users struggle with JRE/path/ADB setup.  
- **Threat to a new GUI:** Low as a *product* competitor; high as a *dependency*—any GUI must plan for engine bit-rot or fork/maintain the relay.

### 3.2 SimpleRT (alternate OSS engine)

- **URL:** https://github.com/robinpaulson/SimpleRT (fork of original vvviperrr work; also historically iteratec releases / F-Droid `com.viper.simplert`)  
- **Languages:** C (desktop), Android/NDK client  
- **Architecture:** Android Open Accessory (AOA) USB protocol + host TUN + iptables NAT; **no ADB required**. Multi-device virtual network.  
- **Host OS:** Linux and macOS; Windows “being researched” (effectively unsupported).  
- **Android:** 4.0+.  
- **Reverse tether:** Yes.  
- **GUI:** None meaningful (standalone service).  
- **Status:** Last push ~2022-12-20; ~927★; stale.  
- **License:** GPL-3.0.  
- **Strengths:** No ADB (security/enterprise appeal); multi-device LAN; DNS selectable; F-Droid presence historically.  
- **Weaknesses:** Stale; host root/`sudo` required; no Windows; GPL complicates commercial reuse; ConnectivityManager recognition caveats same as peers.  
- **Competitive note:** Closest OSS *architectural* alternative to gnirehtet; not a GUI competitor.

### 3.3 Wirebound (most complete modern OSS GUI wrapper)

- **URL:** https://github.com/man612/wirebound  
- **Languages:** TypeScript / Electron + React + Vite  
- **Architecture:** Desktop GUI managing gnirehtet `autorun`, ADB lifecycle, DNS presets, traffic charts, logs. Bundles gnirehtet Rust + platform-tools in releases.  
- **Host OS:** Windows only.  
- **Android:** Via gnirehtet (5.0+).  
- **Reverse tether:** Yes (wrapper).  
- **GUI:** Yes — dashboard, themes, EN/ID localization.  
- **Status:** Created 2026-05; v1.0.1 early stable; ~3★; 10 open issues; maintainer focused on onboarding/packaging.  
- **License:** Apache-2.0.  
- **Strengths:** Closest to a “real product” OSS GUI: traffic monitor, DNS UI, auto-run, bundled binaries.  
- **Weaknesses:** Windows-only; Electron weight; tiny community; still depends on unmaintained gnirehtet; early reliability surface (USB drivers, ADB edge cases).  
- **Positioning:** Direct OSS product competitor for a Windows reverse-tether GUI. Beatable on cross-platform parity, lighter runtime (Tauri), diagnostics depth, and engine stewardship.

### 3.4 YunuUSBNet

- **URL:** https://github.com/YunuP-Dev/YunuUSBNet  
- **Languages:** AutoIt (GUI) + bundled gnirehtet/ADB  
- **Architecture:** Lightweight portable Windows GUI; WMI speedometer; PnP reconnect; “cable-shake” resilience claims.  
- **Host OS:** Windows 7–11 (32/64-bit claimed).  
- **Reverse tether:** Yes (gnirehtet).  
- **GUI:** Yes, ultra-light (~15MB portable claim).  
- **Status:** Created 2026-08; ~1★; active packaging focus.  
- **License:** Unclear (NOASSERTION on GitHub).  
- **Strengths:** Low-RAM niche; reconnect UX; portable.  
- **Weaknesses:** Tiny project; AutoIt stack limits cross-platform; license ambiguity; marketing-heavy README.  
- **Positioning:** Niche Windows competitor; validates demand for reconnect + live throughput UI.

### 3.5 phone-internet-manager

- **URL:** https://github.com/Imroj-Hassan/phone-internet-manager  
- **Languages:** Python/Tkinter + shell + systemd  
- **Architecture:** Wraps gnirehtet 2.5.1 Rust; systemd auto-start; reconnection wrapper; dark “terminal” GUI.  
- **Host OS:** Debian-family Linux.  
- **Reverse tether:** Yes.  
- **Status:** Created 2026-02; ~0★.  
- **License:** MIT.  
- **Strengths:** Boot persistence story; honest limitations section (some apps fail).  
- **Weaknesses:** Linux-only; zero community traction; claims “RNDIS + VPN” which mixes concepts (gnirehtet is VPN-over-adb, not classic RNDIS)—treat technical claims skeptically.  
- **Positioning:** Validates Linux demand for systemd + tray/status UX; not a strong competitor yet.

### 3.6 phone-net & Gnirehtet Easy Manager

- **phone-net:** https://github.com/MeirBen/phone-net — Windows batch start/stop, auto-download gnirehtet + platform-tools, clean shutdown. Useful packaging pattern; not a GUI product.  
- **Gnirehtet Easy Manager:** https://github.com/DannyJr97/Tethering-Reverse-Easy-Start-Progam — C# WinForms; install/start/stop/logs. Educational/personal scale (~1★).  

Both confirm the recurring pattern: **“gnirehtet is hard for non-CLI users → wrap it.”**

### 3.7 RT-RP (VR niche)

- **URL:** https://github.com/Kuijen/RT-RP  
- **Purpose:** Bundle gnirehtet Java + launcher for Quest/Android VR so Wi‑Fi-only apps (e.g. Virtual Desktop) work over USB.  
- **Status:** ~97★; active through 2026-07; GPL-3.0.  
- **Why it matters:** Highest-traction specialized wrapper; proves reverse tether demand outside “share office Ethernet.” Product opportunity: first-class headset profiles, not only phones.

### 3.8 Commercial: re-Link, Tetrd, ReverseTethering NoRoot

| Product | Differentiator vs gnirehtet GUI | Source |
|---|---|---|
| **[re-Link](https://re-link.io/)** | **No ADB / no root**; USB direct; auto-launch Android app on plug; DNS/proxy; free time-limited / $9.99 unlimited / Pro volume | Official site + Play (`com.curiouscompany.relink`) |
| **[Tetrd](https://tetrd.app/server)** | Universal tether (claims both directions); desktop servers for Win/macOS/Linux; Play app still updating into 2026 | tetrd.app + Play |
| **ReverseTethering NoRoot** | Java host server; multi-device; Android 4.0+; Free/Pro on Play | Play (`com.floriandraschbacher.reversetethering.*`) |

**Threat level:** High on UX and enterprise “no USB debugging” requirements. Low on open-source developer mindshare (gnirehtet still cited everywhere). A modern OSS GUI cannot match “no ADB” without a SimpleRT-like or proprietary AOA/USB path—plan that as a later differentiator or accept ADB as the Genymobile/scrcpy-compatible tradeoff.

### 3.9 Historical: google/vpn-reverse-tether

- **URL:** https://github.com/google/vpn-reverse-tether (archived)  
- **Last push:** 2016-11-20; ~191★; Apache-2.0.  
- **Relevance:** Architectural ancestor of VpnService reverse tether; Linux host forwarder + `adb forward`. Not a competitor; cite as prior art.

---

## 4. Positioning vs a gnirehtet desktop GUI opportunity

### What the market already has
- A solid **CLI engine** (gnirehtet) that “works enough.”  
- Many **thin wrappers** that solve install/start/stop for one OS.  
- Commercial apps that win on **ADB-free** and polished installers.

### What is missing (white space)
1. **Cross-platform first-class GUI** (Win/macOS/Linux) with scrcpy-level packaging quality.  
2. **Stewardship of the engine** (or a clean fork): Android 15+/16 compatibility, clearer reconnect (`tunnel`), DNS UX, optional route filters (gnirehtet already has `-r ROUTE`).  
3. **Diagnostics**: “why am I offline?” wizard (ADB unauthorized, VPN permission denied, relay down, DNS fail, ConnectivityManager app quirks).  
4. **Multi-device lab UX** (matrix of devices, per-device start/stop, shared relay).  
5. **Integration with the Genymobile ecosystem** (optional scrcpy alongside reverse tether—AndroidBuddy hints at this).  
6. **Honest app-compatibility guidance** (which apps break on VPN-based reverse tether).

### Recommended positioning statement
> “A modern, cross-platform desktop front-end for reverse tethering—starting with gnirehtet as the battle-tested engine—delivering scrcpy-grade UX: one-click connect, multi-device, tray status, auto-reconnect, DNS controls, and actionable diagnostics. Open source, no root, ADB-based (with a clear roadmap for optional ADB-free transport).”

---

## 5. Threats and white space

### Threats
| Threat | Severity | Mitigation |
|---|---|---|
| gnirehtet bit-rot / Android OS changes | High | Fork or vendor the APK+relay; CI against modern Android; watch issue #587-class regressions |
| Commercial no-ADB products (re-Link, Tetrd) | Medium–High for enterprise | Compete on OSS, price $0, developer workflows; consider AOA path later |
| Wirebound / other wrappers improve quickly | Low–Medium | Differentiate with cross-platform + engine quality, not just buttons |
| ConnectivityManager / Play Store app detection | Persistent industry issue | Document workarounds; explore “always-on Wi‑Fi + VPN priority” assists; per-app routing (see gnirehtetx) |
| Security concerns around ADB | Medium | Clear trust model; optional wireless ADB warnings; no surprise installs |

### White space (actionable)
1. **Cross-platform Tauri/Qt GUI** bundling ADB + gnirehtet (or successor).  
2. **Engine maintenance fork** with IPv6 exploration, better UDP/DNS, Android 15+ fixes.  
3. **Reconnect & hotplug** as a first-class feature (YunuUSBNet/phone-internet-manager prove demand).  
4. **VR/headset profiles** (RT-RP demand).  
5. **Lab / multi-device dashboard** with traffic per serial.  
6. **Optional future ADB-free transport** inspired by SimpleRT/re-Link (separate workstream).

---

## 6. Sources (primary)

- https://github.com/Genymobile/gnirehtet/blob/master/README.md  
- https://github.com/robinpaulson/SimpleRT/blob/master/README.md  
- https://github.com/man612/wirebound/blob/main/README.md  
- https://github.com/YunuP-Dev/YunuUSBNet  
- https://github.com/Imroj-Hassan/phone-internet-manager  
- https://github.com/MeirBen/phone-net  
- https://github.com/Kuijen/RT-RP  
- https://github.com/Botspot/androidbuddy  
- https://github.com/google/vpn-reverse-tether (archived)  
- https://re-link.io/  
- https://tetrd.app/server  
- https://play.google.com/store/apps/details?id=com.floriandraschbacher.reversetethering.free  
- https://pdanet.co/ (forward tether—non-competitor)

