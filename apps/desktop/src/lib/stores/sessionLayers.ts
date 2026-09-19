/**
 * Three-layer session evidence (EVENT_STATUS_MAP).
 *
 * Sharing is NEVER claimed from intent-sent alone.
 * Optional owned-relay LogLine probe (`Client #<n> connected`) upgrades
 * Tunnel/client liveness copy to “Relay accepted client” — not Sharing.
 * Device VPN stays Pending until stronger evidence; see
 * docs/architecture/networking/HANDSHAKE_LIVENESS_MVP.md.
 */

export type TunnelLayer = "unknown" | "ok" | "lost" | "failed";
/**
 * pending = intent sent / waiting consent.
 * active reserved for device-side handshake (logcat etc.) — unused until reliable.
 */
export type VpnLayer = "idle" | "pending" | "error";
export type SessionChip =
  | "Idle"
  | "Starting"
  | "Waiting for VPN"
  | "Interrupted"
  | "Stopping"
  | "Error";
// Deliberately omit "Sharing" until Relay + Tunnel + Device VPN are all healthy
// (probe alone is insufficient per EVENT_STATUS_MAP / HANDSHAKE_LIVENESS_MVP).

export type SessionLayers = {
  tunnel: TunnelLayer;
  /** Serial for which tunnel evidence applies. */
  tunnelSerial: string | null;
  /**
   * Host relay accepted a TCP client (LogLine `Client #<n> connected` from owned relay).
   * Drives Tunnel chip copy only — never Sharing by itself.
   */
  clientAccepted: boolean;
  clientId: number | null;
  vpn: VpnLayer;
  vpnSerial: string | null;
  /** Wall-clock ms when VPN pending started (for START_TIMEOUT heuristic). */
  vpnPendingSinceMs: number | null;
  chipOverride: SessionChip | null;
  lastCode: string | null;
};

export const VPN_PENDING_TIMEOUT_MS = 60_000;

export function initialSessionLayers(): SessionLayers {
  return {
    tunnel: "unknown",
    tunnelSerial: null,
    clientAccepted: false,
    clientId: null,
    vpn: "idle",
    vpnSerial: null,
    vpnPendingSinceMs: null,
    chipOverride: null,
    lastCode: null,
  };
}

export function markTunnelOk(s: SessionLayers, serial: string): SessionLayers {
  return {
    ...s,
    tunnel: "ok",
    tunnelSerial: serial,
    lastCode: s.lastCode === "TUNNEL_LOST" || s.lastCode === "TUNNEL_FAILED" ? null : s.lastCode,
    chipOverride: s.chipOverride === "Interrupted" ? null : s.chipOverride,
  };
}

export function markTunnelFailed(s: SessionLayers, serial: string | null): SessionLayers {
  return {
    ...s,
    tunnel: "failed",
    tunnelSerial: serial,
    clientAccepted: false,
    clientId: null,
    lastCode: "TUNNEL_FAILED",
    chipOverride: "Error",
  };
}

export function markTunnelLost(s: SessionLayers): SessionLayers {
  if (s.tunnel !== "ok") {
    return {
      ...s,
      tunnel: s.tunnel === "unknown" ? "unknown" : "lost",
      clientAccepted: false,
      clientId: null,
      vpn: s.vpn === "pending" ? "idle" : s.vpn,
      vpnPendingSinceMs: null,
    };
  }
  return {
    ...s,
    tunnel: "lost",
    clientAccepted: false,
    clientId: null,
    vpn: "idle",
    vpnPendingSinceMs: null,
    lastCode: "TUNNEL_LOST",
    chipOverride: "Interrupted",
  };
}

/** After successful start/run intent — Waiting for VPN, not Active / not Sharing. */
export function markVpnPending(s: SessionLayers, serial: string, nowMs: number): SessionLayers {
  return {
    ...s,
    vpn: "pending",
    vpnSerial: serial,
    vpnPendingSinceMs: nowMs,
    lastCode: "VPN_PERMISSION_PENDING",
    chipOverride: null,
  };
}

export function markVpnIdle(s: SessionLayers): SessionLayers {
  return {
    ...s,
    vpn: "idle",
    vpnSerial: null,
    vpnPendingSinceMs: null,
    clientAccepted: false,
    clientId: null,
    lastCode:
      s.lastCode === "VPN_PERMISSION_PENDING" || s.lastCode === "START_TIMEOUT"
        ? null
        : s.lastCode,
    chipOverride: null,
  };
}

export function markVpnTimeout(s: SessionLayers): SessionLayers {
  return {
    ...s,
    vpn: "error",
    vpnPendingSinceMs: null,
    lastCode: "START_TIMEOUT",
    chipOverride: "Error",
  };
}

/** Owned-relay LogLine: Client #<n> connected → Tunnel chip “Relay accepted client”. */
export function markClientAccepted(s: SessionLayers, clientId: number): SessionLayers {
  return {
    ...s,
    clientAccepted: true,
    clientId,
    // Keep VPN Pending — host saw TCP client, not full three-layer Sharing.
  };
}

/** Owned-relay LogLine: Client #<n> disconnected (or relay death). */
export function clearClientAccepted(s: SessionLayers): SessionLayers {
  if (!s.clientAccepted && s.clientId == null) return s;
  return {
    ...s,
    clientAccepted: false,
    clientId: null,
  };
}

export function clearSessionIntent(s: SessionLayers): SessionLayers {
  return {
    ...initialSessionLayers(),
    // Keep tunnel unknown after stop; reverse may still exist but we do not claim it.
  };
}

export function relayLayerHealthy(relayState: string, ownedBySession: boolean): boolean {
  return relayState === "relay_running" && ownedBySession;
}

/**
 * Derive session chip. Never returns Sharing — need Relay+Tunnel+VPN Active
 * (EVENT_STATUS_MAP); client-accepted probe alone is not enough.
 */
export function deriveSessionChip(opts: {
  layers: SessionLayers;
  busy: boolean;
  stopping: boolean;
  relayHealthy: boolean;
}): SessionChip {
  const { layers, busy, stopping, relayHealthy } = opts;
  if (stopping) return "Stopping";
  if (layers.chipOverride === "Error" || layers.vpn === "error") return "Error";
  if (layers.chipOverride === "Interrupted" || layers.tunnel === "lost") return "Interrupted";
  if (busy) return "Starting";
  if (layers.vpn === "pending") return "Waiting for VPN";
  // Even with clientAccepted + relay + tunnel ok, VPN Active missing → never Sharing.
  void relayHealthy;
  void layers.clientAccepted;
  return "Idle";
}

export function tunnelLabel(t: TunnelLayer, clientAccepted = false): string {
  if (clientAccepted && (t === "ok" || t === "unknown")) {
    return "Relay accepted client";
  }
  switch (t) {
    case "ok":
      return "OK";
    case "lost":
      return "Lost";
    case "failed":
      return "Failed";
    default:
      return "—";
  }
}

export function vpnLabel(v: VpnLayer): string {
  switch (v) {
    case "pending":
      return "Waiting";
    case "error":
      return "Error";
    default:
      return "Off";
  }
}
