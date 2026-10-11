# PROTOCOL.md

**Upstream pin:** `1eb2e58` / v2.5.1  
**Sources:** `DEVELOP.md` (Client / Packets), `RelayTunnel.java`, `IPPacketOutputStream.java`, `relay-rust` `client.rs`, `tunnel_server.rs`, `ipv4_packet_buffer.rs`, `cli_args.rs`, Android `adb reverse` usage in `main.rs`.

---

## 1. Transport under the “protocol”

There is **no** application-layer TLV beyond a one-time client id. After handshake, the TCP payload is a **concatenation of raw IPv4 packets**.

Path:

```text
Device LocalSocket("gnirehtet")
   ←adb reverse localabstract:gnirehtet tcp:PORT→
Host 127.0.0.1:PORT (DEFAULT_PORT = 31416)
```

Command that installs the reverse:

```text
adb [-s SERIAL] reverse localabstract:gnirehtet tcp:<port>
```

Evidence: `cmd_tunnel` in `relay-rust/src/main.rs`; `RelayTunnel.LOCAL_ABSTRACT_NAME = "gnirehtet"`.

---

## 2. Connection handshake

1. Device connects to abstract local socket `gnirehtet` (`LocalSocketAddress`).
2. Host accepts TCP connection on the relay port.
3. Host assigns `client_id = next_client_id++` (`u32`, starts at 0).
4. Host must send **4 bytes**, big-endian client id, **before** treating the session as ready for packet relay.
5. Device `RelayTunnel.readClientId` uses `DataInputStream.readInt()` and only then logs connected.

**Why:** `adb reverse` can make `connect()` succeed even when the relay is down; the first read fails. Forcing an immediate server→client write detects that (`DEVELOP.md`, `RelayTunnel` javadoc).

**Rust detail:** `Client` starts with `pending_id_bytes = 4` and writable interest only until the id is fully written (`client.rs`).

**Byte order:** Java `DataInputStream.readInt()` is big-endian; Rust writes the id as four big-endian bytes (same convention as other `binary` helpers / Java `readInt`). Treat wire client id as **network byte order u32**.

---

## 3. Packet encapsulation

| Direction | Framing |
|-----------|---------|
| Device → host | Each TUN read is one IPv4 packet; written as opaque bytes on the stream (`RelayTunnel.send`). |
| Host → device | One or more IPv4 packets concatenated on the TCP stream. |
| Demux on receive | Readers parse IPv4 version nibble + **total length** at offset 2 (16-bit BE) to find boundaries. |

**No length prefix, no magic, no compression, no encryption.**

Reassembly implementations:

- Device: `IPPacketOutputStream` (buffer up to `2 * 65536`, sink complete packets only).
- Host: `Ipv4PacketBuffer` (`MAX_PACKET_LENGTH`, peek version/length, `next()` consumes).

Non-IPv4 on device path: logged/dropped (`Forwarder`, `IPPacketOutputStream` clears buffer on bad version).

---

## 4. IPv4 packet contract

Assumptions encoded in code:

- Version must be **4**.
- Header length / total length fields must yield a consistent packet (`is_valid()` on relay).
- Protocols handled by router: **TCP** and **UDP** (others: no connection created → drop with error log).
- IPv6 not spoken (see `IPV6_ANALYSIS.md`).

MTU on TUN is `0x4000`; relay TCP payload sizing uses the same constant (`tcp_connection.rs`). IP total length still caps at 16-bit field (`IPPacketOutputStream.MAX_IP_PACKET_LENGTH = 1 << 16`).

---

## 5. Connection identification inside the relay

**Host destination rewrite:** when the device targets `10.0.2.2`, the relay opens the host socket toward **loopback** instead (`LOCALHOST_FORWARD` in `AbstractConnection.java` / `connection.rs`).

Not on the wire as a header — derived from each IPv4 packet:

```text
ConnectionId = (protocol, src_addr, src_port, dst_addr, dst_port)
```

`Router` finds or creates `TcpConnection` / `UdpConnection` (`router.rs`, `DEVELOP.md`).

---

## 6. UDP semantics

- First packet clones swapped headers as templates for replies (`DEVELOP.md`).
- Datagram boundaries preserved end-to-end through the relay.
- Connected UDP socket to destination (`UdpConnection::create_socket`).
- Idle expiry: `IDLE_TIMEOUT_SECONDS = 2 * 60`; cleaning driven by relay poll timeout (~1 minute cadence per `DEVELOP.md` / `relay.rs` cleanup).

---

## 7. TCP semantics

- Userspace TCP state machine toward the **device** (`TcpState` in `tcp_connection.rs`, RFC 793 oriented).
- Host OS TCP toward the **network**.
- Reliability strategy: never lose packets already read from the network socket; drop device→network if needed; respect client window; valid checksums (`DEVELOP.md`).
- When client send buffer is full, stop reading network socket until space returns (`PacketSource` pull model).

---

## 8. Control plane (out of band)

Not part of the packet stream — host uses adb intents:

| Action | Intent |
|--------|--------|
| Start VPN UX | `com.genymobile.gnirehtet.START` → `GnirehtetActivity` |
| Stop | `com.genymobile.gnirehtet.STOP` |
| DNS / routes | Intent extras `dnsServers`, `routes` (CLI `--esa`) |

VPN service internal actions: `START_VPN` / `CLOSE_VPN`.

---

## 9. Conformance checklist (for desktop / dual-relay)

1. Listen on chosen port (default 31416) on localhost.
2. On accept, write 4-byte BE client id before any other data.
3. Parse concatenated IPv4 packets by total length; never assume one read = one packet.
4. Accept only IPv4 TCP/UDP for forwarding.
5. Preserve UDP datagram boundaries.
6. TCP path must not emit network payloads the client cannot buffer.
7. Remain compatible with upstream APK `RelayTunnel` / `IPPacketOutputStream`.

Any change to this contract needs an ADR + dual-compatibility plan (`ARCHITECTURE.md` consistency rules).

