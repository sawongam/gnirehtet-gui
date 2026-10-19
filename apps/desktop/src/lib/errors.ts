/** User-facing explanations for backend error codes. */

export type ErrorHelp = {
  title: string;
  hint: string;
};

const HELP: Record<string, ErrorHelp> = {
  ADB_MISSING: {
    title: "adb was not found",
    hint: "Install Android platform-tools and make sure adb is on your PATH, or set the ADB environment variable to its location, then restart the app.",
  },
  ADB_PATH_INVALID: {
    title: "adb could not be run",
    hint: "The configured adb is not a working executable. Check the ADB environment variable or reinstall platform-tools.",
  },
  ADB_TIMEOUT: {
    title: "adb is not responding",
    hint: "Unplug and reconnect the device. If it persists, run “adb kill-server” in a terminal and try again.",
  },
  ADB_FAILED: {
    title: "adb reported an error",
    hint: "Open Details or the Log for the full message.",
  },
  APK_MISSING: {
    title: "The gnirehtet client APK is missing",
    hint: "The installation is incomplete. Reinstall the app, or set GNIREHTET_APK to the path of gnirehtet.apk.",
  },
  INSTALL_FAILED: {
    title: "Could not install the client on the phone",
    hint: "Unlock the phone and check for a confirmation prompt. Some phones (e.g. Xiaomi) require enabling “Install via USB” in developer options.",
  },
  INVALID_ARGUMENT: {
    title: "Invalid setting",
    hint: "Check the connection options in Preferences.",
  },
  DEVICE_UNAUTHORIZED: {
    title: "The phone has not authorized this computer",
    hint: "Unlock the phone and accept the “Allow USB debugging?” prompt. If it doesn’t appear, revoke USB debugging authorizations in developer options and reconnect.",
  },
  DEVICE_OFFLINE: {
    title: "The phone is offline",
    hint: "Unlock the phone, or unplug and reconnect the cable.",
  },
  DEVICE_NOT_FOUND: {
    title: "The phone is no longer connected",
    hint: "Reconnect the USB cable and select the device again.",
  },
  MULTIPLE_DEVICES: {
    title: "Several devices are connected",
    hint: "Select the phone to use under Phones.",
  },
  PORT_IN_USE: {
    title: "The relay port is already in use",
    hint: "Another gnirehtet relay may still be running (for example “gnirehtet run” in a terminal). Stop it, or choose a different port in Preferences.",
  },
  RELAY_NOT_FOUND: {
    title: "The relay program is missing",
    hint: "The installation is incomplete. Reinstall the app, or set GNIREHTET_BIN to the path of the gnirehtet executable.",
  },
  RELAY_START_FAILED: {
    title: "The relay could not start",
    hint: "The Log has the details. A firewall or security tool may be blocking it.",
  },
  RELAY_ALREADY_RUNNING: {
    title: "The relay is already running",
    hint: "A connection is still active. Disconnect first, then connect again.",
  },
  RELAY_CRASHED: {
    title: "The relay stopped unexpectedly",
    hint: "Reconnect to start it again. The Log has the details.",
  },
  TUNNEL_FAILED: {
    title: "Could not create the USB tunnel",
    hint: "Reconnect the cable and try again.",
  },
  CLIENT_START_FAILED: {
    title: "Could not start the client on the phone",
    hint: "Unlock the phone and try again. If it keeps failing, reinstall the client under Details.",
  },
};

export function errorHelp(code: string): ErrorHelp {
  return (
    HELP[code] ?? {
      title: "Something went wrong",
      hint: "Open Details or the Log for the full message.",
    }
  );
}

export type DeviceStateInfo = {
  label: string;
  tone: "success" | "warning" | "danger" | "muted";
  /** Guidance when the device cannot be used yet. */
  help: string | null;
};

export function deviceStateInfo(state: string): DeviceStateInfo {
  switch (state) {
    case "device":
      return { label: "Ready", tone: "success", help: null };
    case "unauthorized":
      return {
        label: "Not authorized",
        tone: "warning",
        help: "Unlock the phone and accept the “Allow USB debugging?” prompt.",
      };
    case "offline":
      return {
        label: "Offline",
        tone: "danger",
        help: "Unlock the phone, or unplug and reconnect the cable.",
      };
    case "no permissions":
      return {
        label: "No permission",
        tone: "danger",
        help: "Your user cannot access this USB device. On Linux, install udev rules for Android (e.g. the android-udev package) and reconnect.",
      };
    case "authorizing":
    case "connecting":
      return { label: "Connecting…", tone: "warning", help: "Wait a moment." };
    case "recovery":
    case "sideload":
    case "bootloader":
    case "rescue":
      return {
        label: state[0].toUpperCase() + state.slice(1),
        tone: "muted",
        help: "Reboot the phone into Android to use it.",
      };
    default:
      return { label: state || "Unknown", tone: "muted", help: null };
  }
}
