# FAILURE_MODES.md

**Upstream pin:** `1eb2e58` / v2.5.1  
**Sources:** client Java sources, Rust relay, `DEVELOP.md`, [VpnService](https://developer.android.com/reference/android/net/VpnService) (`prepare`, `establish`, `onRevoke`).

Legend: **Observed in code** vs **Inferred from platform/docs** (labeled).

---

## 1. Matrix

| Failure | Immediate effect | Who notices | Recovery today |
|---------|------------------|-------------|----------------|
| USB disconnect | adb session / reverse dies; LocalSocket EOF/errors | Device tunnel threads; host adb | Client `PersistentRelayTunnel` retries every 5s; host must re-`adb reverse` when device returns |
| `adb` daemon dies | reverse + device control fail | Host CLI/orchestrator | Restart adb; re-tunnel; client keeps retrying |
| Relay process dies | accept/read fails; connect may still “succeed” via reverse | Client id read / later I/O | Client reconnects; needs relay restarted + reverse still present |
| Relay not listening but reverse exists | `connect` OK, first read fails | `readClientId` | Treated as failure; retry after delay |
| VPN permission revoked | TUN deactivated; `onRevoke` → default `stopSelf` | System + service | User must re-approve `prepare`; host must `start` again |
| Another VPN starts | This VPN interface deactivated (platform) | System | Same as revoke/stop path |
| `establish()` returns null | VPN not started | Log “VPN starting failed” | Retry start after prepare |
| Invalid / non-IPv4 packet | Dropped | Logs | None needed |
| Client buffer full (host→device) | Drop / WouldBlock; TCP stops reading network | Relay warn logs | Backpressure via interests |
| UDP idle | Connection removed after 2 min idle | Silent | New packet creates new UDP association |
| Process kill (device) | `START_NOT_STICKY` → no auto restart | User | Host `start` again |

---

## 2. USB disconnect

**Observed:** Tunnel uses adb reverse. When USB (or wireless adb) drops, the abstract socket connection breaks → `receive`/`send` throw or EOF (`PersistentRelayTunnel` invalidates tunnel).

**Host:** Reverse bindings for that serial disappear with the adb transport. Relay may still run; that device’s `Client` is closed and removed (`tunnel_server.remove_client`), clearing its router connections.

**App traffic:** Existing TCP flows die (RST/timeouts). This is expected: relay state is per client TCP session, not durable across disconnect.

**GUI requirement:** Detect adb device loss, show tether offline, on device reappear run `tunnel` + ensure relay up + optionally auto `start` (mirrors upstream `autorun` / `AdbMonitor` intent).

---

## 3. adb dies or is replaced

**Observed:** All control-plane operations (`reverse`, `install`, `am start`) go through `exec_adb` (`main.rs`). Failure surfaces as command execution errors.

**Inferred:** A dead adb server breaks reverse and device-side socket. Client retry loop alone cannot restore reverse.

**GUI requirement:** Health-check adb (`adb devices` / start-server), then re-apply reverse before expecting tunnel CONNECTED.

---

## 4. Relay dies

**Observed:**

- Device may still connect to abstract socket if reverse remains.
- Handshake read fails or later EOF → `invalidateTunnel` → notification failure state (`RelayTunnelListener` → `Notifier.setFailure(true)`).
- All host-side TCP/UDP sockets for that client are cleared on client close.

**Recovery:** Restart relay on same port; client reconnects within retry policy; **in-flight flows are not resumed**.

---

## 5. Android VPN permission revoked

**Platform** ([VpnService.onRevoke](https://developer.android.com/reference/android/net/VpnService#onRevoke())): interface already deactivated; app should close FD and shut down; default implementation calls `stopSelf()`.

**Observed in gnirehtet:**

- **No override** of `onRevoke()` in `GnirehtetService` → default `stopSelf()`.
- `Builder.establish()` documents null if not prepared or revoked (`setupVpn`).
- Explicit `CLOSE_VPN` path closes forwarder + FD.

**Gap:** Relying on default revoke behavior is acceptable but coarse. Forwarder stop / wake workaround may race with system FD teardown. GUI should treat revoke like hard stop and require a fresh START + possible `prepare` UI.

---

## 6. Partial failures inside a healthy session

| Mode | Behavior | Source |
|------|----------|--------|
| Device→network TCP packet drop when buffers stressed | Safe: device retransmits | `DEVELOP.md` |
| Network→device drop after read | **Unsafe** — avoided by design | `DEVELOP.md` |
| Invalid IPv4 on router | Drop + warn | `router.rs` |
| Unexpected IP version on TUN | Warn + skip | `Forwarder` |
| ICMP / non-TCP-UDP | No connection created → drop | `Router` |
| IPv4 fragments | No reassembly logic observed → unsupported | Gap in brief/code survey |
| Spurious mio/NIO events | Ignored WouldBlock | client/connection code |

---

## 7. Multi-device failure isolation

One device USB unplug should remove one reverse + one relay client. Other devices’ clients continue on the shared `TcpListener`.

**Risk:** Orchestrator mistakenly killing shared relay on first device disconnect — **do not**; stop relay only when no tethered devices remain (product policy).

---

## 8. What is fragile enough to engineer around (not rewrite)

1. Reconnect ≠ session resume (document + UX).  
2. Reverse is sticky configuration that must be re-asserted after adb/USB events.  
3. Handshake distinguishes “reverse only” vs “relay up”.  
4. VPN revoke / competing VPN are platform-hard stops.  
5. Blocking TUN read wake workaround — leave as-is unless replacing I/O model with measured gain.

---

## 9. Recommended orchestrator error taxonomy (desktop)

Map to UI without changing wire protocol:

- `AdbUnavailable`
- `DeviceMissing`
- `ReverseFailed`
- `RelayNotRunning` / `RelayCrashed`
- `VpnNotPrepared` / `VpnRevoked` / `VpnEstablishFailed`
- `TunnelDisconnected` (client retrying)
- `TunnelConnected`

Signals already partially exist via notification failure flag on device; host needs symmetric events.

