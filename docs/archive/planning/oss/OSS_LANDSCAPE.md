# OSS Landscape: Android Reverse Tethering & Adjacent Tooling

**Research date:** 2026-09-12  
**Purpose:** Catalog technically relevant open-source (and key commercial) projects for architects building a gnirehtet-based desktop GUI.

---

## 1. Taxonomy of the space

```
Reverse connectivity (PC ↔ Android)
├── Reverse tethering (PC → phone)          ← THIS SPACE
│   ├── VpnService + ADB reverse/relay      (gnirehtet family)
│   ├── VpnService + host forwarder         (google/vpn-reverse-tether)
│   ├── AOA USB + TUN/NAT                   (SimpleRT)
│   ├── RNDIS/USB gadget + root/NAT         (AOSP reverse_tether.sh; rooted)
│   └── Commercial USB stacks               (re-Link, Tetrd, …)
├── Forward tethering (phone → PC)          (PdaNet+, USB tethering, FoxFi)
├── Proxy/SOCKS workflows                   (adb reverse + SOCKS; Termux)
└── Full VPN overlays                       (WireGuard, OpenVPN — architecture refs)

Desktop Android control
├── Screen/audio mirror                     (scrcpy, sndcpy, QtScrcpy, ScrcpyGUI)
└── Device utilities                        (AndroidBuddy, ADB AppControl)

Tunnel primitives
├── tun2socks family                        (hev-socks5-tunnel, go-tun2socks, badvpn)
└── transparent proxy                       (redsocks — host-side, rarely Android-native)
```

**How to read this catalog:**  
- **Direct reverse tether** = primary purpose is PC→Android internet.  
- **Wrappers** = UX over gnirehtet (not new engines).  
- **Primitives** = reusable networking building blocks.  
- **Inspiration** = UX/architecture only.  
- Trivial one-off scripts are omitted or noted as packaging patterns only.

---

## 2. Reverse tether — engines & clients

### 2.1 Genymobile/gnirehtet (baseline)

| Field | Detail |
|---|---|
| **Name** | gnirehtet |
| **URL** | https://github.com/Genymobile/gnirehtet |
| **Languages** | Java, Rust |
| **Architecture** | Android VpnService ↔ `adb reverse` abstract socket ↔ host TCP relay (port 31416) performing IPv4 TCP/UDP NAT |
| **Host OS** | GNU/Linux, Windows, macOS |
| **Android** | API 21+ |
| **Reverse tether** | **Yes** (IPv4 TCP/UDP; no IPv6) |
| **GUI** | No |
| **Maintenance** | Not actively maintained (README); last push ~2024-08-11; ~7,902★; v2.5.1 |
| **License** | Apache-2.0 |
| **Strengths** | Cross-platform; no root; `autorun`; dual Java/Rust; Homebrew |
| **Weaknesses** | CLI-only; maintenance mode; IPv4-only; setup friction on Windows |
| **Approach** | VPN tunnel over USB ADB reverse |

### 2.2 SimpleRT

| Field | Detail |
|---|---|
| **Name** | SimpleRT |
| **URL** | https://github.com/robinpaulson/SimpleRT |
| **Languages** | C (host), Java/NDK (Android) |
| **Architecture** | AOA accessory protocol + host TUN + NAT; multi-device virtual network |
| **Host OS** | Linux, macOS (Windows unsupported) |
| **Android** | 4.0+ |
| **Reverse tether** | **Yes** |
| **GUI** | No (service only) |
| **Maintenance** | Stale (last push ~2022-12-20); ~927★ |
| **License** | GPL-3.0 |
| **Strengths** | No ADB; multi-tether LAN; custom DNS |
| **Weaknesses** | Needs host root; no Windows; stale; GPL |
| **Approach** | USB accessory + kernel TUN |

### 2.3 google/vpn-reverse-tether (historical)

| Field | Detail |
|---|---|
| **Name** | vpn-reverse-tether |
| **URL** | https://github.com/google/vpn-reverse-tether |
| **Languages** | C (JNI/host), Java |
| **Architecture** | VpnService + unix/TCP forwarder + `adb forward` + host iptables |
| **Host OS** | Linux (documented) |
| **Android** | VpnService era |
| **Reverse tether** | **Yes** (historical) |
| **GUI** | No |
| **Maintenance** | **Archived**; last push 2016-11-20; ~191★ |
| **License** | Apache-2.0 |
| **Notes** | Prior art for gnirehtet-style design; do not ship |

### 2.4 Linus789/gnirehtetx (Android client fork)

| Field | Detail |
|---|---|
| **Name** | GnirehtetX |
| **URL** | https://github.com/Linus789/gnirehtetx |
| **Languages** | Java, Kotlin |
| **Architecture** | Stock gnirehtet APK + extras |
| **Reverse tether** | **Yes** (client-side) |
| **GUI** | On-device settings extras |
| **Maintenance** | Low; push ~2025-03; ~15★ |
| **License** | Apache-2.0 |
| **Extras** | Block internet per app; custom DNS; stop on disconnect |
| **Reuse** | Per-app allow/deny and DNS UX ideas for an enhanced client |

### 2.5 AOSP reverse_tether.sh (rooted USB gadget)

- **URL:** https://android.googlesource.com/platform/development/+/master/scripts/reverse_tether.sh (historical paths exist)  
- **Approach:** `svc usb setFunction rndis`, configure `rndis0`/`usb0`, enable IP forward + MASQUERADE.  
- **Reverse tether:** Yes, but **requires root / engineering builds** typically.  
- **Relevance:** Not a product competitor; documents USB gadget alternative when root is available.

---

## 3. Reverse tether — desktop wrappers (gnirehtet front-ends)

### 3.1 man612/wirebound

| Field | Detail |
|---|---|
| **URL** | https://github.com/man612/wirebound |
| **Languages** | TypeScript, Electron, React |
| **Architecture** | GUI process manager for gnirehtet `autorun` + ADB; bundled binaries |
| **Host OS** | Windows |
| **Reverse tether** | **Yes** (via gnirehtet) |
| **GUI** | Yes — dashboard, traffic charts, DNS presets, logs, dark/light |
| **Maintenance** | Early active (created 2026-05); ~3★ |
| **License** | Apache-2.0 |
| **Strengths** | Feature-complete wrapper vision |
| **Weaknesses** | Windows-only; Electron weight; tiny user base |

### 3.2 YunuP-Dev/YunuUSBNet

| Field | Detail |
|---|---|
| **URL** | https://github.com/YunuP-Dev/YunuUSBNet |
| **Languages** | AutoIt + bundled tools |
| **Host OS** | Windows 7–11 |
| **Reverse tether** | **Yes** |
| **GUI** | Yes — speedometer, PnP reconnect |
| **Maintenance** | Active packaging (2026-08); ~1★ |
| **License** | Unclear |
| **Notes** | Targets 2GB RAM PCs; portable folder |

### 3.3 Imroj-Hassan/phone-internet-manager

| Field | Detail |
|---|---|
| **URL** | https://github.com/Imroj-Hassan/phone-internet-manager |
| **Languages** | Python, Tkinter, systemd units |
| **Host OS** | Debian-family Linux |
| **Reverse tether** | **Yes** |
| **GUI** | Yes |
| **Maintenance** | Early (2026-02); ~0★ |
| **License** | MIT |
| **Notes** | Auto-start/reconnect emphasis; some README protocol claims are imprecise |

### 3.4 MeirBen/phone-net

| Field | Detail |
|---|---|
| **URL** | https://github.com/MeirBen/phone-net |
| **Languages** | Batchfile |
| **Host OS** | Windows |
| **Reverse tether** | **Yes** |
| **GUI** | No (double-click scripts) |
| **Maintenance** | 2026-01; ~0★ |
| **Strengths** | Auto-download deps; clean stop (VPN + adb kill) |
| **Classification** | Packaging pattern, not a product competitor |

### 3.5 DannyJr97/Tethering-Reverse-Easy-Start-Progam

| Field | Detail |
|---|---|
| **URL** | https://github.com/DannyJr97/Tethering-Reverse-Easy-Start-Progam |
| **Languages** | C# WinForms (.NET Framework 4.8) |
| **Host OS** | Windows |
| **Reverse tether** | **Yes** |
| **GUI** | Yes (basic) |
| **Maintenance** | 2025; ~1★ |
| **Notes** | AdbService / GnirehtetService separation is a clean educational layout |

### 3.6 ryankwirth/gnirehtet (macOS menu bar)

| Field | Detail |
|---|---|
| **URL** | https://github.com/ryankwirth/gnirehtet |
| **Languages** | Swift |
| **Host OS** | macOS |
| **Reverse tether** | **Yes** (bundles gnirehtet v2.4) |
| **GUI** | Menu bar extra |
| **Maintenance** | **Stale** (2020); ~25★ |
| **License** | Apache-2.0 |
| **Reuse** | Tray/menu-bar UX pattern for macOS |

### 3.7 Kuijen/RT-RP

| Field | Detail |
|---|---|
| **URL** | https://github.com/Kuijen/RT-RP |
| **Languages** | Batch / NSIS installer |
| **Host OS** | Windows |
| **Reverse tether** | **Yes** (gnirehtet Java for VR headsets) |
| **GUI** | Installer + scripted launch |
| **Maintenance** | Active niche (2026-07); ~97★ |
| **License** | GPL-3.0 |
| **Notes** | Highest-star specialized wrapper; Quest Dev Mode caveats |

### 3.8 Botspot/androidbuddy (adjacent suite with reverse tether)

| Field | Detail |
|---|---|
| **URL** | https://github.com/Botspot/androidbuddy |
| **Languages** | Shell + yad |
| **Host OS** | Debian-family Linux (Pi-Apps) |
| **Reverse tether** | **Partial** — button compiles/runs gnirehtet |
| **GUI** | Yes (simple button panel) |
| **Maintenance** | Active (2026-04); ~26★ |
| **Also includes** | File transfer, scrcpy, forward tether, udev autostart |
| **Classification** | Adjacent phone-manager; validates “suite” packaging |

---

## 4. VPN / tunnel primitives (reusable, not reverse-tether products)

### 4.1 heiher/hev-socks5-tunnel

| Field | Detail |
|---|---|
| **URL** | https://github.com/heiher/hev-socks5-tunnel |
| **Languages** | C |
| **Architecture** | High-performance tun2socks (TCP+UDP, IPv4/IPv6, dual stack) |
| **Platforms** | Linux/Android/FreeBSD/macOS/iOS/Windows |
| **Reverse tether** | **No** (primitive); used *inside* VPN apps |
| **License** | MIT (per project docs) |
| **Android use** | NDK build; JNI `TProxyService`; used by SocksTun, Orbot, etc. |
| **Relevance** | If replacing gnirehtet relay with SOCKS-based design: run SOCKS on host, `adb reverse`, tunnel via hev on device |

### 4.2 heiher/sockstun

| Field | Detail |
|---|---|
| **URL** | https://github.com/heiher/sockstun |
| **Architecture** | Android VpnService app wrapping hev-socks5-tunnel |
| **Reverse tether** | **No** alone; **partial** if SOCKS is on PC via `adb reverse` |
| **Relevance** | Reference for VpnService + tun2socks integration UX |

### 4.3 go-tun2socks / badvpn / redsocks

| Project | URL | Role | Reverse tether |
|---|---|---|---|
| **badvpn** (tun2socks) | https://github.com/ambrop72/badvpn | Classic TUN→SOCKS; historical Android use via shadowsocks forks | No (primitive) |
| **go-tun2socks** (ecosystem) | various (e.g. dosgo/go-tun2socks) | Go tun2socks; Android needs `VpnService.protect` | No (primitive) |
| **redsocks** | various packages | Host transparent TCP→proxy redirector | Host-side only |

**Termux / SOCKS substitute workflow (common DIY, not a product):**  
Host SOCKS (`ssh -D` or similar) → `adb reverse tcp:1080 tcp:1080` → Android proxy 127.0.0.1:1080 or SocksTun/hev. **Partial** reverse tether (proxy-aware apps only unless VpnService used).

---

## 5. ADB / desktop Android tools

| Project | URL | Reverse tether | Notes |
|---|---|---|---|
| **ADB AppControl** | https://adbappcontrol.com/en/pc/ | **No** | Device app management + scrcpy; UX reference for ADB desktop apps |
| **AndroidBuddy** | https://github.com/Botspot/androidbuddy | Partial | See §3.8 |
| **platform-tools (adb)** | Google | Enables | `adb reverse` / `forward` are the transport substrate |

---

## 6. scrcpy ecosystem (inspiration / adjacent)

| Project | URL | Stars (approx.) | Status | Reverse tether | Why relevant |
|---|---|---|---|---|---|
| **scrcpy** | https://github.com/Genymobile/scrcpy | ~149k | Very active (push 2026-09) | No | Same org/author lineage as gnirehtet; gold-standard ADB desktop UX; audio now built-in |
| **sndcpy** | https://github.com/rom1v/sndcpy | ~3.5k | Alive (2026-07) | No | Historical audio sidecar; packaging pattern for companion tools |
| **QtScrcpy** | https://github.com/barry-ran/QtScrcpy | ~32k | Active | No | Qt GUI; multi-device; wireless ADB flow; group control |
| **ScrcpyGUI (kil0bit)** | https://github.com/kil0bit-kb/scrcpy-gui | (active Tauri v2) | Active 2026 | No | Tauri+React modern GUI; auto binary updates; wireless pairing UI |
| **guiscrcpy / older GUIs** | various | — | Mostly abandoned | No | Cautionary: GUI wrappers die when engine moves on |
| **Vysor / Mirror** | commercial | — | Commercial | No | Polished desktop Android control UX (closed) |

**Classification:** Inspiration-only for UX, packaging, multi-device, wireless ADB onboarding—not reverse-tether competitors.

---

## 7. Commercial reverse tether (for landscape completeness)

| Product | URL | ADB required? | Host OS | Notes |
|---|---|---|---|---|
| **re-Link** | https://re-link.io/ | **No** | Win/macOS/Linux | Strongest “enterprise USB” story; free limited / paid unlimited |
| **Tetrd** | https://tetrd.app/ | Typically yes on Windows (USB debug) | Win/macOS/Linux | Universal tether claims; active Play updates |
| **ReverseTethering NoRoot** | Play Store | Uses Java PC server | Win/macOS/Linux | Long-standing niche app |
| **PdaNet+ / EasyTether** | https://pdanet.co/ etc. | n/a | Win/macOS | **Forward** tether — not reverse competitors |

---

## 8. Architecture / VPN client references (not competitors)

| Project | URL | Reuse |
|---|---|---|
| **WireGuard Android** | https://github.com/WireGuard/wireguard-android | VpnService lifecycle, `protect()`, foreground service, CONNECTING vs connected states, Go userspace backend |
| **OpenVPN for Android** | ics-openvpn ecosystem | Permission UX, always-on VPN, per-app VPN |

---

## 9. Dependency / reuse map

```
                    ┌─────────────────────────┐
                    │  Desired desktop GUI    │
                    │  (Tauri / Qt / Electron)│
                    └───────────┬─────────────┘
                                │ orchestrates
          ┌─────────────────────┼─────────────────────┐
          ▼                     ▼                     ▼
   platform-tools          gnirehtet relay      gnirehtet.apk
        (adb)              (Rust preferred)     (VpnService)
          │                     │                     │
          │   adb reverse       │   TCP :31416        │
          └──────────►──────────┴──────────◄──────────┘

Optional future paths:
  • Client fork features ← gnirehtetx (DNS, per-app, stop-on-disconnect)
  • ADB-free transport   ← SimpleRT AOA ideas / commercial re-Link class
  • SOCKS redesign       ← hev-socks5-tunnel + SocksTun patterns
  • Suite bundling       ← scrcpy + reverse tether (AndroidBuddy pattern)
```

**License watch:** gnirehtet Apache-2.0 is GUI-friendly. SimpleRT GPL-3.0 contaminates if code is linked/copied. Commercial protocols are closed.

---

## 10. Dropped / trivial (intentionally not cataloged as competitors)

- One-off Gists and “reverse tether” shell pastes without maintenance.  
- Generic “share Wi‑Fi hotspot” guides.  
- PdaNet+/EasyTether as reverse-tether peers (wrong traffic direction).  
- Abandoned scrcpy GUIs except as cautionary notes.

