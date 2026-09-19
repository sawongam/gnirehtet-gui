/**
 * ERROR_UX titles / short explanations for device + ADB codes (Phase 1 shell).
 * Source: docs/architecture/ux/ERROR_UX.md — do not invent Sharing claims.
 */

export type ErrorUxCopy = {
  code: string;
  title: string;
  explanation: string;
  recoveryHint: string;
};

const TABLE: Record<string, Omit<ErrorUxCopy, "code">> = {
  ADB_MISSING: {
    title: "ADB not found",
    explanation:
      "Reverse tether needs the Android Debug Bridge (adb). It isn’t on the configured path.",
    recoveryHint: "Install platform-tools or set the ADB env / path in Settings (later).",
  },
  ADB_PATH_INVALID: {
    title: "ADB path invalid",
    explanation: "The configured adb path isn’t a working binary (adb version failed).",
    recoveryHint: "Clear the path or point at a working adb binary.",
  },
  NO_DEVICES: {
    title: "No Android devices detected",
    explanation:
      "ADB is running but no phones/tablets appear. Often a charge-only cable, USB debugging off, or a bad port.",
    recoveryHint: "Refresh · enable USB debugging · try another cable/port.",
  },
  DEVICE_UNAUTHORIZED: {
    title: "Waiting for USB authorization",
    explanation:
      "The phone is connected but hasn’t allowed this computer. Unlock it and accept Allow USB debugging?",
    recoveryHint: "I’ve allowed it (refresh) · revoke authorizations on phone, replug.",
  },
  DEVICE_OFFLINE: {
    title: "Device offline",
    explanation:
      "ADB sees the serial but the transport is offline (sleep, cable glitch, driver).",
    recoveryHint: "Wake/replug · Refresh · try another cable/port.",
  },
  MULTIPLE_DEVICES_NO_SELECTION: {
    title: "Select a device",
    explanation: "More than one device is connected. This version shares with one device at a time.",
    recoveryHint: "Select a row · Refresh.",
  },
};

export function errorUxFor(code: string): ErrorUxCopy | null {
  const row = TABLE[code];
  if (!row) return null;
  return { code, ...row };
}

/** Map raw AdbDevice.state → ERROR_UX device code when applicable. */
export function deviceStateToUxCode(adbState: string): string | null {
  switch (adbState) {
    case "unauthorized":
      return "DEVICE_UNAUTHORIZED";
    case "offline":
      return "DEVICE_OFFLINE";
    default:
      return null;
  }
}

/** Human label for the device row chip (text + color class, not Sharing). */
export function deviceStateLabel(adbState: string): string {
  switch (adbState) {
    case "device":
      return "ready";
    case "unauthorized":
      return "unauthorized";
    case "offline":
      return "offline";
    default:
      return adbState || "unknown";
  }
}

export function deviceStateClass(adbState: string): "ok" | "warn" | "err" | "muted" {
  switch (adbState) {
    case "device":
      return "ok";
    case "unauthorized":
      return "warn";
    case "offline":
      return "err";
    default:
      return "muted";
  }
}
