/** CamelCase IPC shapes mirrored from Rust serde(rename_all = "camelCase"). */

export type AdbInfo = {
  path: string;
  version: string;
  available: boolean;
};

/** Raw adb state token: device | unauthorized | offline | … */
export type DeviceInfo = {
  serial: string;
  adbState: string;
  model?: string | null;
  product?: string | null;
  /** Optional Android version label when known (e.g. "14"). */
  androidVersion?: string | null;
};

/** DeviceChanged event — full list snapshot (MVP replace-all). */
export type DeviceChangedPayload = {
  devices: DeviceInfo[];
};

/** relay_stopped | relay_starting | relay_running | relay_error | relay_exited */
export type RelayStatePayload = {
  state: string;
  port?: number | null;
  pid?: number | null;
  ownedBySession: boolean;
  message?: string | null;
};

export type LogLine = {
  timestampMs: number;
  level: string;
  source: string;
  message: string;
};

export type OrchError = {
  code: string;
  message: string;
};

export type AppError = {
  code: string;
  message: string;
  serial?: string | null;
};
