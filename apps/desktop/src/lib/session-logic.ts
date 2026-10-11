import type { DeviceInfo } from "./types";

/**
 * Session lifecycle as shown to the user:
 * - starting: relay/client being started
 * - waiting: client started, waiting for the VPN prompt to be accepted
 * - connected: the phone's VPN is connected to the relay
 * - interrupted: the session device disappeared (cable unplugged)
 * - failed: the relay crashed while a session was active
 * - stopping: disconnect in progress
 */
export type Phase =
  | "idle"
  | "starting"
  | "waiting"
  | "connected"
  | "interrupted"
  | "failed"
  | "stopping";

/** Phases in which the device side may still be running. */
export function isSessionPhase(phase: Phase): boolean {
  return phase !== "idle";
}

export function isReady(device: DeviceInfo | undefined | null): boolean {
  return device?.state === "device";
}

/**
 * Keep the current selection when possible; during a session the session
 * device stays selected; otherwise prefer the first ready device.
 */
export function pickSelection(
  devices: DeviceInfo[],
  current: string | null,
  sessionSerial: string | null,
): string | null {
  if (sessionSerial) return sessionSerial;
  if (current && devices.some((d) => d.serial === current)) return current;
  return devices.find(isReady)?.serial ?? devices[0]?.serial ?? null;
}

/** What a new device list means for the running session. */
export function sessionDeviceChange(
  phase: Phase,
  sessionSerial: string | null,
  devices: DeviceInfo[],
): "lost" | "returned" | null {
  if (!sessionSerial) return null;
  const ready = isReady(devices.find((d) => d.serial === sessionSerial));
  if (!ready && (phase === "waiting" || phase === "connected")) return "lost";
  if (ready && phase === "interrupted") return "returned";
  return null;
}

export function deviceName(device: DeviceInfo): string {
  return device.model ?? device.product ?? device.serial;
}

/** 0:42, 12:05, 1:02:03 */
export function formatDuration(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}
