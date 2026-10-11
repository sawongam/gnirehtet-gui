# Research notes (working log)

**Date:** 2026-09-12

## Method
- WebSearch + WebFetch of primary READMEs and product sites
- GitHub API for stars / pushed_at / archived (rate-limited mid-run; some later repos used search synthesis)
- Cross-checked commercial vs OSS and forward vs reverse tether

## Key factual anchors
- gnirehtet: ~7902★, pushed 2024-08-11, Apache-2.0, maintenance-mode README, v2.5.1, IPv4 only
- SimpleRT: ~927★, pushed 2022-12-20, GPL-3.0, AOA, no ADB, Linux/macOS
- google/vpn-reverse-tether: archived 2016, ~191★
- Wirebound: created 2026-05, Electron, Windows, Apache-2.0, ~3★
- scrcpy: ~149k★, very active — UX gold standard for Genymobile-style tools
- PdaNet+: forward tether only — excluded as reverse competitor

## Uncertainties
- Exact commercial internals (re-Link/Tetrd transport) — closed source; described from marketing only
- YunuUSBNet license (GitHub NOASSERTION)
- phone-internet-manager README claims “RNDIS + VPN” while wrapping gnirehtet — treat as imprecise
- IPv6 support claims for commercials — unverified
- GitHub API rate limit blocked some star counts (hev-socks5-tunnel, sockstun, wireguard-android, kil0bit scrcpy-gui) — used secondary sources / README presence

## Intentionally dropped
- One-off scripts/gists
- Abandoned scrcpy GUIs except as cautionary notes
- Forward-tether products as reverse competitors
