import { describe, expect, it } from "vitest";
import { initialStartupState, StartupPreference, type StartupBackend, type StartupState } from "./startup";

function fixture(initial = false) {
  let enabled = initial;
  let failRead = false;
  let failWrite = false;
  let writes = 0;
  let state: StartupState = { ...initialStartupState };
  const backend: StartupBackend = {
    async isAutostartEnabled() {
      if (failRead) throw new Error("read denied");
      return enabled;
    },
    async setAutostart(next) {
      writes++;
      if (failWrite) throw new Error("write denied");
      enabled = next;
      if (failRead) throw new Error("read denied");
      return enabled;
    },
  };
  const preference = new StartupPreference(backend, (next) => (state = next));
  return {
    preference,
    get state() { return state; },
    get writes() { return writes; },
    failRead(value: boolean) { failRead = value; },
    failWrite(value: boolean) { failWrite = value; },
    externalChange(value: boolean) { enabled = value; },
  };
}

describe("startup preference", () => {
  it("reads the OS setting without changing it", async () => {
    const f = fixture(true);
    await f.preference.load();
    expect(f.state).toEqual({ enabled: true, loaded: true, busy: false, error: null });
    expect(f.writes).toBe(0);
  });

  it("enables and disables startup and reads back the OS state", async () => {
    const f = fixture();
    await f.preference.load();
    await f.preference.setEnabled(true);
    expect(f.state.enabled).toBe(true);
    await f.preference.setEnabled(false);
    expect(f.state.enabled).toBe(false);
    expect(f.writes).toBe(2);
  });

  it("does not overwrite an unknown or loading setting", async () => {
    const f = fixture();
    await f.preference.setEnabled(true);
    const loading = f.preference.load();
    await f.preference.setEnabled(true);
    await loading;
    expect(f.writes).toBe(0);
  });

  it("reports read errors and can reload a setting changed outside the app", async () => {
    const f = fixture();
    f.failRead(true);
    await f.preference.load();
    expect(f.state.loaded).toBe(false);
    expect(f.state.error).toContain("Could not read");
    expect(f.state.busy).toBe(false);
    f.failRead(false);
    f.externalChange(true);
    await f.preference.load();
    expect(f.state.enabled).toBe(true);
    expect(f.state.error).toBeNull();
  });

  it.each([false, true])("requires a fresh read after a failed change from %s", async (initial) => {
    const f = fixture(initial);
    await f.preference.load();
    f.failWrite(true);
    await f.preference.setEnabled(!initial);
    expect(f.state.enabled).toBe(initial);
    expect(f.state.loaded).toBe(false);
    expect(f.state.busy).toBe(false);
    expect(f.state.error).toContain("Could not update");
    await f.preference.setEnabled(!initial);
    expect(f.writes).toBe(1);
    f.failWrite(false);
    await f.preference.load();
    await f.preference.setEnabled(!initial);
    expect(f.state.enabled).toBe(!initial);
    expect(f.state.error).toBeNull();
  });

  it("does not show success when the OS state cannot be verified after a write", async () => {
    const f = fixture();
    await f.preference.load();
    f.failRead(true);
    await f.preference.setEnabled(true);
    expect(f.state.loaded).toBe(false);
    expect(f.state.error).not.toBeNull();
    f.failRead(false);
    await f.preference.load();
    expect(f.state.enabled).toBe(true);
  });

  it("ignores a second change while the first is in progress", async () => {
    const f = fixture();
    await f.preference.load();
    const enabling = f.preference.setEnabled(true);
    await f.preference.setEnabled(false);
    await enabling;
    expect(f.state.enabled).toBe(true);
    expect(f.writes).toBe(1);
  });
});
