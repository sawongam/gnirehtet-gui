import { invoke } from "@tauri-apps/api/core";
import type { AdbInfo, DeviceInfo, RelayStatePayload } from "../types/orchestrator";

export const ensureAdb = () => invoke<AdbInfo>("ensure_adb");
export const listDevices = () => invoke<DeviceInfo[]>("list_devices");
export const startRelay = (port?: number) =>
  invoke<RelayStatePayload>("start_relay", { port: port ?? null });
export const stopRelay = () => invoke<RelayStatePayload>("stop_relay");
export const getRelayState = () => invoke<RelayStatePayload>("get_relay_state");
