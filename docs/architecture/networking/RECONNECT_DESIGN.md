# RECONNECT_DESIGN.md

**Upstream pin:** `1eb2e58` / v2.5.1  
**Sources:** `PersistentRelayTunnel.java`, `RelayTunnelProvider.java`, `RelayTunnel.java`, `Forwarder.java`, `GnirehtetService` connection-state handler, `DEVELOP.md`, host `cmd_tunnel` / `AdbMonitor`.

---

## 1. Goals (upstream)

`PersistentRelayTunnel` javadoc: expose a `Tunnel` that **automatically handles `RelayTunnel` reconnections** so the VPN can keep running while the relay is stopped/started.

Non-goals in upstream:

- Resume TCP/UDP flow state across reconnect.
- Re-establish `adb reverse` from the device (device cannot; host must).
- Survive VPN revoke without user re-approval.

---

## 2. Device-side reconnect algorithm (as implemented)

```text
send/receive loop:
  while not stopped:
    tunnel = provider.getCurrentTunnel()  # may connect
    try I/O
    on IOException / EOF:
      provider.invalidateTunnel(tunnel)
      retry
```

### `RelayTunnelProvider` details

| Item | Behavior | Code |
|------|----------|------|
| First attempt | No delay | `waitUntilNextAttemptSlot` + `first` |
| Later attempts | Wait until `lastFailureTimestamp + 5000ms` | `DELAY_BETWEEN_ATTEMPTS_MS = 5000` |
| Success | `notifyRelayTunnelConnected` | listener → clear failure UI |
| Failure / invalidate | Close socket, `tunnel = null`, `notifyRelayTunnelDisconnected` | failure UI |
| Mutexes | `getCurrentTunnelLock` prevents concurrent connect; `this` protects shared fields | comments in provider |

### Handshake gate

`RelayTunnel.connect()`:

1. `localSocket.connect(abstract "gnirehtet")`
2. `readClientId` (blocking `readInt`)

Only after (2) is the tunnel considered good. This is the core **liveness** check vs reverse-without-relay.

---

## 3. What reconnect restores vs loses

| Restored automatically (if reverse + relay healthy) | Lost |
|-----------------------------------------------------|------|
| Ability to send/receive new IPv4 packets | Prior relay `Client` id |
| VPN interface itself (still up) | All TCP TCBs / UDP associations on relay |
| | Mid-flight TCP streams (apps must reconnect) |
| | Any packets in dead socket buffers |

**Design implication for GUI copy:** “Reconnecting…” means **control tunnel**, not seamless TCP continuity.

---

## 4. Host-side responsibilities (not in APK)

Device reconnect **cannot** fix:

1. Missing `adb reverse` after USB/adb restart.  
2. Relay not bound to port.  
3. Wrong serial when multiple devices present.

Upstream host mitigations:

- `gnirehtet tunnel` — re-apply reverse only (README/`main.rs`: often sufficient if VPN still active).
- `gnirehtet start` — tunnel + start intent.
- `autorun` / `AdbMonitor` — react to device list changes.

**Desktop reconnect policy (recommended, wraps upstream):**

```text
on_device_online(serial):
  ensure_relay(port)
  adb_reverse(serial, port)
  # optional: start intent if VPN not running

on_tunnel_disconnected_while_device_present:
  reassert_reverse(serial)
  ensure_relay_alive()
  # device APK retries on its own

on_usb_gone(serial):
  mark offline; do not kill relay if other clients exist
```

Do **not** change the 5s client backoff unless telemetry shows harm; if changing, keep ≥ handshake RTT and avoid tight loops that spam adb.

---

## 5. Interaction with Forwarder threads

- Two threads block in `send`/`receive` on `PersistentRelayTunnel`.
- Invalidating the tunnel unblocks them via socket shutdown/close (`RelayTunnel.close` shuts input/output).
- Stopping VPN: `forwarder.stop()` sets stopped, invalidates, cancels futures, and sends dummy UDP to wake TUN read (`wakeUpReadWorkaround`).

Reconnect design depends on this dual-thread blocking model; replacing it is a large change — **not required for GUI MVP**.

---

## 6. State signaling

`RelayTunnelListener` messages:

- `MSG_RELAY_TUNNEL_CONNECTED` → `notifier.setFailure(false)`
- `MSG_RELAY_TUNNEL_DISCONNECTED` → `notifier.setFailure(true)`

Obsolete events ignored if VPN already stopped (`RelayTunnelConnectionStateHandler`).

Desktop should mirror: Connected / Reconnecting / Failed, driven by host probes + optional future logcat hooks — without APK changes for MVP.

---

## 7. Judgment

| Piece | Verdict |
|-------|---------|
| Device persistent tunnel + client-id gate | **Sound — keep** |
| 5s backoff | Acceptable; tune later with data |
| No session resume | Inherent to architecture; document, don’t fake |
| Host reverse re-assert | **Required** for real-world reconnect; implement in orchestrator |
| Rewriting networking for “better reconnect” | **Not justified** unless session continuity becomes a hard product requirement (would imply protocol extension or userspace TCP migrate — high risk) |

