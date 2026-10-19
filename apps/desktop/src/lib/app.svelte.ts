import { api, isTauri, subscribe, toCommandError } from "./api";
import { settings } from "./settings.svelte";
import { pickSelection, sessionDeviceChange, isSessionPhase, type Phase } from "./session-logic";
import type {
  AdbInfo,
  ClientEvent,
  CommandError,
  DeviceInfo,
  LogEntry,
  RelayStatus,
} from "./types";

const DEVICE_POLL_MS = 2000;
/** While adb is missing, re-check it every Nth poll only. */
const ADB_RETRY_EVERY = 5;
const LOG_LIMIT = 2000;

const STOPPED: RelayStatus = { state: "stopped", port: null, pid: null, message: null };

class AppState {
  adb = $state<AdbInfo | null>(null);
  adbError = $state<CommandError | null>(null);
  devices = $state<DeviceInfo[]>([]);
  selectedSerial = $state<string | null>(null);
  relay = $state<RelayStatus>(STOPPED);

  phase = $state<Phase>("idle");
  sessionSerial = $state<string | null>(null);
  /** Last known details of the session device, kept while it is unplugged. */
  sessionDevice = $state<DeviceInfo | null>(null);
  /** Start of the current waiting/connected period, for elapsed-time display. */
  phaseSince = $state<number>(Date.now());

  error = $state<CommandError | null>(null);
  notice = $state<string | null>(null);
  task = $state<"install" | "repair" | null>(null);
  loaded = $state(false);
  logs = $state<LogEntry[]>([]);

  #pollTimer: ReturnType<typeof setInterval> | null = null;
  #polls = 0;
  #polling = false;
  #unsubscribe: (() => void) | null = null;

  get selectedDevice(): DeviceInfo | null {
    return this.devices.find((d) => d.serial === this.selectedSerial) ?? null;
  }

  get sessionActive(): boolean {
    return isSessionPhase(this.phase);
  }

  get busy(): boolean {
    return this.task !== null || this.phase === "starting" || this.phase === "stopping";
  }

  async start() {
    if (!isTauri()) {
      this.adbError = {
        code: "INTERNAL",
        message: "The interface is running outside the desktop app; start it with `pnpm tauri dev`.",
      };
      this.loaded = true;
      return;
    }
    this.#unsubscribe = await subscribe({
      onLog: (entry) => this.#onLog(entry),
      onRelayStatus: (status) => this.#onRelayStatus(status),
      onRelayClient: (event) => this.#onRelayClient(event),
    });
    await this.#restoreSession();
    await this.checkAdb();
    await this.refreshDevices();
    this.loaded = true;
    this.#pollTimer = setInterval(() => void this.#poll(), DEVICE_POLL_MS);
  }

  stop() {
    if (this.#pollTimer) clearInterval(this.#pollTimer);
    this.#pollTimer = null;
    this.#unsubscribe?.();
    this.#unsubscribe = null;
  }

  async checkAdb() {
    try {
      this.adb = await api.checkAdb();
      this.adbError = null;
    } catch (e) {
      this.adb = null;
      this.adbError = toCommandError(e);
    }
  }

  async refreshDevices() {
    if (!this.adb) return;
    try {
      this.#applyDevices(await api.listDevices());
    } catch (e) {
      const error = toCommandError(e);
      if (error.code === "ADB_MISSING" || error.code === "ADB_PATH_INVALID") {
        this.adb = null;
        this.adbError = error;
        this.#applyDevices([]);
      }
    }
  }

  select(serial: string) {
    if (!this.sessionActive) this.selectedSerial = serial;
  }

  async connect() {
    const serial = this.selectedSerial;
    if (!serial || this.busy) return;
    this.error = null;
    this.notice = null;
    this.phase = "starting";
    this.sessionSerial = serial;
    this.sessionDevice = this.selectedDevice;
    try {
      this.relay = await api.connect(serial, settings.connectOptions());
      // The VPN may already have connected while the command was finishing.
      if (this.phase === "starting") this.#setPhase("waiting");
    } catch (e) {
      this.error = toCommandError(e);
      await this.#restoreSession();
    }
  }

  async disconnect() {
    if (this.phase === "idle" || this.phase === "stopping") return;
    this.phase = "stopping";
    this.error = null;
    try {
      const status = await api.disconnect();
      this.relay = status;
      this.notice = status.message;
    } catch (e) {
      this.error = toCommandError(e);
    }
    this.phase = "idle";
    this.sessionSerial = null;
    this.sessionDevice = null;
  }

  async installClient() {
    const serial = this.selectedSerial;
    if (!serial || this.busy) return;
    this.task = "install";
    this.error = null;
    this.notice = null;
    try {
      await api.installClient(serial);
      this.notice = "The gnirehtet client was installed on the phone.";
    } catch (e) {
      this.error = toCommandError(e);
    } finally {
      this.task = null;
    }
  }

  async repairTunnel() {
    const serial = this.sessionSerial ?? this.selectedSerial;
    if (!serial || this.task) return;
    this.task = "repair";
    this.error = null;
    try {
      await api.repairTunnel(serial);
      if (this.phase === "interrupted") this.#setPhase("waiting");
    } catch (e) {
      this.error = toCommandError(e);
    } finally {
      this.task = null;
    }
  }

  clearLogs() {
    this.logs = [];
  }

  dismissMessages() {
    this.error = null;
    this.notice = null;
  }

  #setPhase(phase: Phase) {
    this.phase = phase;
    this.phaseSince = Date.now();
  }

  async #restoreSession() {
    try {
      const snapshot = await api.getSession();
      this.relay = snapshot.relay;
      this.sessionSerial = snapshot.serial;
      if (!snapshot.serial) {
        this.phase = "idle";
        this.sessionDevice = null;
      } else if (snapshot.relay.state !== "running") {
        this.phase = "failed";
      } else {
        this.#setPhase(snapshot.connectedClients > 0 ? "connected" : "waiting");
      }
      if (snapshot.serial) this.selectedSerial = snapshot.serial;
    } catch {
      this.phase = "idle";
      this.sessionSerial = null;
    }
  }

  async #poll() {
    if (this.#polling || document.hidden) return;
    this.#polling = true;
    try {
      if (!this.adb) {
        if (this.#polls++ % ADB_RETRY_EVERY !== 0) return;
        await this.checkAdb();
      }
      await this.refreshDevices();
    } finally {
      this.#polling = false;
    }
  }

  #applyDevices(devices: DeviceInfo[]) {
    this.devices = devices;
    this.selectedSerial = pickSelection(devices, this.selectedSerial, this.sessionSerial);
    const current = devices.find((d) => d.serial === this.sessionSerial);
    if (current) this.sessionDevice = current;

    const change = sessionDeviceChange(this.phase, this.sessionSerial, devices);
    if (change === "lost") {
      this.#setPhase("interrupted");
    } else if (change === "returned") {
      // Replugging drops the adb reverse tunnel; restoring it lets the
      // client on the phone reconnect by itself.
      void this.repairTunnel();
    }
  }

  #onLog(entry: LogEntry) {
    const logs = this.logs.length >= LOG_LIMIT ? this.logs.slice(-LOG_LIMIT + 1) : this.logs;
    this.logs = [...logs, entry];
  }

  #onRelayStatus(status: RelayStatus) {
    this.relay = status;
    if (status.state === "crashed" && this.sessionActive && this.phase !== "stopping") {
      this.phase = "failed";
      this.error = { code: "RELAY_CRASHED", message: status.message ?? "The relay exited." };
    }
  }

  #onRelayClient(event: ClientEvent) {
    if (!this.sessionActive || this.phase === "stopping") return;
    if (event.connected) {
      if (this.phase !== "connected") this.#setPhase("connected");
      this.error = null;
    } else if (event.connectedClients === 0 && this.phase === "connected") {
      // The phone's VPN went away (turned off on the phone, or cable pulled);
      // the client reconnects by itself if it can.
      this.#setPhase("waiting");
    }
  }
}

export const app = new AppState();
