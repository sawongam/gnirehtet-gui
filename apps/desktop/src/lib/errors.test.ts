import { describe, expect, it } from "vitest";
import { deviceStateInfo, errorHelp } from "./errors";

describe("errorHelp", () => {
  it("explains every code the backend produces", () => {
    const codes = [
      "ADB_MISSING",
      "ADB_PATH_INVALID",
      "ADB_TIMEOUT",
      "ADB_FAILED",
      "APK_MISSING",
      "INSTALL_FAILED",
      "INVALID_ARGUMENT",
      "DEVICE_UNAUTHORIZED",
      "DEVICE_OFFLINE",
      "DEVICE_NOT_FOUND",
      "MULTIPLE_DEVICES",
      "PORT_IN_USE",
      "RELAY_NOT_FOUND",
      "RELAY_START_FAILED",
      "RELAY_CRASHED",
      "TUNNEL_FAILED",
      "CLIENT_START_FAILED",
    ];
    for (const code of codes) {
      expect(errorHelp(code).title, code).not.toBe("Something went wrong");
    }
  });

  it("falls back for unknown codes", () => {
    expect(errorHelp("SOMETHING_NEW").title).toBe("Something went wrong");
  });
});

describe("deviceStateInfo", () => {
  it("marks only 'device' as ready", () => {
    expect(deviceStateInfo("device").tone).toBe("success");
    expect(deviceStateInfo("device").help).toBeNull();
    for (const state of ["unauthorized", "offline", "no permissions", "recovery"]) {
      expect(deviceStateInfo(state).help, state).not.toBeNull();
    }
    expect(deviceStateInfo("").label).toBe("Unknown");
  });
});
