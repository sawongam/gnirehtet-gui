/** IPC payloads, mirrored from `src-tauri/src/types.rs`. */

export type AdbInfo = {
  path: string;
  version: string;
};

export type DeviceInfo = {
  serial: string;
  /** Raw adb state: "device", "unauthorized", "offline", "no permissions", ... */
  state: string;
  model: string | null;
  product: string | null;
};

export type RelayState = "stopped" | "running" | "crashed";

export type RelayStatus = {
  state: RelayState;
  port: number | null;
  pid: number | null;
  message: string | null;
};

export type SessionSnapshot = {
  relay: RelayStatus;
  serial: string | null;
  connectedClients: number;
};

export type ClientEvent = {
  connected: boolean;
  clientId: number;
  connectedClients: number;
};

export type LogLevel = "debug" | "info" | "warn" | "error";

export type LogEntry = {
  timestampMs: number;
  level: LogLevel;
  source: "app" | "relay";
  message: string;
};

export type ConnectOptions = {
  port: number;
  dnsServers: string | null;
  routes: string | null;
};

export type Environment = {
  appVersion: string;
  adbPath: string;
  relayPath: string;
  relayFound: boolean;
  apkPath: string;
  apkFound: boolean;
};

/** Rejection value of every command. */
export type CommandError = {
  code: string;
  message: string;
};
