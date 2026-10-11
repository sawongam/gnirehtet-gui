# NETWORK_ARCHITECTURE.md

**Role:** Source-backed map of gnirehtet networking for the desktop GUI successor.  
**Upstream pin:** `Genymobile/gnirehtet` @ `1eb2e58bc91e268dba62561d11e3bebd9b100883` (tag **v2.5.1**, 2023-07-09)  
**Primary sources:** upstream `README.md`, `DEVELOP.md`, Android client under `app/`, Rust relay under `relay-rust/` (preferred), Java relay under `relay-java/` (parity), [Android VpnService](https://developer.android.com/reference/android/net/VpnService), [RFC 793](https://tools.ietf.org/html/rfc793).  
**Rule:** Prefer **reuse → wrap → refactor → rewrite**. Do not rewrite the networking layer unless a failure mode or measured defect justifies it.

---

## 1. End-to-end picture

```text
Android apps / system
        │  IPv4 packets
        ▼
VpnService TUN (GnirehtetService)
  addr 10.0.0.2/32, MTU 0x4000, routes + DNS from VpnConfiguration
        │  FileDescriptor (blocking)
        ▼
Forwarder (2 threads)
  device→tunnel: read TUN, drop non-IPv4, send bytes to tunnel
  tunnel→device: receive stream, IPPacketOutputStream splits at packet boundaries, write TUN
        │
        ▼
PersistentRelayTunnel → RelayTunnel
  LocalSocket abstract name "gnirehtet"
        │  adb reverse localabstract:gnirehtet tcp:<port>
        ▼
Host relay (relay-rust preferred)
  TcpListener 127.0.0.1:31416 (DEFAULT_PORT)
  Client id u32 BE → then raw IPv4 stream
        │
        ▼
Router (5-tuple) → TcpConnection | UdpConnection
  L3↔L5 translation via host Berkeley sockets (NAT-like)
        │
        ▼
Host network / Internet
```

**Evidence:** `DEVELOP.md` Overview; `GnirehtetService.java`; `Forwarder.java`; `RelayTunnel.java`; `relay-rust/src/relay/tunnel_server.rs`; `cli_args.rs` `DEFAULT_PORT`.

---

## 2. Component responsibilities

| Component | Owns | Does not own |
|-----------|------|--------------|
| `GnirehtetService` | VpnService.Builder, TUN FD, MTU/routes/DNS, underlying network hint, forwarder lifecycle | Host adb / relay process |
| `Forwarder` | Bidirectional copy TUN ↔ tunnel; IPv4 filter; VPN read wake workaround | Protocol parsing beyond IP version |
| `RelayTunnel` / `PersistentRelayTunnel` | Abstract local socket, client-id handshake, reconnect loop | Host listening socket |
| Host CLI (`main.rs`) | `adb reverse`, install/start/stop intents, spawn relay, multi-device monitor | Packet processing |
| `TunnelServer` + `Client` | Accept clients, assign ids, stream reassembly of IPv4 packets | TUN / VpnService |
| `Router` + connections | 5-tuple demux, TCP state machine, UDP datagram relay, expiry cleanup | UI / adb |

---

## 3. Android VpnService setup (what is technically sound)

From `GnirehtetService.setupVpn`:

| Setting | Value | Source |
|---------|-------|--------|
| VPN address | `10.0.0.2/32` | `VPN_ADDRESS` |
| Default route | `0.0.0.0/0` if no custom routes | `builder.addRoute` |
| Custom routes | From `VpnConfiguration` / CLI `-r` | `CIDR[]` |
| Default DNS | `8.8.8.8` if none configured | comment + `addDnsServer` |
| Custom DNS | From CLI `-d` extras | `VpnConfiguration` |
| MTU | `0x4000` (16384) | comment: higher/lower poorer perf |
| Blocking mode | `builder.setBlocking(true)` | FileChannel not selectable |
| Service start | `START_NOT_STICKY`; O+ uses `startForegroundService` | `onStartCommand` |
| Manifest | `BIND_VPN_SERVICE` + `android.net.VpnService` filter | `AndroidManifest.xml` |

**Android platform constraints** ([VpnService](https://developer.android.com/reference/android/net/VpnService)):

- User must approve via `VpnService.prepare` (handled in `GnirehtetActivity`).
- Only one VPN at a time; another VPN deactivates this one.
- System shows ongoing notification; user can disconnect from system UI.
- Closing the FD restores network; `onRevoke()` is invoked when permission is revoked.
- Packets on the FD always start with IP headers.

**Judgment:** The VpnService usage is **technically sound** and aligned with the platform model. Blocking TUN I/O + two worker threads is a pragmatic choice given Android’s non-selectable FileChannel comment in source.

---

## 4. Packet flow (device → host → network)

1. App generates IPv4 traffic routed into the VPN interface.
2. `Forwarder.forwardDeviceToTunnel` blocking-reads the TUN FD (`BUFSIZE = 0x10000`).
3. If `version == 4`, bytes are sent on the current `RelayTunnel` (`LocalSocket` write of the raw packet). Non-IPv4 is logged and dropped (`Forwarder` + issue #69 reference).
4. Host `Client` accumulates TCP stream bytes in `Ipv4PacketBuffer` until a full packet (IPv4 total length) is available.
5. `Router.send_to_network` validates and demuxes by connection id (protocol + src/dst addr/port).
6. **UDP:** strip headers → write datagram on connected `UdpSocket`; reverse via `Packetizer`.
7. **TCP:** userspace TCB answers the device per TCP semantics while payload rides a host `TcpStream` to the real destination (`DEVELOP.md` + `tcp_connection.rs`).

**Reverse path:** network sockets → packetize IPv4 → client `StreamBuffer` → TCP to device → `IPPacketOutputStream` writes **one complete packet at a time** to TUN.

---

## 5. NAT-like behavior

`DEVELOP.md` states the relay behaves like a **port-restricted cone NAT**: it opens host sockets on behalf of private peers, but the “private side” uses the custom client↔relay stream instead of classic NAT rewrite on a shared L2/L3 link.

Implications:

- Source addresses seen by the Internet are the **host’s**, not `10.0.0.2`.
- Per-flow state lives in `Router`’s connection table keyed by the device’s 5-tuple.
- UDP flows expire after idle timeout; TCP follows explicit close / error paths.

This design is **sound** for reverse tethering and should **remain unchanged** for MVP.

**Address rewrite special case (verified):** destination **`10.0.2.2` is mapped to host loopback** (`LOCALHOST_FORWARD = 0x0a000202`) in Java `AbstractConnection` / Rust `connection.rs`. Emulator-style “gateway” traffic therefore hits the host, not a remote `10.0.2.2`.

**Not implemented on the data plane:** ICMP handling and IPv4 fragment reassembly were **not found** in the relay. Non-TCP/UDP (and fragmented payloads beyond treating total length as one buffer) should be treated as unsupported — packets are dropped at the router/forwarder filters.

---

## 6. Asynchronous I/O model (relay)

- **Rust:** single-threaded `mio` poll loop (`relay.rs` + `selector.rs`); channels registered with readiness handlers.
- **Java:** single `Selector` with `SelectionHandler` attachments (`Relay.java`).
- Monothreaded by design → no packet-level locks (`DEVELOP.md`).

**Outdated / fragile for a GUI embed:** `relaylib::relay(port)` is a **blocking** event loop with Ctrl+C-oriented lifetime; no graceful stop API in `lib.rs`. Desktop orchestrator should treat the relay as a **managed child process** (sidecar) until a clean shutdown API exists. That is orchestration, not a networking rewrite.

---

## 7. Multi-device behavior

| Layer | Behavior | Source |
|-------|----------|--------|
| Relay | Accepts **multiple** TCP clients; assigns monotonically increasing `u32` ids | `tunnel_server.rs` |
| Abstract socket name | Fixed `"gnirehtet"` on every device | `RelayTunnel.LOCAL_ABSTRACT_NAME` |
| `adb reverse` | Per-device (optionally `-s serial`) to same host port | `cmd_tunnel` |
| CLI | Multi-device needs `serial`; `autorun`/`AdbMonitor` starts per device | `main.rs` help text |

**What breaks / collides under multiple devices:**

- All devices reverse to the **same** host `127.0.0.1:port`. That works: each device’s reverse creates a distinct TCP connection to the shared relay (multi-client).
- Shared abstract name is fine because each device has its own namespace.
- Host process must keep reverse tunnels alive per serial; USB unplug drops that device’s reverse/client only.
- Concurrent DNS/route configs are per-device start intent extras — independent.

**Judgment:** Multi-device is supported at protocol/relay level. Fragility is in **host orchestration** (discover serials, re-apply reverse, track which client id maps to which device), not in the packet path.

---

## 8. What is sound vs fragile vs outdated

### Technically sound (safely remain)

- VpnService capture of IPv4 + raw packet tunnel.
- `adb reverse` to abstract local socket (avoids needing `VpnService.protect` for the tunnel; see `RelayTunnel.open` comment).
- Client-id handshake before considering the tunnel connected.
- Stream reassembly by IPv4 total length on both ends.
- Userspace TCP that **does not retransmit**; reliability by never reading host TCP data the client buffer cannot hold (`DEVELOP.md`).
- UDP datagram boundary preservation + idle expiry.
- Prefer Rust relay over Java (README).

### Fragile (keep, but wrap carefully)

- `DEVELOP.md` still mentions `AuthorizationActivity`, but that class is **absent** in v2.5.1; VPN prepare UX lives in `GnirehtetActivity` (stale doc vs tree).

- Client reconnect **does not** restore in-flight TCP/UDP session state on the relay (new client id, empty router). Apps see connection resets.
- Fixed 5s reconnect delay (`RelayTunnelProvider.DELAY_BETWEEN_ATTEMPTS_MS`).
- TUN blocking read does not wake on FD close → UDP wake workaround (`Forwarder.wakeUpReadWorkaround`).
- No custom `onRevoke()` — relies on default `stopSelf()`; `establish()` null-check covers prepare/revoke at start.
- `START_NOT_STICKY` — process kill does not auto-restart VPN.
- Large MTU `0x4000` couples client and relay (`tcp_connection.rs` duplicates constant).
- `setUnderlyingNetworks` hack for API ≥ 22 so apps see “network available”.
- Default public DNS `8.8.8.8` (privacy / captive portal / regional filtering concerns) — configuration exists (`-d`).

### Outdated (modernize only with justification)

- Upstream “not actively maintained” (README).
- mio **0.6**-era async style; Java 8 NIO relay kept as fallback.
- No IPv6 (explicit README).
- Foreground-service / always-on VPN metadata may need revisiting for newer Android targets (platform docs: temporary allowlist, promote to foreground; `SERVICE_META_DATA_SUPPORTS_ALWAYS_ON`). Validate against target SDK when forking APK — **not** a reason to rewrite L3/L5.

### Should be refactored (narrow, not a rewrite)

1. **Host orchestrator** around existing CLI semantics (tunnel/relay/start/stop) with health events.
2. Optional: expose relay **graceful shutdown** + structured metrics — library surface only.
3. Document/conformance-test the wire protocol (this doc set).
4. Revisit MTU with measurements if modern links show regressions.
5. Prefer configurable DNS in GUI rather than silent `8.8.8.8`.

### Would *not* rewrite without new evidence

- Userspace TCP/UDP relay core.
- Client↔relay raw IPv4 framing.
- Abstract socket + adb reverse path.

---

## 9. Socket lifecycle summary

| Socket | Created when | Destroyed when |
|--------|--------------|----------------|
| TUN FD | `Builder.establish()` | `close()` / revoke / crash |
| Abstract LocalSocket | `RelayTunnel.connect()` | invalidate / EOF / stop |
| Relay `TcpListener` | relay start | process exit |
| Per-client `TcpStream` | accept | client close / error |
| Per-flow TCP/UDP host sockets | first matching packet | TCP close path / UDP idle expiry / client drop |

---

## 10. Desktop GUI integration stance

Keep upstream APK + Rust relay binary as the networking path. GUI owns lifecycle, UX for VPN permission, adb health, and surfacing tunnel connected/disconnected (`RelayTunnelListener` → notification failure state). Packet processing stays in `relay-core`.

