# IPV6_ANALYSIS.md

**Upstream pin:** `1eb2e58` / v2.5.1  
**Sources:** `README.md` (“relays TCP and UDP over IPv4 … does not support IPv6”), `Forwarder.java`, `IPPacketOutputStream.java`, Rust/Java IPv4-only packet types, `GnirehtetService` routes/DNS, [VpnService.Builder](https://developer.android.com/reference/android/net/VpnService.Builder).

---

## 1. Current behavior (facts)

| Area | IPv4 | IPv6 |
|------|------|------|
| README capability claim | Yes | Explicitly no |
| TUN address | `10.0.0.2/32` | None configured |
| Default route | `0.0.0.0/0` | No `::/0` |
| DNS added | IPv4 DNS only (`8.8.8.8` or `-d`) | No IPv6 DNS servers in builder |
| Device forwarder | Forwards version==4 only | Logs “Unexpected packet IP version” and drops |
| `IPPacketOutputStream` | Parses v4 length | On version≠4: error + **clear buffer** |
| Relay parsers | `Ipv4Packet` / `IPv4Header` | No v6 types |
| Router protocols | TCP/UDP over IPv4 headers | N/A |
| Host sockets | `Ipv4Addr` / IPv4 destinations | Not selected for v6 destinations |

**Conclusion:** Dual-stack apps on the device may still *generate* IPv6 if the system prefers it, but gnirehtet **black-holes** those packets. Depending on Android routing/DNS, some hosts may be IPv6-only from the app’s perspective → failures unless Happy Eyeballs falls back to IPv4.

---

## 2. What “IPv6 support” would require

Minimum viable IPv6 reverse tether (analysis only — **not** an implementation plan to execute now):

### A. VpnService configuration

- `builder.addAddress(<ULA or unique local IPv6>, prefix)`
- `builder.addRoute("::", 0)` or selective v6 routes
- Optional IPv6 DNS (`addDnsServer` with v6 literals)
- MTU policy for v6 (possibly different from `0x4000`)

Platform: VpnService FD still delivers IP packets starting with version nibble — v6 version == 6.

### B. Device forwarder / packet boundary layer

- Accept version 6 in `Forwarder`.
- Extend or parallelize `IPPacketOutputStream` to parse IPv6 payload length (IPv6 header layout ≠ IPv4 total length at offset 2). **Blindly reusing IPv4 length parsing is incorrect** and today’s “clear buffer on non-v4” would destroy stream sync if v6 were mixed without a proper parser.

### C. Wire protocol

Options:

1. **Same stream, mixed v4/v6 packets** — simplest evolution; demux by version nibble; requires both ends upgraded together.  
2. **Separate tunnels / ports** — heavier orchestration.  
3. **Length-prefixed frames** — breaks APK compatibility; needs dual-compat.

Compatibility rule from project architecture: protocol changes need ADR + dual-compat.

### D. Relay data plane

- `Ipv6Packet` parse/serialize, extension header policy (drop unknown ext headers vs limited support).
- `ConnectionId` with 128-bit addresses.
- Host `TcpStream`/`UdpSocket` to IPv6 destinations.
- TCP userspace state machine mostly version-agnostic but checksum/pseudo-header differs (TCP/UDP checksum includes IPv6 pseudo-header).
- NAT66 vs NAT64 vs “host sockets with global v6” product choice:

  - If the **host has native IPv6**, opening IPv6 sockets is the analogue of today’s IPv4 NAT-like behavior.  
  - If the host is IPv4-only, you need **NAT64/DNS64** or refuse v6 — large scope.

### E. DNS

- Device DNS may return AAAA. Without data plane v6 (or DNS64), AAAA answers cause avoidable failures. Strategies: DNS64 on relay, filter AAAA on a DNS proxy, or only add v4 DNS and hope — last is fragile on modern Android.

---

## 3. Effort / risk assessment

| Work item | Risk | Notes |
|-----------|------|-------|
| APK VpnService v6 addr/route | Medium | Platform OK; testing matrix heavy |
| Packet boundary for v6 | Medium–Hard | Must not desync TCP stream |
| Relay IPv6 parse + checksum | Hard | Touches hottest paths |
| TCP TCB vs IPv6 | Medium | Pseudo-header / MSS clamping |
| UDP | Lower than TCP | Still need checksum + buffer sizes |
| Host without IPv6 uplink | Product Hard | NAT64 or document limitation |
| Keep old APK working | Process | Dual-compat or feature-flag |

**This is a justified multi-milestone feature, not a tidy refactor.** It is **not** required to rewrite the existing IPv4 networking layer beforehand; IPv6 should extend or parallel the IPv4 path with conformance tests.

---

## 4. Interim mitigations (no protocol change)

1. Ensure IPv4 routes cover needed traffic (`0.0.0.0/0` default already).  
2. Prefer configuring working IPv4 DNS (`-d`) appropriate to the network.  
3. Document that IPv6-only destinations will fail.  
4. Optional future APK knobs: reject AAAA via private DNS proxy — only if product demands before full v6.

---

## 5. Recommendation for the desktop project

- **MVP:** Keep IPv4-only path; treat IPv6 as known upstream limitation (README).  
- **Do not** rewrite IPv4 relay “to prepare for IPv6”.  
- If IPv6 becomes a milestone: start with protocol ADR (mixed packets vs prefixed), APK+relay dual support, and explicit host uplink requirements (native v6 vs NAT64).  
- Success metric: AAAA + TCP/UDP to a known IPv6-only endpoint works with host native IPv6; IPv4 regression suite stays green.

