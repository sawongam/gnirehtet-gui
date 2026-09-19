import { invoke } from "@tauri-apps/api/core";
import type { AdbInfo, DeviceInfo, RelayStatePayload } from "../types/orchestrator";

export const ensureAdb = () => invoke<AdbInfo>("ensure_adb");
export const listDevices = () => invoke<DeviceInfo[]>("list_devices");
export const startRelay = (port?: number) =>
  invoke<RelayStatePayload>("start_relay", { port: port ?? null });
export const stopRelay = () => invoke<RelayStatePayload>("stop_relay");
export const getRelayState = () => invoke<RelayStatePayload>("get_relay_state");

/** Thin ADB / tunnel verbs via SessionController. */
export const install = (serial?: string | null) =>
  invoke<void>("install", { serial: serial ?? null });
export const startClient = (opts?: {
  serial?: string | null;
  dnsServers?: string | null;
  routes?: string | null;
  port?: number | null;
}) =>
  invoke<void>("start_client", {
    serial: opts?.serial ?? null,
    dnsServers: opts?.dnsServers ?? null,
    routes: opts?.routes ?? null,
    port: opts?.port ?? null,
  });
export const stopClient = (serial?: string | null) =>
  invoke<void>("stop_client", { serial: serial ?? null });
export const resetTunnel = (serial?: string | null, port?: number | null) =>
  invoke<void>("reset_tunnel", { serial: serial ?? null, port: port ?? null });
export const runSession = (opts?: {
  serial?: string | null;
  dnsServers?: string | null;
  routes?: string | null;
  port?: number | null;
}) =>
  invoke<RelayStatePayload>("run_session", {
    serial: opts?.serial ?? null,
    dnsServers: opts?.dnsServers ?? null,
    routes: opts?.routes ?? null,
    port: opts?.port ?? null,
  });

/**
 * Window close / Quit: best-effort stop client + clear_owned_relay (epoch bump).
 * Prefer this before destroy so UI can clear layers; Exit also teardowns as safety net.
 */
export const prepareQuit = (serial?: string | null) =>
  invoke<RelayStatePayload>("prepare_quit", { serial: serial ?? null });
