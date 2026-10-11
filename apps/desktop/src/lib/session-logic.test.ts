import { describe, expect, it } from "vitest";
import { deviceName, formatDuration, pickSelection, sessionDeviceChange } from "./session-logic";
import type { DeviceInfo } from "./types";

const device = (serial: string, state = "device", model: string | null = null): DeviceInfo => ({
  serial,
  state,
  model,
  product: null,
});

describe("pickSelection", () => {
  it("keeps the session device selected", () => {
    expect(pickSelection([device("A")], "A", "B")).toBe("B");
  });

  it("keeps a still-present selection", () => {
    expect(pickSelection([device("A"), device("B")], "B", null)).toBe("B");
  });

  it("prefers the first ready device", () => {
    expect(pickSelection([device("A", "unauthorized"), device("B")], null, null)).toBe("B");
    expect(pickSelection([device("A", "offline")], "gone", null)).toBe("A");
    expect(pickSelection([], "gone", null)).toBeNull();
  });
});

describe("sessionDeviceChange", () => {
  it("detects the session device going away", () => {
    expect(sessionDeviceChange("connected", "A", [])).toBe("lost");
    expect(sessionDeviceChange("waiting", "A", [device("A", "offline")])).toBe("lost");
  });

  it("detects the session device coming back", () => {
    expect(sessionDeviceChange("interrupted", "A", [device("A")])).toBe("returned");
  });

  it("ignores devices outside of a session and other phases", () => {
    expect(sessionDeviceChange("idle", null, [])).toBeNull();
    expect(sessionDeviceChange("starting", "A", [])).toBeNull();
    expect(sessionDeviceChange("interrupted", "A", [])).toBeNull();
    expect(sessionDeviceChange("connected", "A", [device("A")])).toBeNull();
  });
});

describe("formatting", () => {
  it("formats durations", () => {
    expect(formatDuration(0)).toBe("0:00");
    expect(formatDuration(42_000)).toBe("0:42");
    expect(formatDuration(725_000)).toBe("12:05");
    expect(formatDuration(3_723_000)).toBe("1:02:03");
    expect(formatDuration(-5)).toBe("0:00");
  });

  it("names devices by model, product, then serial", () => {
    expect(deviceName(device("A", "device", "Pixel 8"))).toBe("Pixel 8");
    expect(deviceName({ ...device("A"), product: "oriole" })).toBe("oriole");
    expect(deviceName(device("A"))).toBe("A");
  });
});
