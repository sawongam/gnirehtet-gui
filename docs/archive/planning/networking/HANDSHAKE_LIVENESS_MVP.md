# Handshake liveness — MVP probe (safe)

**Audience:** Desktop + Lead  
**Pin:** upstream gnirehtet v2.5.1 / PROTOCOL.md  
**Rule:** Do **not** invent wire messages or claim **Sharing**. Intent-sent ≠ handshake.

---

## 1. What handshake actually is

After `adb reverse localabstract:gnirehtet tcp:<port>` and VPN client connect:

1. Device opens abstract socket `gnirehtet`.
2. Host relay accepts TCP, assigns `u32` client id.
3. Relay writes **4 bytes BE** client id **before** packet relay.
4. Device `RelayTunnel.readClientId()` must succeed — only then logs  
   `Connected to the relay server as #<id>`.

`adb reverse` success alone can make `connect()` succeed while the relay is down; first read fails. That is why client-id is the liveness gate (`PROTOCOL.md` §2).

---

## 2. Safe MVP signals (no protocol change)

| Signal | Source | Means | Safe for UI? |
|--------|--------|-------|--------------|
| A. Relay log `Client #<n> connected` | Sidecar stdout / LogLine (`tunnel_server.rs`) | Host accepted TCP + started id write | **Yes** → Device VPN / Tunnel layer can leave *Pending* toward **Connected (host saw client)** |
| B. Relay log `Client #<n> disconnected` | Same | Client TCP gone | **Yes** → drop that layer; do not keep Sharing |
| C. Device logcat `Connected to the relay server as #` | `adb logcat -s RelayTunnel:D` (tag may vary) | Device finished `readClientId` | **Yes** → stronger VPN/tunnel liveness |
| D. `am start … START` exit 0 | adb | Intent delivered | **No** — keep **Pending** (current Phase 2 behavior is correct) |
| E. `adb reverse` OK | adb | Port redirect installed | **No** — not handshake |

**MVP recommendation:**

1. Keep today’s honesty: after start intent → **Device VPN = Pending**.
2. When LogLine matches `Client #(\d+) connected` (from owned relay only) → mark **tunnel/client liveness = OK** (chip copy: “Relay accepted client”, not Sharing).
3. Optional stretch (same MVP, no protocol change): one-shot or short `adb logcat` watch for RelayTunnel connected line → upgrade Pending → Connected on device side.
4. On `Client #<n> disconnected` or relay death (`RELAY_CRASHED`) → clear liveness; reconnect ≠ session resume (`RECONNECT_DESIGN.md`).

---

## 3. What not to do

- Do not add a new opcode / ping frame on the wire.
- Do not claim **Sharing** from A or C alone — Sharing = three-layer healthy (relay owned + reverse + VPN/handshake + traffic path per EVENT_STATUS_MAP).
- Do not treat reconnect as restoring TCP/UDP flows.
- Do not parse foreign (non-owned) listeners as our client.

---

## 4. Suggested Desktop wiring

- Regex on owned-relay LogLine: `Client #(\d+) connected` / `Client #(\d+) disconnected`.
- Map to existing ERROR_UX / status chips only (no new invent codes). Missing log → stay Pending.
- Lab later: reverse-without-relay must **not** flip Connected (handshake fail → `HANDSHAKE_FAILED` when you have signal; until then Pending/Error per ERROR_CODE_MAP G4).

---

## 5. Bottom line for Lead

**Safe MVP probe exists:** host LogLine client-connected (A), optional logcat (C).  
**If those pipes aren’t reliable yet:** stay **honest Pending** — that is correct, not a blocker to invent around.
