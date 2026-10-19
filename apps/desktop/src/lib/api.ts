import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AdbInfo,
  ClientEvent,
  CommandError,
  ConnectOptions,
  DeviceInfo,
  Environment,
  LogEntry,
  RelayStatus,
  SessionSnapshot,
} from "./types";

export const api = {
  checkAdb: () => invoke<AdbInfo>("check_adb"),
  listDevices: () => invoke<DeviceInfo[]>("list_devices"),
  getSession: () => invoke<SessionSnapshot>("get_session"),
  connect: (serial: string, options: ConnectOptions) =>
    invoke<RelayStatus>("connect", { serial, options }),
  disconnect: () => invoke<RelayStatus>("disconnect"),
  installClient: (serial: string) => invoke<void>("install_client", { serial }),
  repairTunnel: (serial: string) => invoke<void>("repair_tunnel", { serial }),
  getEnvironment: () => invoke<Environment>("get_environment"),
  openRepo: () => invoke<void>("open_repo"),
};

export type Listeners = {
  onLog: (entry: LogEntry) => void;
  onRelayStatus: (status: RelayStatus) => void;
  onRelayClient: (event: ClientEvent) => void;
};

export async function subscribe(listeners: Listeners): Promise<UnlistenFn> {
  const unlisten = await Promise.all([
    listen<LogEntry>("log", (e) => listeners.onLog(e.payload)),
    listen<RelayStatus>("relay-status", (e) => listeners.onRelayStatus(e.payload)),
    listen<ClientEvent>("relay-client", (e) => listeners.onRelayClient(e.payload)),
  ]);
  return () => unlisten.forEach((fn) => fn());
}

/** True when running inside the Tauri webview (false in a plain browser). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Normalize anything a command may reject with. */
export function toCommandError(error: unknown): CommandError {
  if (error && typeof error === "object" && "code" in error && "message" in error) {
    const { code, message } = error as CommandError;
    return { code: String(code), message: String(message) };
  }
  return { code: "INTERNAL", message: error instanceof Error ? error.message : String(error) };
}
