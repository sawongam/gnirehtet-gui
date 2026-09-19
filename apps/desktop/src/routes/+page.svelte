<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
    clearSessionIntent,
    deriveSessionChip,
    initialSessionLayers,
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
  import {
    deviceStateClass,
    deviceStateLabel,
    deviceStateToUxCode,
    errorUxFor,
  } from "$lib/errorUx";

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

  onMount(() => {
    const unlisteners: UnlistenFn[] = [];
    let cancelled = false;
    let pollTimer: ReturnType<typeof setInterval> | null = null;
    let vpnTimer: ReturnType<typeof setInterval> | null = null;

    (async () => {
      unlisteners.push(
        await listen<LogLine>("LogLine", (ev) => {
          logs = [...logs, ev.payload].slice(-LOG_CAP);
        }),
      );
      unlisteners.push(
        await listen<RelayStatePayload>("RelayState", (ev) => {
          relay = ev.payload;
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
            // Relay layer unhealthy — do not claim VPN/Sharing.
            layers = markVpnIdle(layers);
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

  const actionBanner = $derived.by(() => {
    if (!actionError || adbBanner) return null;
    const code = lastAppError?.code;
    if (code && (code === "VPN_PERMISSION_PENDING" || code === "START_TIMEOUT" || code === "TUNNEL_LOST")) {
      return null; // dedicated banners
    }
    if (code) {
      const ux = errorUxFor(code);
      if (ux) return { kind: "ux" as const, ux, raw: actionError };
    }
    return { kind: "raw" as const, raw: actionError };
  });
</script>

<main class="app">
  <header>
    <h1>gnirehtet-gui</h1>
    <p class="sub">
      Phase 2 — install / tunnel / run / stop for one device. Three layers;
      never claim Sharing without handshake.
    </p>
  </header>

  <section class="status-bar" aria-live="polite">
    <div class="chip">
      <span class="label">Session</span>
      <span
        class:ok={sessionChip === "Idle"}
        class:warn={sessionChip === "Waiting for VPN" || sessionChip === "Interrupted" || sessionChip === "Starting" || sessionChip === "Stopping"}
        class:err={sessionChip === "Error"}>{sessionChip}</span
      >
    </div>
    <div class="chip">
      <span class="label">ADB</span>
      {#if adb}
        <span class="ok">ok</span>
        <span class="muted">{adb.version}</span>
      {:else if adbErrorCode}
        <span class="err">{adbErrorCode}</span>
      {:else}
        <span class="muted">…</span>
      {/if}
    </div>
    <div class="chip">
      <span class="label">Device</span>
      {#if selected}
        <code>{selected.serial}</code>
        <span class={deviceStateClass(selected.adbState)}
          >{deviceStateLabel(selected.adbState)}</span
        >
      {:else}
        <span class="muted">none selected</span>
      {/if}
    </div>
  </section>

  <section class="layers" aria-label="Three-layer status">
    <div class="layer">
      <span class="label">Relay</span>
      <span
        class:ok={relayHealthy}
        class:err={relay.state === "relay_error" || relay.state === "relay_exited"}
        class:warn={relay.state === "relay_starting"}
        >{relayHealthy ? "Listening" : relayLabel}</span
      >
      {#if relay.port}<span class="muted">:{relay.port}</span>{/if}
      {#if relay.ownedBySession}<span class="ok">owned</span>{/if}
    </div>
    <div class="layer">
      <span class="label">Tunnel</span>
      <span
        class:ok={layers.tunnel === "ok"}
        class:err={layers.tunnel === "lost" || layers.tunnel === "failed"}
        class:muted={layers.tunnel === "unknown"}>{tunnelLabel(layers.tunnel)}</span
      >
    </div>
    <div class="layer">
      <span class="label">Device VPN</span>
      <span
        class:warn={layers.vpn === "pending"}
        class:err={layers.vpn === "error"}
        class:muted={layers.vpn === "idle"}>{vpnLabel(layers.vpn)}</span
      >
      <span class="muted tip">Active only after handshake — not claimed</span>
    </div>
  </section>

  {#if adbBanner}
    <div class="banner err" role="alert">
      <strong>[{adbBanner.code}] {adbBanner.title}</strong>
      <p>{adbBanner.explanation}</p>
      <p class="muted">{adbBanner.recoveryHint}</p>
    </div>
  {/if}
  {#if authBanner}
    <div class="banner warn" role="status">
      <strong>[{authBanner.code}] {authBanner.title}</strong>
      <p>{authBanner.explanation}</p>
      <div class="actions tight">
        <button type="button" onclick={() => refreshDevices()} disabled={busy}>
          I’ve allowed it (refresh)
        </button>
      </div>
    </div>
  {/if}
  {#if emptyBanner}
    <div class="banner" role="status">
      <strong>[{emptyBanner.code}] {emptyBanner.title}</strong>
      <p>{emptyBanner.explanation}</p>
      <p class="muted">{emptyBanner.recoveryHint}</p>
    </div>
  {/if}
  {#if vpnBanner && !adbBanner}
    <div
      class="banner"
      class:warn={vpnBanner.code === "VPN_PERMISSION_PENDING" || vpnBanner.code === "TUNNEL_LOST"}
      class:err={vpnBanner.code === "START_TIMEOUT"}
      role="status"
    >
      <strong>[{vpnBanner.code}] {vpnBanner.title}</strong>
      <p>{vpnBanner.explanation}</p>
      <p class="muted">{vpnBanner.recoveryHint}</p>
      <div class="actions tight">
        {#if vpnBanner.code === "VPN_PERMISSION_PENDING"}
          <button type="button" onclick={onIveAllowedVpn} disabled={busy}>
            I’ve allowed it (recheck)
          </button>
          <button type="button" onclick={onStop} disabled={busy || stopping}>Stop</button>
        {:else if vpnBanner.code === "TUNNEL_LOST"}
          <button type="button" class="primary" onclick={onRepairTunnel} disabled={!canAct}>
            Repair tunnel
          </button>
          <button type="button" onclick={onStop} disabled={busy || stopping}>Stop</button>
        {:else}
          <button type="button" class="primary" onclick={onRun} disabled={!canAct}>Retry Run</button>
          <button type="button" onclick={onStop} disabled={busy || stopping}>Stop</button>
        {/if}
      </div>
    </div>
  {/if}
  {#if actionBanner}
    <div class="banner err" role="alert">
      {#if actionBanner.kind === "ux"}
        <strong>[{actionBanner.ux.code}] {actionBanner.ux.title}</strong>
        <p>{actionBanner.ux.explanation}</p>
        <p class="muted">{actionBanner.ux.recoveryHint}</p>
        <p class="muted">{actionBanner.raw}</p>
      {:else}
        {actionBanner.raw}
      {/if}
    </div>
  {/if}

  <div class="grid">
    <section class="panel">
      <div class="panel-head">
        <h2>Devices</h2>
        <button type="button" onclick={() => refreshAll()} disabled={busy}>
          Refresh
        </button>
      </div>
      {#if devicesError && !adbBanner}
        <p class="err">{devicesError}</p>
      {:else if devices.length === 0}
        <p class="muted">
          No devices. Plug in USB and enable debugging. List polls every
          {DEVICE_POLL_VISIBLE_MS / 1000}s while visible.
        </p>
      {:else}
        <ul class="devices">
          {#each devices as d}
            <li>
              <label>
                <input
                  type="radio"
                  name="serial"
                  value={d.serial}
                  bind:group={selectedSerial}
                />
                <code>{d.serial}</code>
                <span class="state {deviceStateClass(d.adbState)}"
                  >{deviceStateLabel(d.adbState)}</span
                >
                {#if d.model}<span class="muted">{d.model}</span>{/if}
              </label>
            </li>
          {/each}
        </ul>
      {/if}
      {#if selected}
        <p class="muted select-hint">
          Selected: <code>{selected.serial}</code>
          ({deviceStateLabel(selected.adbState)})
        </p>
      {/if}
    </section>

    <section class="panel">
      <div class="panel-head">
        <h2>Session actions</h2>
      </div>
      <p class="muted">
        Run ≈ install-if-needed → tunnel → start + session-owned relay. Stop =
        stop client + clear owned relay. Sharing is not claimed without
        handshake.
      </p>
      <div class="actions wrap">
        <button type="button" onclick={onInstall} disabled={!canAct}>Install</button>
        <button type="button" onclick={onRepairTunnel} disabled={!canAct}>
          Repair tunnel
        </button>
        <button type="button" class="primary" onclick={onRun} disabled={!canAct}>
          {busy ? "Starting…" : "Run"}
        </button>
        <button
          type="button"
          onclick={onStop}
          disabled={!adbOk || !selectedSerial || busy || stopping}
        >
          {stopping ? "Stopping…" : "Stop"}
        </button>
      </div>
      <hr class="sep" />
      <p class="muted">Relay-only (advanced):</p>
      <div class="actions">
        <button type="button" onclick={onStartRelay} disabled={busy || stopping}>
          Start Relay
        </button>
        <button
          type="button"
          onclick={onStopRelay}
          disabled={busy || stopping || !relay.ownedBySession}
        >
          Stop Relay
        </button>
      </div>
      {#if relay.message}
        <p class="muted">{relay.message}</p>
      {/if}
    </section>
  </div>

  <section class="panel logs">
    <div class="panel-head">
      <h2>Logs</h2>
      <button type="button" onclick={() => (logs = [])}>Clear</button>
    </div>
    <pre class="log-pane">{#each logs as line}{new Date(line.timestampMs).toISOString()} [{line.level}] {line.source}: {line.message}
{/each}</pre>
  </section>
</main>

<style>
  :root {
    font-family: Inter, system-ui, sans-serif;
    color: #e8eaed;
    background: #12141a;
    line-height: 1.45;
  }
  :global(body) {
    margin: 0;
    background: #12141a;
  }
  .app {
    max-width: 960px;
    margin: 0 auto;
    padding: 1.25rem 1.5rem 2rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  header h1 {
    margin: 0;
    font-size: 1.5rem;
  }
  .sub {
    margin: 0.25rem 0 0;
    color: #9aa0a6;
    font-size: 0.9rem;
  }
  .status-bar {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
  }
  .chip,
  .layer {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: #1e222b;
    border: 1px solid #2a2f3a;
    border-radius: 999px;
    padding: 0.35rem 0.75rem;
    font-size: 0.85rem;
  }
  .chip .label,
  .layer .label {
    font-weight: 600;
    color: #9aa0a6;
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 0.04em;
  }
  .layers {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  .layer {
    border-radius: 8px;
  }
  .layer .tip {
    font-size: 0.7rem;
  }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }
  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
  .panel {
    background: #1a1d24;
    border: 1px solid #2a2f3a;
    border-radius: 10px;
    padding: 0.9rem 1rem;
  }
  .panel-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  .panel-head h2 {
    margin: 0;
    font-size: 1.05rem;
  }
  .devices {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .devices li {
    padding: 0.35rem 0;
    border-bottom: 1px solid #2a2f3a;
  }
  .devices label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    cursor: pointer;
  }
  .state {
    font-size: 0.8rem;
    text-transform: uppercase;
  }
  .select-hint {
    margin-top: 0.75rem;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
    flex-wrap: wrap;
  }
  .actions.tight {
    margin-top: 0.5rem;
  }
  .actions.wrap {
    flex-wrap: wrap;
  }
  .sep {
    border: none;
    border-top: 1px solid #2a2f3a;
    margin: 1rem 0 0.5rem;
  }
  button {
    border-radius: 8px;
    border: 1px solid #3c4454;
    background: #252a35;
    color: inherit;
    padding: 0.45rem 0.9rem;
    font: inherit;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    border-color: #5b8def;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button.primary {
    background: #1a73e8;
    border-color: #1a73e8;
  }
  .banner {
    padding: 0.6rem 0.8rem;
    border-radius: 8px;
    background: #1e222b;
    border: 1px solid #3c4454;
  }
  .banner p {
    margin: 0.35rem 0 0;
  }
  .banner.err {
    background: #3b1d1d;
    border-color: #8b3a3a;
  }
  .banner.warn {
    background: #3b3218;
    border-color: #a67c2a;
  }
  .ok {
    color: #81c995;
  }
  .warn {
    color: #fdd663;
  }
  .err {
    color: #f28b82;
  }
  .muted {
    color: #9aa0a6;
    font-size: 0.9rem;
  }
  .logs .log-pane {
    margin: 0;
    max-height: 280px;
    overflow: auto;
    background: #0e1014;
    border-radius: 6px;
    padding: 0.6rem 0.75rem;
    font-size: 0.78rem;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    white-space: pre-wrap;
    word-break: break-word;
  }
  code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    font-size: 0.85em;
  }
</style>
