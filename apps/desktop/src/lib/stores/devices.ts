/**
 * Device list + selection helpers for the Phase 1 shell.
 * Truth comes from list_devices invoke and/or DeviceChanged events.
 */

import type { DeviceInfo } from "$lib/types/orchestrator";

/** Visible-window poll interval (DESKTOP_LIFECYCLE §3.1). */
export const DEVICE_POLL_VISIBLE_MS = 2000;

export function applyDeviceList(
  next: DeviceInfo[],
  selectedSerial: string | null,
): { devices: DeviceInfo[]; selectedSerial: string | null } {
  let selected = selectedSerial;
  if (selected && !next.some((d) => d.serial === selected)) {
    selected = null;
  }
  // MVP one device: auto-select the only row.
  if (!selected && next.length === 1) {
    selected = next[0].serial;
  }
  return { devices: next, selectedSerial: selected };
}

export function selectedDevice(
  devices: DeviceInfo[],
  serial: string | null,
): DeviceInfo | null {
  if (!serial) return null;
  return devices.find((d) => d.serial === serial) ?? null;
}
