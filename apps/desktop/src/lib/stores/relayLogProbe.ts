/**
 * Safe MVP probe: owned-relay LogLine only (HANDSHAKE_LIVENESS_MVP.md).
 * Do not invent wire pings. Connect ≠ Sharing.
 */

/** Match upstream tunnel_server: `Client #<id> connected` / `disconnected`. */
const CLIENT_LINE =
  /Client #(\d+)\s+(connected|disconnected)\b/i;

export type RelayClientProbe =
  | { kind: "connected"; clientId: number }
  | { kind: "disconnected"; clientId: number }
  | null;

/**
 * Parse a single LogLine message from source `relay` while we own the relay.
 * Returns null if not a client liveness line (caller must gate on owned + source).
 */
export function parseRelayClientLogLine(message: string): RelayClientProbe {
  const m = CLIENT_LINE.exec(message);
  if (!m) return null;
  const clientId = Number(m[1]);
  if (!Number.isFinite(clientId)) return null;
  if (m[2].toLowerCase() === "connected") {
    return { kind: "connected", clientId };
  }
  return { kind: "disconnected", clientId };
}
