<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import {
    ensureAdb,
    listDevices,
    startRelay,
    stopRelay,
    getRelayState,
    install,
    resetTunnel,
    runSession,
    stopClient,
    prepareQuit,
  } from "$lib/api/orchestrator";
  import type {
    AdbInfo,
    DeviceInfo,
    AppError,
    LogLine,
    RelayStatePayload,
    DeviceChangedPayload,
  } from "$lib/types/orchestrator";
  import {
    DEVICE_POLL_VISIBLE_MS,
    applyDeviceList,
    selectedDevice,
  } from "$lib/stores/devices";
  import {
    clearClientAccepted,
    clearSessionIntent,
    deriveSessionChip,
    initialSessionLayers,
    markClientAccepted,
    markTunnelFailed,
    markTunnelLost,
    markTunnelOk,
    markVpnIdle,
    markVpnPending,
    markVpnTimeout,
    relayLayerHealthy,
    tunnelLabel,
    vpnLabel,
    VPN_PENDING_TIMEOUT_MS,
    type SessionLayers,
  } from "$lib/stores/sessionLayers";
  import { parseRelayClientLogLine } from "$lib/stores/relayLogProbe";
  import {
    deviceStateClass,
    deviceStateLabel,
    deviceStateToUxCode,
    errorUxFor,
    type ErrorUxCopy,
  } from "$lib/errorUx";
  import Button from "$lib/components/ui/button.svelte";
  import Badge from "$lib/components/ui/badge.svelte";
  import Card from "$lib/components/ui/card.svelte";
  import Alert from "$lib/components/ui/alert.svelte";
  import Separator from "$lib/components/ui/separator.svelte";
  import Collapsible from "$lib/components/ui/collapsible.svelte";
  import ScrollArea from "$lib/components/ui/scroll-area.svelte";
  import Tooltip from "$lib/components/ui/tooltip.svelte";
  import Dialog from "$lib/components/ui/dialog.svelte";
  import Switch from "$lib/components/ui/switch.svelte";
  import AppShell from "$lib/components/shell/AppShell.svelte";
  import type { NavId } from "$lib/components/shell/types";
  import {
    Radio,
    Square,
    Plus,
    Download,
    ChevronRight,
    ChevronDown,
    MoreHorizontal,
    Usb,
    Zap,
    Lightbulb,
    Laptop,
    Smartphone,
    ArrowDownUp,
    Clock,
    Wrench,
    CircleCheck,
    AlertTriangle,
  } from "@lucide/svelte";

  let adb = $state<AdbInfo | null>(null);
  let adbErrorCode = $state<string | null>(null);
  let devices = $state<DeviceInfo[]>([]);
  let devicesError = $state<string | null>(null);
  let selectedSerial = $state<string | null>(null);
  let relay = $state<RelayStatePayload>({
    state: "relay_stopped",
    ownedBySession: false,
  });
  let logs = $state<LogLine[]>([]);
  let busy = $state(false);
  let stopping = $state(false);
  let actionError = $state<string | null>(null);
  let lastAppError = $state<AppError | null>(null);
  let pollBusy = $state(false);
  let layers = $state<SessionLayers>(initialSessionLayers());
  let vpnTick = $state(0);
  let advancedOpen = $state(false);
  let logsOpen = $state(false);
  let quitDialogOpen = $state(false);
  let quitPending = $state(false);
  let activeNav = $state<NavId>("dashboard");
  /** DEV preview-only: show Connected hero chrome (not real three-layer). */
  let connectedPreview = $state(false);
  /* Quick-action stubs — UI only, no behavior change */
  let stubAutoStartRelay = $state(true);
  let stubKeepAdb = $state(true);
  let stubVerboseLogs = $state(false);

  const LOG_CAP = 500;

  function errMsg(e: unknown): string {
    if (e && typeof e === "object" && "message" in e) {
      const m = (e as { code?: string; message: string }).message;
      const c = (e as { code?: string }).code;
      return c ? `[${c}] ${m}` : m;
    }
    return String(e);
  }

  function errCode(e: unknown): string | null {
    if (e && typeof e === "object" && "code" in e) {
      const c = (e as { code?: string }).code;
      return c ?? null;
    }
    return null;
  }

  function setDevices(next: DeviceInfo[]) {
    const applied = applyDeviceList(next, selectedSerial);
    const prevSerial = selectedSerial;
    devices = applied.devices;
    selectedSerial = applied.selectedSerial;

    // Device gone / not ready while tunnel was OK → TUNNEL_LOST (Interrupted).
    if (prevSerial && layers.tunnel === "ok" && layers.tunnelSerial === prevSerial) {
      const still = applied.devices.find((d) => d.serial === prevSerial);
      if (!still || still.adbState !== "device") {
        layers = markTunnelLost(layers);
        lastAppError = {
          code: "TUNNEL_LOST",
          message: "Device unplugged or not ready — adb reverse tunnel lost",
          serial: prevSerial,
        };
        actionError = `[TUNNEL_LOST] ${lastAppError.message}`;
      }
    }
  }

  async function refreshAdb() {
    try {
      adb = await ensureAdb();
      adbErrorCode = null;
    } catch (e) {
      adb = null;
      adbErrorCode = errCode(e) ?? "ADB_MISSING";
    }
  }

  async function refreshDevices() {
    devicesError = null;
    try {
      const next = await listDevices();
      setDevices(next);
    } catch (e) {
      setDevices([]);
      devicesError = errMsg(e);
      const code = errCode(e);
      if (code === "ADB_MISSING" || code === "ADB_PATH_INVALID") {
        adb = null;
        adbErrorCode = code;
      }
    }
  }

  async function refreshAll() {
    await refreshAdb();
    if (adb) {
      await refreshDevices();
    } else {
      setDevices([]);
    }
    await refreshRelay();
  }

  async function pollDevices() {
    if (pollBusy || busy || stopping) return;
    pollBusy = true;
    try {
      if (!adb) {
        await refreshAdb();
      }
      if (adb) {
        await refreshDevices();
      }
    } finally {
      pollBusy = false;
    }
  }

  async function refreshRelay() {
    try {
      relay = await getRelayState();
    } catch {
      /* ignore */
    }
  }

  const selected = $derived(selectedDevice(devices, selectedSerial));
  const deviceReady = $derived(!!selected && selected.adbState === "device");
  const adbOk = $derived(!!adb && !adbErrorCode);
  const canAct = $derived(adbOk && deviceReady && !busy && !stopping);
  const relayHealthy = $derived(
    relayLayerHealthy(relay.state, relay.ownedBySession),
  );

  const sessionChip = $derived(
    deriveSessionChip({
      layers,
      busy,
      stopping,
      relayHealthy,
    }),
  );

  const sessionActive = $derived(
    sessionChip === "Starting" ||
      sessionChip === "Waiting for VPN" ||
      sessionChip === "Interrupted" ||
      sessionChip === "Stopping" ||
      sessionChip === "Error" ||
      layers.vpn === "pending" ||
      (relay.ownedBySession && relayHealthy),
  );

  const interrupted = $derived(sessionChip === "Interrupted");
  const waitingVpn = $derived(sessionChip === "Waiting for VPN");
  // Re-evaluate VPN pending timeout when vpnTick advances.
  const vpnPendingExpired = $derived.by(() => {
    void vpnTick;
    if (layers.vpn !== "pending" || layers.vpnPendingSinceMs == null) return false;
    return Date.now() - layers.vpnPendingSinceMs >= VPN_PENDING_TIMEOUT_MS;
  });

  $effect(() => {
    if (vpnPendingExpired && layers.vpn === "pending") {
      layers = markVpnTimeout(layers);
      lastAppError = {
        code: "START_TIMEOUT",
        message:
          "VPN permission / handshake not confirmed in time (intent-sent ≠ connected)",
        serial: layers.vpnSerial,
      };
      actionError = `[START_TIMEOUT] ${lastAppError.message}`;
    }
  });

  async function onInstall() {
    if (!selectedSerial || !canAct) return;
    busy = true;
    actionError = null;
    try {
      await install(selectedSerial);
      logs = [
        ...logs,
        {
          timestampMs: Date.now(),
          level: "info",
          source: "ui",
          message: `install ok serial=${selectedSerial}`,
        },
      ].slice(-LOG_CAP);
    } catch (e) {
      actionError = errMsg(e);
      lastAppError = {
        code: errCode(e) ?? "INSTALL_FAILED",
        message: errMsg(e),
        serial: selectedSerial,
      };
    } finally {
      busy = false;
    }
  }

  async function onRepairTunnel() {
    if (!selectedSerial || !canAct) return;
    busy = true;
    actionError = null;
    try {
      await resetTunnel(selectedSerial);
      layers = markTunnelOk(layers, selectedSerial);
      actionError = null;
      if (lastAppError?.code === "TUNNEL_LOST" || lastAppError?.code === "TUNNEL_FAILED") {
        lastAppError = null;
      }
    } catch (e) {
      layers = markTunnelFailed(layers, selectedSerial);
      actionError = errMsg(e);
    } finally {
      busy = false;
    }
  }

  /** One-click ≈ run: owned relay + install-if-needed + tunnel + start intent. */
  async function onRun() {
    if (!selectedSerial || !canAct) return;
    busy = true;
    actionError = null;
    lastAppError = null;
    try {
      relay = await runSession({ serial: selectedSerial });
      // start/run includes tunnel — mark OK on success; VPN stays Pending (no handshake).
      layers = markTunnelOk(layers, selectedSerial);
      layers = markVpnPending(layers, selectedSerial, Date.now());
    } catch (e) {
      const code = errCode(e);
      actionError = errMsg(e);
      lastAppError = {
        code: code ?? "INTERNAL",
        message: errMsg(e),
        serial: selectedSerial,
      };
      if (code === "TUNNEL_FAILED") {
        layers = markTunnelFailed(layers, selectedSerial);
      } else if (code === "APK_MISSING" || code === "INSTALL_FAILED" || code === "CLIENT_START_FAILED") {
        layers = { ...layers, vpn: "error", lastCode: code, chipOverride: "Error" };
      }
    } finally {
      busy = false;
    }
  }

  /** Session-scoped stop: device client + clear owned relay if we started it. */
  async function onStop() {
    if (!selectedSerial || busy || stopping) return;
    if (!adbOk) return;
    stopping = true;
    actionError = null;
    try {
      try {
        await stopClient(selectedSerial);
      } catch (e) {
        actionError = errMsg(e);
      }
      if (relay.ownedBySession) {
        try {
          relay = await stopRelay();
        } catch (e) {
          actionError = actionError ? `${actionError}; ${errMsg(e)}` : errMsg(e);
        }
      }
      layers = clearSessionIntent(layers);
      if (lastAppError?.code === "VPN_PERMISSION_PENDING" || lastAppError?.code === "START_TIMEOUT") {
        lastAppError = null;
      }
    } finally {
      stopping = false;
    }
  }

  function onIveAllowedVpn() {
    // Honesty: we cannot confirm handshake — reset pending timer, stay Waiting (not Sharing).
    if (layers.vpn === "pending" && selectedSerial) {
      layers = markVpnPending(layers, selectedSerial, Date.now());
      actionError = null;
      lastAppError = {
        code: "VPN_PERMISSION_PENDING",
        message: "Still waiting — check the phone Connection request (not Sharing yet)",
        serial: selectedSerial,
      };
    }
  }

  async function onStartRelay() {
    busy = true;
    actionError = null;
    try {
      relay = await startRelay();
    } catch (e) {
      actionError = errMsg(e);
    } finally {
      busy = false;
    }
  }

  async function onStopRelay() {
    busy = true;
    actionError = null;
    try {
      relay = await stopRelay();
    } catch (e) {
      actionError = errMsg(e);
    } finally {
      busy = false;
    }
  }

  async function finishQuit() {
    quitPending = true;
    try {
      try {
        await prepareQuit(selectedSerial);
      } catch {
        /* best-effort; Exit handler is the safety net */
      }
      layers = clearSessionIntent(layers);
      busy = false;
      stopping = false;
      actionError = null;
      lastAppError = null;
      try {
        await getCurrentWindow().destroy();
      } catch {
        /* already closing */
      }
    } finally {
      quitPending = false;
      quitDialogOpen = false;
    }
  }

  /** DEV-only screenshot / layout previews via ?preview= */
  function applyDevPreview() {
    if (!import.meta.env.DEV) return;
    const preview = new URLSearchParams(location.search).get("preview");
    if (!preview) return;
    connectedPreview = false;
    if (preview === "adb-missing") {
      adb = null;
      adbErrorCode = "ADB_MISSING";
      devices = [];
      selectedSerial = null;
      relay = { state: "relay_stopped", ownedBySession: false };
      layers = initialSessionLayers();
      lastAppError = {
        code: "ADB_MISSING",
        message: "ADB not found",
      };
      actionError = null;
    } else if (preview === "idle") {
      // Idle empty — ADB ok, no devices (SaaS empty CTA)
      adb = { path: "/usr/bin/adb", version: "1.0.41", available: true };
      adbErrorCode = null;
      devices = [];
      selectedSerial = null;
      relay = { state: "relay_stopped", ownedBySession: false };
      layers = initialSessionLayers();
      lastAppError = null;
      actionError = null;
    } else if (preview === "idle-ready") {
      adb = { path: "/usr/bin/adb", version: "1.0.41", available: true };
      adbErrorCode = null;
      devices = [
        { serial: "3A7E1F2C", adbState: "device", model: "Pixel 8" },
      ];
      selectedSerial = "3A7E1F2C";
      relay = { state: "relay_stopped", ownedBySession: false };
      layers = initialSessionLayers();
      lastAppError = null;
      actionError = null;
    } else if (preview === "waiting-vpn") {
      adb = { path: "/usr/bin/adb", version: "1.0.41", available: true };
      adbErrorCode = null;
      devices = [
        { serial: "emulator-5554", adbState: "device", model: "Pixel Preview" },
      ];
      selectedSerial = "emulator-5554";
      relay = {
        state: "relay_running",
        ownedBySession: true,
        port: 31416,
      };
      layers = markTunnelOk(initialSessionLayers(), "emulator-5554");
      layers = markVpnPending(layers, "emulator-5554", Date.now());
      lastAppError = {
        code: "VPN_PERMISSION_PENDING",
        message: "Waiting for phone Connection request",
        serial: "emulator-5554",
      };
    } else if (preview === "connected") {
      // Layout-only Connected chrome — VPN Active not available in MVP.
      // Must never ship as real Sharing; gated behind DEV preview.
      connectedPreview = true;
      adb = { path: "/usr/bin/adb", version: "1.0.41", available: true };
      adbErrorCode = null;
      devices = [
        { serial: "3A7E1F2C", adbState: "device", model: "Pixel 8" },
      ];
      selectedSerial = "3A7E1F2C";
      relay = {
        state: "relay_running",
        ownedBySession: true,
        port: 31416,
      };
      layers = markTunnelOk(initialSessionLayers(), "3A7E1F2C");
      layers = markClientAccepted(layers, 1);
      lastAppError = null;
      actionError = null;
    } else if (preview === "interrupted") {
      adb = { path: "/usr/bin/adb", version: "1.0.41", available: true };
      adbErrorCode = null;
      devices = [
        { serial: "ABCD1234", adbState: "device", model: "Pixel Preview" },
      ];
      selectedSerial = "ABCD1234";
      relay = {
        state: "relay_running",
        ownedBySession: true,
        port: 31416,
      };
      layers = markTunnelLost(
        markTunnelOk(initialSessionLayers(), "ABCD1234"),
      );
      lastAppError = {
        code: "TUNNEL_LOST",
        message: "Device unplugged or not ready — adb reverse tunnel lost",
        serial: "ABCD1234",
      };
      actionError = `[TUNNEL_LOST] ${lastAppError.message}`;
    }
  }

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    let vpnTimer: ReturnType<typeof setInterval> | null = null;

    const previewActive =
      import.meta.env.DEV &&
      !!new URLSearchParams(location.search).get("preview");
    applyDevPreview();

    (async () => {
      // Browser / non-Tauri preview: skip IPC listeners.
      try {
        await getCurrentWindow();
      } catch {
        return;
      }
      if (previewActive) {
        // Keep staged preview for screenshots; skip live refresh/poll.
        return;
      }
      unlisteners.push(
        await listen<LogLine>("LogLine", (ev) => {
          logs = [...logs, ev.payload].slice(-LOG_CAP);
          // Owned-relay LogLine Client# probe — Tunnel chip copy only, never Sharing.
          if (ev.payload.source === "relay" && relay.ownedBySession) {
            const probe = parseRelayClientLogLine(ev.payload.message);
            if (probe?.kind === "connected") {
              layers = markClientAccepted(layers, probe.clientId);
            } else if (probe?.kind === "disconnected") {
              layers = clearClientAccepted(layers);
            }
          }
        }),
      );
      unlisteners.push(
        await listen<RelayStatePayload>("RelayState", (ev) => {
          relay = ev.payload;
          if (!ev.payload.ownedBySession) {
            layers = clearClientAccepted(layers);
          }
        }),
      );
      unlisteners.push(
        await listen<AppError>("Error", (ev) => {
          lastAppError = ev.payload;
          actionError = `[${ev.payload.code}] ${ev.payload.message}`;
          if (
            ev.payload.code === "ADB_MISSING" ||
            ev.payload.code === "ADB_PATH_INVALID"
          ) {
            adb = null;
            adbErrorCode = ev.payload.code;
          }
          if (ev.payload.code === "TUNNEL_FAILED") {
            layers = markTunnelFailed(layers, ev.payload.serial ?? selectedSerial);
          }
          if (ev.payload.code === "TUNNEL_LOST") {
            layers = markTunnelLost(layers);
          }
          if (ev.payload.code === "RELAY_CRASHED") {
            // Relay layer unhealthy — clear liveness; do not claim VPN/Sharing.
            layers = clearClientAccepted(markVpnIdle(layers));
          }
        }),
      );
      unlisteners.push(
        await listen<DeviceChangedPayload>("DeviceChanged", (ev) => {
          setDevices(ev.payload.devices);
          devicesError = null;
        }),
      );
      if (!cancelled) {
        await refreshAll();
        pollTimer = setInterval(() => {
          if (!cancelled) void pollDevices();
        }, DEVICE_POLL_VISIBLE_MS);
        vpnTimer = setInterval(() => {
          if (!cancelled) vpnTick = Date.now();
        }, 2000);
        // Quit-clean: confirm if session active, then prepare_quit + destroy.
        try {
          unlisteners.push(
            await getCurrentWindow().onCloseRequested(async (event) => {
              event.preventDefault();
              if (quitPending) return;
              if (sessionActive) {
                quitDialogOpen = true;
                return;
              }
              await finishQuit();
            }),
          );
        } catch {
          /* non-Tauri / check env — skip close hook */
        }
      }
    })();

    return () => {
      cancelled = true;
      if (pollTimer) clearInterval(pollTimer);
      if (vpnTimer) clearInterval(vpnTimer);
      unlisteners.forEach((u) => u());
    };
  });

  const relayLabel = $derived(
    relay.state.replace(/^relay_/, "").replace(/_/g, " "),
  );

  const adbBanner = $derived(adbErrorCode ? errorUxFor(adbErrorCode) : null);

  const authBanner = $derived.by(() => {
    const d = selected;
    if (!d) return null;
    const code = deviceStateToUxCode(d.adbState);
    return code ? errorUxFor(code) : null;
  });

  const emptyBanner = $derived(
    adb && !adbErrorCode && devices.length === 0 && !devicesError
      ? errorUxFor("NO_DEVICES")
      : null,
  );

  const vpnBanner = $derived.by(() => {
    if (layers.vpn === "pending") return errorUxFor("VPN_PERMISSION_PENDING");
    if (layers.lastCode === "START_TIMEOUT") return errorUxFor("START_TIMEOUT");
    if (layers.lastCode === "TUNNEL_LOST") return errorUxFor("TUNNEL_LOST");
    return null;
  });

  /** Blocking recovery codes shown in a prominent banner with CTAs (ERROR_UX). */
  const recoveryBanner = $derived.by(() => {
    if (adbBanner) return null;
    const code = lastAppError?.code;
    if (!code) return null;
    if (
      code === "VPN_PERMISSION_PENDING" ||
      code === "START_TIMEOUT" ||
      code === "TUNNEL_LOST" ||
      code === "DEVICE_UNAUTHORIZED" ||
      code === "DEVICE_OFFLINE"
    ) {
      return null;
    }
    const ux = errorUxFor(code);
    if (!ux) return null;
    if (
      code === "APK_MISSING" ||
      code === "INSTALL_FAILED" ||
      code === "CLIENT_START_FAILED" ||
      code === "RELAY_CRASHED" ||
      code === "RELAY_START_FAILED" ||
      code === "PORT_IN_USE" ||
      code === "TUNNEL_FAILED" ||
      code === "STOP_FAILED"
    ) {
      return ux;
    }
    return null;
  });

  const actionBanner = $derived.by(() => {
    if (!actionError || adbBanner || recoveryBanner) return null;
    const code = lastAppError?.code;
    if (code && (code === "VPN_PERMISSION_PENDING" || code === "START_TIMEOUT" || code === "TUNNEL_LOST")) {
      return null;
    }
    if (code) {
      const ux = errorUxFor(code);
      if (ux) return { kind: "ux" as const, ux, raw: actionError };
    }
    return { kind: "raw" as const, raw: actionError };
  });

  /** Single banner slot — SHELL_SCREENS priority. */
  type BannerSlot =
    | { kind: "adb"; ux: ErrorUxCopy }
    | { kind: "auth"; ux: ErrorUxCopy }
    | { kind: "recovery"; ux: ErrorUxCopy }
    | { kind: "vpn"; ux: ErrorUxCopy }
    | { kind: "empty"; ux: ErrorUxCopy }
    | { kind: "action"; payload: { kind: "ux"; ux: ErrorUxCopy; raw: string } | { kind: "raw"; raw: string } };

  const bannerSlot = $derived.by((): BannerSlot | null => {
    if (adbBanner) return { kind: "adb", ux: adbBanner };
    if (authBanner) return { kind: "auth", ux: authBanner };
    if (recoveryBanner) return { kind: "recovery", ux: recoveryBanner };
    if (vpnBanner && !adbBanner) return { kind: "vpn", ux: vpnBanner };
    if (emptyBanner) return { kind: "empty", ux: emptyBanner };
    if (actionBanner) return { kind: "action", payload: actionBanner };
    return null;
  });

  function chipTone(
    chip: string,
  ): "default" | "success" | "warning" | "danger" | "muted" {
    switch (chip) {
      case "Sharing":
        return "success";
      case "Waiting for VPN":
      case "Starting":
      case "Stopping":
        return "warning";
      case "Interrupted":
      case "Error":
        return "danger";
      case "Idle":
        return "muted";
      default:
        return "default";
    }
  }

  function deviceTone(
    cls: ReturnType<typeof deviceStateClass>,
  ): "success" | "warning" | "danger" | "muted" {
    switch (cls) {
      case "ok":
        return "success";
      case "warn":
        return "warning";
      case "err":
        return "danger";
      default:
        return "muted";
    }
  }

  function alertVariantFor(
    code: string | undefined,
  ): "default" | "warning" | "destructive" | "info" {
    if (!code) return "default";
    if (
      code === "VPN_PERMISSION_PENDING" ||
      code === "DEVICE_UNAUTHORIZED" ||
      code === "TUNNEL_LOST" ||
      code === "NO_DEVICES"
    ) {
      return "warning";
    }
    if (code.startsWith("ADB_") || code === "START_TIMEOUT" || code.includes("FAILED") || code.includes("CRASHED") || code === "PORT_IN_USE") {
      return "destructive";
    }
    return "default";
  }

  function selectDevice(serial: string) {
    selectedSerial = serial;
  }

  /**
   * Connected/Sharing hero ONLY when three-layer healthy.
   * MVP: Device VPN never reaches Active → real Connected never shows.
   * DEV ?preview=connected sets connectedPreview for layout screenshots only.
   */
  const showConnectedHero = $derived(connectedPreview);

  const serviceRunning = $derived(
    relayHealthy || sessionChip === "Waiting for VPN" || sessionChip === "Starting",
  );

  const adbFooterLabel = $derived(
    adbOk ? "ADB connected" : adbErrorCode ? "ADB missing" : "ADB …",
  );

  const greeting = $derived.by(() => {
    const h = new Date().getHours();
    if (h < 12) return "Good morning!";
    if (h < 18) return "Good afternoon!";
    return "Good to see you!";
  });

  function shortSerial(serial: string): string {
    if (serial.length <= 12) return serial;
    return `${serial.slice(0, 4)}-${serial.slice(4, 8)}-…`;
  }
</script>
<AppShell
  bind:activeNav
  adbOk={adbOk}
  adbLabel={adbFooterLabel}
  serviceRunning={serviceRunning}
  deviceCount={devices.filter((d) => d.adbState === "device").length}
  version="0.1.0"
>
  {#if activeNav === "dashboard"}
    <div class="mx-auto flex max-w-[1100px] flex-col gap-5 p-6 pb-10">
      <!-- Welcome -->
      <div>
        <h1 class="text-[28px] font-semibold tracking-tight text-fg">{greeting}</h1>
        <p class="mt-1 text-sm text-fg-body">
          Manage your reverse tethering setup and connected devices.
        </p>
      </div>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-[minmax(0,1.35fr)_minmax(300px,0.9fr)]">
        <!-- Main column -->
        <div class="flex min-w-0 flex-col gap-4">
          <!-- Connection hero -->
          {#if showConnectedHero}
            <Card class="border-success/25 bg-success-bg p-5">
              <div class="flex flex-wrap items-center gap-5">
                <div class="flex min-w-[140px] flex-col items-start gap-2">
                  <div
                    class="flex size-14 items-center justify-center rounded-full bg-success text-white shadow-sm"
                  >
                    <Radio class="size-7" strokeWidth={2.25} />
                  </div>
                  <div>
                    <p class="text-lg font-semibold text-success-strong">Connected</p>
                    <p class="text-sm text-fg-body">Internet tunnel active</p>
                    <p class="mt-1 text-[11px] text-fg-subtle">Preview chrome — not live Sharing</p>
                  </div>
                </div>

                <div class="flex min-w-0 flex-1 flex-col items-center gap-2 px-2">
                  <div class="flex w-full max-w-md items-center justify-between gap-3">
                    <div class="flex flex-col items-center gap-1.5">
                      <div
                        class="flex size-12 items-center justify-center rounded-[var(--radius-lg)] border border-border bg-bg-elevated"
                      >
                        <Laptop class="size-6 text-fg-muted" />
                      </div>
                      <span class="text-xs font-medium text-fg">Computer</span>
                      <span class="text-[11px] text-fg-subtle">(Your internet)</span>
                    </div>
                    <div class="flex flex-1 flex-col items-center gap-1">
                      <div class="h-px w-full border-t border-dashed border-success/50"></div>
                      <div
                        class="flex size-8 items-center justify-center rounded-full bg-success text-white"
                      >
                        <ArrowDownUp class="size-4" />
                      </div>
                    </div>
                    <div class="flex flex-col items-center gap-1.5">
                      <div
                        class="flex size-12 items-center justify-center rounded-[var(--radius-lg)] border border-border bg-bg-elevated"
                      >
                        <Smartphone class="size-6 text-fg-muted" />
                      </div>
                      <span class="text-xs font-medium text-fg">Android Device</span>
                      <span class="text-[11px] text-fg-subtle">(Uses computer's internet)</span>
                    </div>
                  </div>
                  <div class="mt-2 flex flex-wrap gap-2">
                    <Badge tone="success">
                      <span class="inline-block size-1.5 rounded-full bg-success"></span>
                      ADB
                    </Badge>
                    <Badge tone="success">
                      <span class="inline-block size-1.5 rounded-full bg-success"></span>
                      USB Bridge
                    </Badge>
                  </div>
                </div>

                <Button variant="destructive" size="default" onclick={onStop} disabled={busy || stopping}>
                  <Square class="size-3.5 fill-current" />
                  {stopping ? "Stopping…" : "Stop tethering"}
                </Button>
              </div>
            </Card>
          {:else if waitingVpn || sessionChip === "Starting"}
            <Card class="border-warning/30 bg-warning-bg p-5">
              <div class="flex flex-wrap items-start justify-between gap-4">
                <div class="flex gap-4">
                  <div
                    class="flex size-12 items-center justify-center rounded-full bg-warning-soft text-warning-strong"
                  >
                    <AlertTriangle class="size-6" />
                  </div>
                  <div>
                    <p class="text-lg font-semibold text-warning-strong">
                      {sessionChip === "Starting" ? "Starting…" : "Waiting for VPN"}
                    </p>
                    <p class="mt-0.5 text-sm text-fg-body">
                      Allow the Connection request on your phone — not Sharing yet.
                    </p>
                    <div class="mt-3 flex flex-wrap gap-2">
                      <Badge tone={relayHealthy ? "success" : "muted"}>
                        Relay · {relayHealthy ? (relay.port ? `:${relay.port}` : "Listening") : relayLabel}
                      </Badge>
                      <Badge
                        tone={layers.tunnel === "ok" || layers.clientAccepted ? "success" : "muted"}
                      >
                        Tunnel · {tunnelLabel(layers.tunnel, layers.clientAccepted)}
                      </Badge>
                      <Badge tone="warning">Device VPN · Waiting</Badge>
                    </div>
                  </div>
                </div>
                <div class="flex min-w-[200px] flex-col gap-2">
                  {#if waitingVpn}
                    <Button variant="outline" onclick={onIveAllowedVpn} disabled={busy}>
                      <CircleCheck class="size-4" />
                      I’ve allowed it
                    </Button>
                  {/if}
                  <Button variant="destructive" onclick={onStop} disabled={busy || stopping}>
                    <Square class="size-3.5 fill-current" />
                    {stopping ? "Stopping…" : "Stop"}
                  </Button>
                </div>
              </div>
            </Card>
          {:else if interrupted || sessionChip === "Error"}
            <Card class="border-danger/25 bg-danger-bg p-5">
              <div class="flex flex-wrap items-start justify-between gap-4">
                <div class="flex gap-4">
                  <div
                    class="flex size-12 items-center justify-center rounded-full bg-danger-soft text-danger"
                  >
                    <AlertTriangle class="size-6" />
                  </div>
                  <div>
                    <p class="text-lg font-semibold text-danger-strong">
                      {interrupted ? "Interrupted" : "Error"}
                    </p>
                    <p class="mt-0.5 text-sm text-fg-body">
                      {interrupted
                        ? "Tunnel lost — repair to continue."
                        : lastAppError
                          ? `[${lastAppError.code}] ${lastAppError.message}`
                          : "Something went wrong."}
                    </p>
                  </div>
                </div>
                <div class="flex min-w-[200px] flex-col gap-2">
                  {#if interrupted}
                    <Button variant="default" onclick={onRepairTunnel} disabled={!canAct}>
                      <Wrench class="size-4" />
                      Repair tunnel
                    </Button>
                  {/if}
                  <Button
                    variant="destructive"
                    onclick={onStop}
                    disabled={!adbOk || !selectedSerial || busy || stopping}
                  >
                    <Square class="size-3.5 fill-current" />
                    {stopping ? "Stopping…" : "Stop"}
                  </Button>
                </div>
              </div>
            </Card>
          {:else if devices.length === 0}
            <Card class="flex flex-col items-center gap-4 px-6 py-10 text-center">
              <div class="flex items-end justify-center gap-3 opacity-90" aria-hidden="true">
                <div class="rounded-lg border border-border bg-bg-muted px-3 py-2.5">
                  <Laptop class="size-8 text-fg-muted" />
                </div>
                <div class="mb-3 h-px w-8 border-t border-dashed border-accent/40"></div>
                <div class="rounded-xl border border-border bg-bg-muted px-2.5 py-3">
                  <Smartphone class="size-7 text-fg-muted" />
                </div>
              </div>
              <div>
                <h2 class="text-[15px] font-semibold text-fg">No device connected</h2>
                <p class="mx-auto mt-1 max-w-md text-sm text-fg-body">
                  Connect your Android device via USB and start tethering to use your computer's
                  internet connection.
                </p>
              </div>
              <Button variant="default" onclick={() => refreshAll()} disabled={busy}>
                <Plus class="size-4" />
                Connect device
              </Button>
              <div
                class="mt-1 flex max-w-md items-start gap-2 rounded-[var(--radius)] bg-warning-bg px-3 py-2 text-left text-xs text-fg-body"
              >
                <Lightbulb class="mt-0.5 size-3.5 shrink-0 text-warning" />
                <span>
                  Tip: Install the Gnirehtet client on the phone after USB debugging is authorized.
                </span>
              </div>
            </Card>
          {:else}
            <!-- Idle + device ready -->
            <Card class="p-5">
              <div class="flex flex-wrap items-start justify-between gap-4">
                <div class="flex gap-4">
                  <div
                    class="flex size-12 items-center justify-center rounded-full bg-accent-50 text-accent"
                  >
                    <Radio class="size-6" />
                  </div>
                  <div>
                    <div class="flex flex-wrap items-center gap-2">
                      <p class="text-lg font-semibold text-fg">Idle</p>
                      <Badge tone="muted">
                        <span class="inline-block size-1.5 rounded-full bg-fg-subtle"></span>
                        Session · Idle
                      </Badge>
                      <Tooltip content="Phone uses this PC’s internet — not the other way.">
                        <Badge tone="info">Internet: This PC → Phone</Badge>
                      </Tooltip>
                    </div>
                    <p class="mt-1 text-sm text-fg-body">Share this PC’s network with the selected device.</p>
                    <div class="mt-3 flex flex-wrap gap-2">
                      <Badge tone="muted">Relay · {relayLabel}</Badge>
                      <Badge tone="muted">Tunnel · {tunnelLabel(layers.tunnel, layers.clientAccepted)}</Badge>
                      <Badge tone="muted">Device VPN · {vpnLabel(layers.vpn)}</Badge>
                    </div>
                  </div>
                </div>
                <div class="flex min-w-[200px] flex-col gap-2">
                  <Button variant="default" onclick={onRun} disabled={!canAct}>
                    {busy ? "Starting…" : "Run"}
                  </Button>
                  <div class="flex gap-2">
                    <Button
                      variant="secondary"
                      size="sm"
                      class="flex-1"
                      onclick={onRepairTunnel}
                      disabled={!canAct}
                    >
                      <Wrench class="size-3.5" />
                      Repair
                    </Button>
                    <Button
                      variant="secondary"
                      size="sm"
                      class="flex-1"
                      onclick={onInstall}
                      disabled={!canAct}
                    >
                      <Download class="size-3.5" />
                      Install
                    </Button>
                  </div>
                </div>
              </div>
            </Card>
          {/if}

          <!-- Network Traffic (placeholder — no live counters) -->
          <Card class="p-5">
            <div class="flex flex-wrap items-center justify-between gap-2">
              <div class="flex items-center gap-2">
                <h2 class="text-[15px] font-semibold text-fg">Network Traffic</h2>
                <Badge tone="muted">Placeholder</Badge>
              </div>
              <span class="inline-flex items-center gap-1 text-xs text-fg-muted">
                Last 10 minutes
                <ChevronDown class="size-3.5" />
              </span>
            </div>
            <div class="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
              <div class="rounded-[var(--radius)] bg-bg-muted/80 px-3 py-3">
                <p class="text-xs text-fg-muted">Download</p>
                <p class="mt-1 font-mono text-xl font-semibold text-fg-subtle">—</p>
                <p class="text-xs text-fg-subtle">No live data</p>
              </div>
              <div class="rounded-[var(--radius)] bg-bg-muted/80 px-3 py-3">
                <p class="text-xs text-fg-muted">Upload</p>
                <p class="mt-1 font-mono text-xl font-semibold text-fg-subtle">—</p>
                <p class="text-xs text-fg-subtle">No live data</p>
              </div>
              <div class="rounded-[var(--radius)] bg-bg-muted/80 px-3 py-3">
                <p class="text-xs text-fg-muted">Total transferred</p>
                <p class="mt-1 font-mono text-xl font-semibold text-fg-subtle">—</p>
              </div>
              <div class="rounded-[var(--radius)] bg-bg-muted/80 px-3 py-3">
                <div class="flex items-center gap-1 text-xs text-fg-muted">
                  <Clock class="size-3" />
                  Session time
                </div>
                <p class="mt-1 font-mono text-xl font-semibold text-fg-subtle">—</p>
              </div>
            </div>
            <div
              class="mt-4 flex h-36 items-center justify-center rounded-[var(--radius)] border border-dashed border-border bg-bg-muted/50"
            >
              <p class="text-sm text-fg-muted">Traffic graphs deferred until real counters exist</p>
            </div>
          </Card>

          <!-- Banner slot -->
          {#if bannerSlot}
            {@const ux =
              bannerSlot.kind === "action"
                ? bannerSlot.payload.kind === "ux"
                  ? bannerSlot.payload.ux
                  : null
                : bannerSlot.ux}
            {@const code =
              ux?.code ?? (bannerSlot.kind === "action" ? lastAppError?.code : undefined)}
            <Alert
              variant={alertVariantFor(code)}
              role={bannerSlot.kind === "adb" || bannerSlot.kind === "recovery" ? "alert" : "status"}
            >
              {#if ux}
                <strong class="font-semibold">[{ux.code}] {ux.title}</strong>
                <p class="mt-1 text-sm text-fg">{ux.explanation}</p>
                <p class="mt-1 text-xs text-fg-muted">{ux.recoveryHint}</p>
              {:else if bannerSlot.kind === "action" && bannerSlot.payload.kind === "raw"}
                <p class="text-sm">{bannerSlot.payload.raw}</p>
              {/if}
              {#if bannerSlot.kind === "action" && bannerSlot.payload.kind === "ux"}
                <p class="mt-1 font-mono text-xs text-fg-subtle">{bannerSlot.payload.raw}</p>
              {/if}
              {#if bannerSlot.kind === "recovery" && actionError}
                <p class="mt-1 font-mono text-xs text-fg-subtle">{actionError}</p>
              {/if}

              {@const bannerHasCtas =
                bannerSlot.kind === "auth" ||
                bannerSlot.kind === "recovery" ||
                bannerSlot.kind === "empty" ||
                (bannerSlot.kind === "vpn" && ux?.code !== "VPN_PERMISSION_PENDING")}
              {#if bannerHasCtas}
                <div class="mt-3 flex flex-wrap gap-2">
                  {#if bannerSlot.kind === "auth"}
                    <Button variant="secondary" size="sm" onclick={() => refreshDevices()} disabled={busy}>
                      I’ve allowed it
                    </Button>
                  {:else if bannerSlot.kind === "vpn" && ux}
                    {#if ux.code === "TUNNEL_LOST"}
                      <Button variant="default" size="sm" onclick={onRepairTunnel} disabled={!canAct}>
                        Repair tunnel
                      </Button>
                      <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}
                        >Stop</Button
                      >
                    {:else}
                      <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}
                        >Retry Run</Button
                      >
                      <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}
                        >Stop</Button
                      >
                    {/if}
                  {:else if bannerSlot.kind === "recovery" && ux}
                    {#if ux.code === "APK_MISSING"}
                      <Button variant="default" size="sm" onclick={onInstall} disabled={!canAct}
                        >Retry Install</Button
                      >
                      <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}
                        >Refresh paths</Button
                      >
                    {:else if ux.code === "INSTALL_FAILED"}
                      <Button variant="default" size="sm" onclick={onInstall} disabled={!canAct}
                        >Reinstall helper</Button
                      >
                      <Button variant="secondary" size="sm" onclick={onRun} disabled={!canAct}
                        >Retry Run</Button
                      >
                    {:else if ux.code === "TUNNEL_FAILED"}
                      <Button variant="default" size="sm" onclick={onRepairTunnel} disabled={!canAct}
                        >Repair tunnel</Button
                      >
                      <Button variant="secondary" size="sm" onclick={onRun} disabled={!canAct}
                        >Retry Run</Button
                      >
                    {:else if ux.code === "RELAY_CRASHED" || ux.code === "RELAY_START_FAILED" || ux.code === "PORT_IN_USE"}
                      <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}
                        >Restart Run</Button
                      >
                      <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}
                        >Stop</Button
                      >
                    {:else}
                      <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}
                        >Retry Run</Button
                      >
                      <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}
                        >Stop</Button
                      >
                    {/if}
                  {:else if bannerSlot.kind === "empty"}
                    <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}
                      >Refresh devices</Button
                    >
                  {/if}
                </div>
              {/if}
            </Alert>
          {/if}
        </div>

        <!-- Right column widgets -->
        <div class="flex min-w-0 flex-col gap-4">
          <!-- Devices -->
          <Card class="p-4">
            <div class="mb-3 flex items-center justify-between gap-2">
              <div class="flex items-center gap-2">
                <h2 class="text-[15px] font-semibold text-fg">Devices</h2>
                {#if devices.filter((d) => d.adbState === "device").length > 0}
                  <Badge tone="success"
                    >{devices.filter((d) => d.adbState === "device").length} connected</Badge
                  >
                {/if}
              </div>
              <button
                type="button"
                class="text-xs font-medium text-accent hover:underline"
                onclick={() => (activeNav = "devices")}
              >
                View all →
              </button>
            </div>

            {#if devicesError && !adbBanner}
              <p class="text-sm text-danger">{devicesError}</p>
            {:else if devices.length === 0}
              <div
                class="rounded-[var(--radius)] border border-dashed border-border bg-bg-muted px-3 py-6 text-center"
              >
                <p class="text-sm text-fg-muted">No devices yet</p>
                <Button
                  variant="ghost"
                  size="sm"
                  class="mt-2"
                  onclick={() => refreshAll()}
                  disabled={busy}
                >
                  Refresh devices
                </Button>
              </div>
            {:else}
              <ul class="flex flex-col gap-2" role="listbox" aria-label="Device list">
                {#each devices as d}
                  <li>
                    <button
                      type="button"
                      role="option"
                      aria-selected={selectedSerial === d.serial}
                      class="w-full text-left"
                      onclick={() => selectDevice(d.serial)}
                    >
                      <div
                        class="flex items-start gap-3 rounded-[var(--radius)] border px-3 py-2.5 transition-colors"
                        class:border-accent={selectedSerial === d.serial}
                        class:bg-accent-50={selectedSerial === d.serial}
                        class:border-border={selectedSerial !== d.serial}
                        class:bg-bg-elevated={selectedSerial !== d.serial}
                      >
                        <div
                          class="flex size-9 shrink-0 items-center justify-center rounded-[var(--radius)] bg-bg-muted"
                        >
                          <Smartphone class="size-4 text-fg-muted" />
                        </div>
                        <div class="min-w-0 flex-1">
                          <div class="flex items-center gap-2">
                            <span class="truncate text-sm font-semibold text-fg"
                              >{d.model ?? d.serial}</span
                            >
                            <Badge tone={deviceTone(deviceStateClass(d.adbState))}
                              >{d.adbState === "device" ? "Online" : deviceStateLabel(d.adbState)}</Badge
                            >
                          </div>
                          <p class="mt-0.5 flex items-center gap-1.5 text-xs text-fg-muted">
                            {#if d.model}
                              <span>Android</span>
                              <span aria-hidden="true">·</span>
                            {/if}
                            <Usb class="size-3" />
                            <span class="font-mono">SN: {shortSerial(d.serial)}</span>
                          </p>
                        </div>
                        <MoreHorizontal class="size-4 shrink-0 text-fg-subtle" />
                      </div>
                    </button>
                  </li>
                {/each}
              </ul>
              {#if selected && deviceReady}
                <div class="mt-3 flex gap-2">
                  <Button
                    variant="destructive"
                    size="sm"
                    class="flex-1"
                    onclick={onStop}
                    disabled={!sessionActive || busy || stopping}
                  >
                    Disconnect
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="flex-1"
                    onclick={onRepairTunnel}
                    disabled={!canAct}
                  >
                    Restart tunnel
                    <ChevronDown class="size-3.5" />
                  </Button>
                </div>
              {/if}
            {/if}
          </Card>

          <!-- Install client CTA -->
          <button
            type="button"
            class="flex w-full items-center gap-3 rounded-[var(--radius-lg)] border border-accent/20 bg-accent-50 px-4 py-3.5 text-left transition-colors hover:bg-accent-100/80 disabled:opacity-50"
            onclick={onInstall}
            disabled={!canAct}
          >
            <div
              class="flex size-9 shrink-0 items-center justify-center rounded-[var(--radius)] bg-accent text-accent-fg"
            >
              <Download class="size-4" />
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-sm font-semibold text-fg">Install client</p>
              <p class="text-xs text-fg-body">Install the Gnirehtet client on your device</p>
            </div>
            <ChevronRight class="size-4 shrink-0 text-accent" />
          </button>

          <!-- How it works -->
          <Card class="p-4">
            <h2 class="text-[15px] font-semibold text-fg">How it works</h2>
            <ol class="mt-3 space-y-2.5 text-sm text-fg-body">
              <li class="flex gap-2.5">
                <span
                  class="flex size-5 shrink-0 items-center justify-center rounded-full bg-accent-50 text-[11px] font-semibold text-accent"
                  >1</span
                >
                <span>Plug in USB and authorize debugging</span>
              </li>
              <li class="flex gap-2.5">
                <span
                  class="flex size-5 shrink-0 items-center justify-center rounded-full bg-accent-50 text-[11px] font-semibold text-accent"
                  >2</span
                >
                <span>Install the helper client, then Run</span>
              </li>
              <li class="flex gap-2.5">
                <span
                  class="flex size-5 shrink-0 items-center justify-center rounded-full bg-accent-50 text-[11px] font-semibold text-accent"
                  >3</span
                >
                <span>Allow the VPN prompt on the phone</span>
              </li>
            </ol>
            <button
              type="button"
              class="mt-3 text-xs font-medium text-accent hover:underline"
              onclick={() => (activeNav = "settings")}
            >
              Learn more →
            </button>
          </Card>

          <!-- Quick actions (stubs) -->
          <Card class="p-4">
            <div class="mb-3 flex items-center gap-2">
              <Zap class="size-4 text-warning" />
              <h2 class="text-[15px] font-semibold text-fg">Quick actions</h2>
            </div>
            <ul class="flex flex-col gap-3">
              <li class="flex items-center justify-between gap-3">
                <span class="text-sm text-fg-body">Auto-start relay</span>
                <Switch bind:checked={stubAutoStartRelay} label="Auto-start relay (stub)" />
              </li>
              <li class="flex items-center justify-between gap-3">
                <span class="text-sm text-fg-body">Keep ADB connected</span>
                <Switch bind:checked={stubKeepAdb} label="Keep ADB connected (stub)" />
              </li>
              <li class="flex items-center justify-between gap-3">
                <span class="text-sm text-fg-body">Verbose logs</span>
                <Switch bind:checked={stubVerboseLogs} label="Verbose logs (stub)" />
              </li>
            </ul>
            <p class="mt-3 text-[11px] text-fg-subtle">Stubs only — no settings persistence yet.</p>
          </Card>

          <!-- Advanced relay -->
          <Card class="p-4">
            <Collapsible bind:open={advancedOpen}>
              {#snippet trigger({ open, toggle })}
                <Button variant="ghost" size="sm" class="px-1" onclick={toggle}>
                  {open ? "▾" : "▸"} Advanced
                </Button>
              {/snippet}
              <div class="mt-2 flex flex-wrap gap-2">
                <Button variant="secondary" size="sm" onclick={onStartRelay} disabled={busy || stopping}>
                  Start Relay
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onclick={onStopRelay}
                  disabled={busy || stopping || !relay.ownedBySession}
                >
                  Stop Relay
                </Button>
              </div>
              {#if relay.message}
                <p class="mt-1.5 text-xs text-fg-muted">{relay.message}</p>
              {/if}
            </Collapsible>
          </Card>
        </div>
      </div>
    </div>
  {:else if activeNav === "devices"}
    <div class="mx-auto max-w-[900px] p-6">
      <h1 class="text-[28px] font-semibold tracking-tight text-fg">Devices</h1>
      <p class="mt-1 text-sm text-fg-body">Select a device, then return to Dashboard to Run.</p>
      <Card class="mt-5 p-4">
        <div class="mb-3 flex justify-end">
          <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}
            >Refresh devices</Button
          >
        </div>
        {#if devices.length === 0}
          <p class="py-8 text-center text-sm text-fg-muted">No devices connected.</p>
        {:else}
          <ul class="flex flex-col gap-2">
            {#each devices as d}
              <li>
                <button type="button" class="w-full text-left" onclick={() => selectDevice(d.serial)}>
                  <Card selected={selectedSerial === d.serial} class="flex items-center gap-3 px-3.5 py-3">
                    <Smartphone class="size-4 text-fg-muted" />
                    <div class="min-w-0 flex-1">
                      <code class="block truncate font-mono text-sm font-medium">{d.serial}</code>
                      {#if d.model}
                        <span class="text-xs text-fg-muted">{d.model}</span>
                      {/if}
                    </div>
                    <Badge tone={deviceTone(deviceStateClass(d.adbState))}
                      >{deviceStateLabel(d.adbState)}</Badge
                    >
                  </Card>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
        <div class="mt-4 flex flex-wrap gap-2">
          <Button variant="default" onclick={onRun} disabled={!canAct}>{busy ? "Starting…" : "Run"}</Button>
          <Button
            variant="destructive"
            onclick={onStop}
            disabled={!adbOk || !selectedSerial || busy || stopping}
          >
            Stop
          </Button>
          <Button variant="secondary" onclick={onRepairTunnel} disabled={!canAct}>Repair tunnel</Button>
          <Button variant="secondary" onclick={onInstall} disabled={!canAct}>Install helper</Button>
        </div>
      </Card>
    </div>
  {:else if activeNav === "traffic"}
    <div class="mx-auto max-w-[900px] p-6">
      <h1 class="text-[28px] font-semibold tracking-tight text-fg">Traffic</h1>
      <Card class="mt-5 flex h-48 items-center justify-center p-6">
        <p class="text-sm text-fg-muted">Live traffic counters are not available in MVP.</p>
      </Card>
    </div>
  {:else if activeNav === "logs"}
    <div class="mx-auto max-w-[900px] p-6">
      <div class="flex items-center justify-between gap-2">
        <h1 class="text-[28px] font-semibold tracking-tight text-fg">Logs</h1>
        <Button variant="ghost" size="sm" onclick={() => (logs = [])}>Clear</Button>
      </div>
      <Card class="mt-5 p-4">
        <ScrollArea
          class="log-pane max-h-[480px] rounded-[var(--radius)] border border-border-subtle bg-bg-muted px-3 py-2.5 text-xs leading-relaxed"
        >
          <pre class="m-0 whitespace-pre-wrap break-words font-mono">{#each logs as line}{new Date(line.timestampMs).toISOString()} [{line.level}] {line.source}: {line.message}
{/each}</pre>
        </ScrollArea>
      </Card>
    </div>
  {:else}
    <div class="mx-auto max-w-[900px] p-6">
      <h1 class="text-[28px] font-semibold tracking-tight text-fg">Settings</h1>
      <Card class="mt-5 space-y-3 p-5">
        <p class="text-sm text-fg-body">
          Settings stubs only. ADB path and preferences stay orchestrator-owned for MVP.
        </p>
        <Separator />
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm">Auto-start relay</span>
          <Switch bind:checked={stubAutoStartRelay} label="Auto-start relay (stub)" />
        </div>
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm">Keep ADB connected</span>
          <Switch bind:checked={stubKeepAdb} label="Keep ADB connected (stub)" />
        </div>
        <div class="flex items-center justify-between gap-3">
          <span class="text-sm">Verbose logs</span>
          <Switch bind:checked={stubVerboseLogs} label="Verbose logs (stub)" />
        </div>
      </Card>
    </div>
  {/if}
</AppShell>

<Dialog
  bind:open={quitDialogOpen}
  title="Stop sharing and quit?"
  description="An active session will be torn down (stop client + clear owned relay) before the window closes."
  confirmLabel={quitPending ? "Quitting…" : "Stop and quit"}
  cancelLabel="Cancel"
  confirmVariant="destructive"
  onConfirm={finishQuit}
/>
