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
  let logsOpen = $state(true);
  let quitDialogOpen = $state(false);
  let quitPending = $state(false);

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
    if (preview === "waiting-vpn") {
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
</script>

<main class="mx-auto flex min-h-screen max-w-[920px] flex-col gap-3 p-4 pb-6">
  <!-- Title / chrome -->
  <header class="flex flex-wrap items-center justify-between gap-2">
    <div class="flex flex-wrap items-center gap-2">
      <h1 class="text-lg font-semibold tracking-tight text-fg">gnirehtet-gui</h1>
      <Tooltip content="Phone uses this PC’s internet — not the other way.">
        <Badge tone="info">Internet: This PC → Phone</Badge>
      </Tooltip>
    </div>
    <div class="flex flex-wrap items-center gap-2">
      <Badge tone={chipTone(sessionChip)}>Session · {sessionChip}</Badge>
      {#if adb}
        <Badge tone="success">ADB · ok</Badge>
        <span class="font-mono text-[12px] text-fg-subtle">{adb.version}</span>
      {:else if adbErrorCode}
        <Badge tone="danger">ADB · {adbErrorCode}</Badge>
      {:else}
        <Badge tone="muted">ADB · …</Badge>
      {/if}
    </div>
  </header>

  <!-- Three-layer strip -->
  <section
    class="grid grid-cols-1 gap-2 sm:grid-cols-3"
    aria-label="Three-layer status"
  >
    <Card class="border-l-[3px] border-l-layer-relay px-3 py-2">
      <div class="flex items-center justify-between gap-2">
        <Tooltip content="PC process that phones connect to.">
          <span class="text-[12px] font-semibold uppercase tracking-wide text-fg-subtle"
            >Relay</span
          >
        </Tooltip>
        <span
          class="text-sm font-medium"
          class:text-success={relayHealthy}
          class:text-danger={relay.state === "relay_error" || relay.state === "relay_exited"}
          class:text-warning={relay.state === "relay_starting"}
          class:text-fg-muted={!relayHealthy && relay.state !== "relay_error" && relay.state !== "relay_exited" && relay.state !== "relay_starting"}
        >
          {relayHealthy ? "Listening" : relayLabel}
        </span>
      </div>
      {#if relay.port}
        <p class="mt-0.5 font-mono text-[12px] text-fg-subtle">:{relay.port}</p>
      {/if}
    </Card>

    <Card class="border-l-[3px] border-l-layer-tunnel px-3 py-2">
      <div class="flex items-center justify-between gap-2">
        <Tooltip content="USB reverse path from phone to this PC.">
          <span class="text-[12px] font-semibold uppercase tracking-wide text-fg-subtle"
            >Tunnel</span
          >
        </Tooltip>
        <span
          class="text-sm font-medium"
          class:text-success={layers.tunnel === "ok" || layers.clientAccepted}
          class:text-danger={layers.tunnel === "lost" || layers.tunnel === "failed"}
          class:text-fg-muted={layers.tunnel === "unknown" && !layers.clientAccepted}
        >
          {tunnelLabel(layers.tunnel, layers.clientAccepted)}
        </span>
      </div>
      {#if layers.clientAccepted && layers.clientId != null}
        <p class="mt-0.5 font-mono text-[12px] text-fg-subtle">#{layers.clientId}</p>
      {/if}
    </Card>

    <Card class="border-l-[3px] border-l-layer-vpn px-3 py-2">
      <div class="flex items-center justify-between gap-2">
        <Tooltip content="Phone VPN permission + helper handshake.">
          <span class="text-[12px] font-semibold uppercase tracking-wide text-fg-subtle"
            >Device VPN</span
          >
        </Tooltip>
        <span
          class="text-sm font-medium"
          class:text-warning={layers.vpn === "pending"}
          class:text-danger={layers.vpn === "error"}
          class:text-fg-muted={layers.vpn === "idle"}
        >
          {vpnLabel(layers.vpn)}
        </span>
      </div>
      {#if layers.vpn === "pending"}
        <p class="mt-0.5 text-[12px] text-fg-muted">Waiting for phone to finish connecting</p>
      {/if}
    </Card>
  </section>

  <!-- Body: devices + session -->
  <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
    <Card class="flex flex-col gap-2 p-3">
      <div class="flex items-center justify-between gap-2">
        <h2 class="text-base font-semibold text-fg">Devices</h2>
        <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}>
          Refresh devices
        </Button>
      </div>

      {#if devicesError && !adbBanner}
        <p class="text-sm text-danger">{devicesError}</p>
      {:else if devices.length === 0}
        <div class="rounded-md border border-dashed border-border bg-bg-muted px-3 py-6 text-center">
          <p class="text-sm text-fg-muted">
            No devices. Plug in USB, enable debugging, unlock, and accept Allow USB debugging.
          </p>
          <p class="mt-1 text-[12px] text-fg-subtle">
            List polls every {DEVICE_POLL_VISIBLE_MS / 1000}s while visible.
          </p>
        </div>
      {:else}
        <ul class="flex flex-col gap-1.5" role="listbox" aria-label="Device list">
          {#each devices as d}
            <li>
              <button
                type="button"
                role="option"
                aria-selected={selectedSerial === d.serial}
                class="w-full text-left"
                onclick={() => selectDevice(d.serial)}
              >
                <Card
                  selected={selectedSerial === d.serial}
                  class="flex items-center gap-2 px-2.5 py-2 hover:bg-bg-muted/80"
                >
                  <code class="font-mono text-[13px] text-fg">{d.serial}</code>
                  <Badge tone={deviceTone(deviceStateClass(d.adbState))}
                    >{deviceStateLabel(d.adbState)}</Badge
                  >
                  {#if d.model}
                    <span class="ml-auto truncate text-[12px] text-fg-muted">{d.model}</span>
                  {/if}
                </Card>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </Card>

    <Card class="flex flex-col gap-3 p-3">
      <h2 class="text-base font-semibold text-fg">Session</h2>

      <!-- Primary CTA hierarchy -->
      <div class="flex flex-col gap-2">
        {#if interrupted}
          <Button
            variant="default"
            size="lg"
            class="w-full"
            onclick={onRepairTunnel}
            disabled={!canAct}
          >
            Repair tunnel
          </Button>
          <Button variant="destructive" onclick={onStop} disabled={!adbOk || !selectedSerial || busy || stopping}>
            {stopping ? "Stopping…" : "Stop"}
          </Button>
        {:else if waitingVpn}
          <Button variant="secondary" size="lg" class="w-full" onclick={onIveAllowedVpn} disabled={busy}>
            I’ve allowed it
          </Button>
          <Button variant="destructive" onclick={onStop} disabled={busy || stopping}>
            {stopping ? "Stopping…" : "Stop"}
          </Button>
        {:else if sessionChip === "Starting"}
          <Button variant="default" size="lg" class="w-full" disabled>
            Starting…
          </Button>
          <Button
            variant="destructive"
            onclick={onStop}
            disabled={!adbOk || !selectedSerial || stopping}
          >
            Stop
          </Button>
        {:else if sessionChip === "Stopping"}
          <Button variant="destructive" size="lg" class="w-full" disabled>
            Stopping…
          </Button>
        {:else}
          <Button
            variant="default"
            size="lg"
            class="w-full"
            onclick={onRun}
            disabled={!canAct}
          >
            {busy ? "Starting…" : "Run"}
          </Button>
          <p class="text-[12px] text-fg-muted">Share this PC’s network</p>
          {#if relay.ownedBySession || layers.vpn === "pending"}
            <Button
              variant="destructive"
              onclick={onStop}
              disabled={!adbOk || !selectedSerial || busy || stopping}
            >
              {stopping ? "Stopping…" : "Stop"}
            </Button>
          {/if}
        {/if}
      </div>

      <div class="flex flex-wrap gap-2">
        {#if !interrupted}
          <Button variant="secondary" size="sm" onclick={onRepairTunnel} disabled={!canAct}>
            Repair tunnel
          </Button>
        {/if}
        <Button variant="ghost" size="sm" onclick={onInstall} disabled={!canAct}>
          Install helper
        </Button>
      </div>

      <Separator />

      <Collapsible bind:open={advancedOpen}>
        {#snippet trigger({ open, toggle })}
          <Button variant="ghost" size="sm" class="px-1" onclick={toggle}>
            {open ? "▾" : "▸"} Advanced
          </Button>
        {/snippet}
        <div class="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onclick={onStartRelay} disabled={busy || stopping}>
            Start Relay
          </Button>
          <Button
            variant="outline"
            size="sm"
            onclick={onStopRelay}
            disabled={busy || stopping || !relay.ownedBySession}
          >
            Stop Relay
          </Button>
        </div>
        {#if relay.message}
          <p class="mt-1.5 text-[12px] text-fg-muted">{relay.message}</p>
        {/if}
      </Collapsible>
    </Card>
  </div>

  <!-- Single contextual banner -->
  {#if bannerSlot}
    {@const ux =
      bannerSlot.kind === "action"
        ? bannerSlot.payload.kind === "ux"
          ? bannerSlot.payload.ux
          : null
        : bannerSlot.ux}
    {@const code = ux?.code ?? (bannerSlot.kind === "action" ? lastAppError?.code : undefined)}
    <Alert variant={alertVariantFor(code)} role={bannerSlot.kind === "adb" || bannerSlot.kind === "recovery" ? "alert" : "status"}>
      {#if ux}
        <strong class="font-semibold">[{ux.code}] {ux.title}</strong>
        <p class="mt-1 text-sm text-fg">{ux.explanation}</p>
        <p class="mt-0.5 text-[12px] text-fg-muted">{ux.recoveryHint}</p>
      {:else if bannerSlot.kind === "action" && bannerSlot.payload.kind === "raw"}
        <p class="text-sm">{bannerSlot.payload.raw}</p>
      {/if}
      {#if bannerSlot.kind === "action" && bannerSlot.payload.kind === "ux"}
        <p class="mt-0.5 font-mono text-[12px] text-fg-subtle">{bannerSlot.payload.raw}</p>
      {/if}
      {#if bannerSlot.kind === "recovery" && actionError}
        <p class="mt-0.5 font-mono text-[12px] text-fg-subtle">{actionError}</p>
      {/if}

      <div class="mt-2 flex flex-wrap gap-2">
        {#if bannerSlot.kind === "auth"}
          <Button variant="secondary" size="sm" onclick={() => refreshDevices()} disabled={busy}>
            I’ve allowed it
          </Button>
        {:else if bannerSlot.kind === "vpn" && ux}
          {#if ux.code === "VPN_PERMISSION_PENDING"}
            <Button variant="secondary" size="sm" onclick={onIveAllowedVpn} disabled={busy}>
              I’ve allowed it
            </Button>
            <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}>Stop</Button>
          {:else if ux.code === "TUNNEL_LOST"}
            <Button variant="default" size="sm" onclick={onRepairTunnel} disabled={!canAct}>
              Repair tunnel
            </Button>
            <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}>Stop</Button>
          {:else}
            <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}>Retry Run</Button>
            <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}>Stop</Button>
          {/if}
        {:else if bannerSlot.kind === "recovery" && ux}
          {#if ux.code === "APK_MISSING"}
            <Button variant="default" size="sm" onclick={onInstall} disabled={!canAct}>Retry Install</Button>
            <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}>Refresh paths</Button>
          {:else if ux.code === "INSTALL_FAILED"}
            <Button variant="default" size="sm" onclick={onInstall} disabled={!canAct}>Reinstall helper</Button>
            <Button variant="secondary" size="sm" onclick={onRun} disabled={!canAct}>Retry Run</Button>
          {:else if ux.code === "TUNNEL_FAILED"}
            <Button variant="default" size="sm" onclick={onRepairTunnel} disabled={!canAct}>Repair tunnel</Button>
            <Button variant="secondary" size="sm" onclick={onRun} disabled={!canAct}>Retry Run</Button>
          {:else if ux.code === "RELAY_CRASHED" || ux.code === "RELAY_START_FAILED" || ux.code === "PORT_IN_USE"}
            <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}>Restart Run</Button>
            <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}>Stop</Button>
          {:else}
            <Button variant="default" size="sm" onclick={onRun} disabled={!canAct}>Retry Run</Button>
            <Button variant="destructive" size="sm" onclick={onStop} disabled={busy || stopping}>Stop</Button>
          {/if}
        {:else if bannerSlot.kind === "empty"}
          <Button variant="ghost" size="sm" onclick={() => refreshAll()} disabled={busy}>Refresh devices</Button>
        {/if}
      </div>
    </Alert>
  {/if}

  <!-- Logs -->
  <Card class="p-3">
    <Collapsible bind:open={logsOpen}>
      {#snippet trigger({ open, toggle })}
        <div class="flex items-center justify-between gap-2">
          <Button variant="ghost" size="sm" class="px-1" onclick={toggle}>
            {open ? "▾" : "▸"} Logs
          </Button>
          <Button variant="ghost" size="sm" onclick={() => (logs = [])}>Clear</Button>
        </div>
      {/snippet}
      <ScrollArea class="log-pane mt-2 max-h-[240px] rounded-md border border-border-subtle bg-bg-muted px-2.5 py-2 text-[12px] leading-relaxed text-fg">
        <pre class="m-0 whitespace-pre-wrap break-words font-mono">{#each logs as line}{new Date(line.timestampMs).toISOString()} [{line.level}] {line.source}: {line.message}
{/each}</pre>
      </ScrollArea>
    </Collapsible>
  </Card>
</main>

<Dialog
  bind:open={quitDialogOpen}
  title="Stop sharing and quit?"
  description="An active session will be torn down (stop client + clear owned relay) before the window closes."
  confirmLabel={quitPending ? "Quitting…" : "Stop and quit"}
  cancelLabel="Cancel"
  confirmVariant="destructive"
  onConfirm={finishQuit}
/>
