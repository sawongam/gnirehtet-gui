/**
 * ERROR_UX titles / short explanations (Phase 2 shell).
 * Source: docs/architecture/ux/ERROR_UX.md — do not invent codes or Sharing claims.
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
  APK_MISSING: {
    title: "Helper APK not found",
    explanation:
      "The Android helper (gnirehtet client) isn’t bundled or the configured path is empty.",
    recoveryHint: "Set GNIREHTET_APK or place resources/gnirehtet.apk · Copy error.",
  },
  INSTALL_FAILED: {
    title: "Couldn’t install helper",
    explanation:
      "adb install failed (storage, signature, OEM install-via-USB, etc.).",
    recoveryHint: "Reinstall helper · Enable Install via USB (MIUI) · Copy log.",
  },
  PORT_IN_USE: {
    title: "Relay port already in use",
    explanation:
      "Nothing new can listen on the configured port (default 31416). Another relay may be bound.",
    recoveryHint: "Stop leftover relay · Change port next start · Copy error.",
  },
  RELAY_START_FAILED: {
    title: "Relay didn’t start",
    explanation: "The PC relay process exited or never listened.",
    recoveryHint: "Retry Start/Run · Allow app through firewall · Check logs.",
  },
  RELAY_CRASHED: {
    title: "Relay stopped unexpectedly",
    explanation: "The phone path cannot continue without the relay.",
    recoveryHint: "Restart Run · Stop (clean phone side) · Copy logs.",
  },
  TUNNEL_FAILED: {
    title: "Couldn’t create tunnel",
    explanation: "adb reverse (tunnel) did not set up. Traffic can’t reach the relay.",
    recoveryHint: "Repair tunnel · Replug USB · Refresh device · Copy log.",
  },
  TUNNEL_LOST: {
    title: "Sharing interrupted",
    explanation:
      "Unplugging USB kills the adb reverse tunnel. The relay may still be listening; the phone path is broken.",
    recoveryHint: "Replug · Repair tunnel · Restart Run · Stop.",
  },
  CLIENT_START_FAILED: {
    title: "Couldn’t start helper on phone",
    explanation: "The start intent failed.",
    recoveryHint: "Retry · Open helper manually · Check OEM Permission Monitoring.",
  },
  VPN_PERMISSION_PENDING: {
    title: "Waiting for VPN permission",
    explanation:
      "Android must show a Connection request. The desktop cannot tap it. Do not treat this as Sharing yet.",
    recoveryHint: "Allow on phone · I’ve allowed it · Stop.",
  },
  START_TIMEOUT: {
    title: "Start timed out",
    explanation:
      "Start didn’t reach a healthy Relay + Tunnel + Device VPN handshake in time. Intent-sent is not connected.",
    recoveryHint: "Retry · I’ve allowed VPN · Repair tunnel · Stop.",
  },
  STOP_FAILED: {
    title: "Couldn’t stop cleanly",
    explanation: "Stop intent or teardown failed. The phone key icon or relay may linger.",
    recoveryHint: "Retry Stop · Stop leftover relay · Stop helper on phone.",
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
