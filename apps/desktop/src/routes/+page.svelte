<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    ensureAdb,
    listDevices,
    startRelay,
    stopRelay,
    getRelayState,
  } from "$lib/api/orchestrator";
  import type {
    AdbInfo,
    DeviceInfo,
    AppError,
    LogLine,
    RelayStatePayload,
  } from "$lib/types/orchestrator";

  let adb = $state<AdbInfo | null>(null);
  let adbError = $state<string | null>(null);
  let devices = $state<DeviceInfo[]>([]);
  let devicesError = $state<string | null>(null);
  let selectedSerial = $state<string | null>(null);
  let relay = $state<RelayStatePayload>({
    state: "relay_stopped",
    ownedBySession: false,
  });
  let logs = $state<LogLine[]>([]);
  let busy = $state(false);
  let actionError = $state<string | null>(null);
  let lastAppError = $state<AppError | null>(null);

  const LOG_CAP = 500;

  function errMsg(e: unknown): string {
    if (e && typeof e === "object" && "message" in e) {
      const m = (e as { code?: string; message: string }).message;
      const c = (e as { code?: string }).code;
      return c ? `[${c}] ${m}` : m;
    }
    return String(e);
  }

  async function refreshAdb() {
    adbError = null;
    try {
      adb = await ensureAdb();
    } catch (e) {
      adb = null;
      adbError = errMsg(e);
    }
  }

  async function refreshDevices() {
    devicesError = null;
    try {
      devices = await listDevices();
      if (selectedSerial && !devices.some((d) => d.serial === selectedSerial)) {
        selectedSerial = null;
      }
      if (!selectedSerial && devices.length === 1) {
        selectedSerial = devices[0].serial;
      }
    } catch (e) {
      devices = [];
      devicesError = errMsg(e);
    }
  }

  async function refreshRelay() {
    try {
      relay = await getRelayState();
    } catch {
      /* ignore */
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
        }),
      );
      if (!cancelled) {
        await refreshAdb();
        await refreshDevices();
        await refreshRelay();
      }
    })();

    return () => {
      cancelled = true;
      unlisteners.forEach((u) => u());
    };
  });

  const relayLabel = $derived(
    relay.state.replace(/^relay_/, "").replace(/_/g, " "),
  );
</script>

<main class="app">
  <header>
    <h1>gnirehtet-gui</h1>
    <p class="sub">Phase 0 scaffold — host-orchestrator stubs (one device)</p>
  </header>

  <section class="status-bar" aria-live="polite">
    <div class="chip">
      <span class="label">ADB</span>
      {#if adb}
        <span class="ok">ok</span>
        <span class="muted">{adb.version}</span>
      {:else if adbError}
        <span class="err">missing</span>
      {:else}
        <span class="muted">…</span>
      {/if}
    </div>
    <div class="chip">
      <span class="label">Relay</span>
      <span class:ok={relay.state === "relay_running"} class:err={relay.state === "relay_error" || relay.state === "relay_exited"}
        >{relayLabel}</span
      >
      {#if relay.port}
        <span class="muted">:{relay.port}</span>
      {/if}
      {#if relay.ownedBySession}
        <span class="ok">owned</span>
      {/if}
    </div>
  </section>

  {#if adbError}
    <div class="banner err" role="alert">{adbError}</div>
  {/if}
  {#if actionError}
    <div class="banner err" role="alert">{actionError}</div>
  {/if}

  <div class="grid">
    <section class="panel">
      <div class="panel-head">
        <h2>Devices</h2>
        <button type="button" onclick={() => refreshDevices()} disabled={busy}>Refresh</button>
      </div>
      {#if devicesError}
        <p class="err">{devicesError}</p>
      {:else if devices.length === 0}
        <p class="muted">No devices. Plug in USB and enable debugging.</p>
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
                <span class="state">{d.adbState}</span>
                {#if d.model}<span class="muted">{d.model}</span>{/if}
              </label>
            </li>
          {/each}
        </ul>
      {/if}
    </section>

    <section class="panel">
      <div class="panel-head">
        <h2>Relay</h2>
      </div>
      <p class="muted">
        Start/Stop spawn the stock external <code>gnirehtet</code> binary (session-owned only).
        Maps toward EVENT_STATUS_MAP later.
      </p>
      <div class="actions">
        <button type="button" class="primary" onclick={onStartRelay} disabled={busy}>
          Start Relay
        </button>
        <button type="button" onclick={onStopRelay} disabled={busy || !relay.ownedBySession}>
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
  .chip {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: #1e222b;
    border: 1px solid #2a2f3a;
    border-radius: 999px;
    padding: 0.35rem 0.75rem;
    font-size: 0.85rem;
  }
  .chip .label {
    font-weight: 600;
    color: #9aa0a6;
    text-transform: uppercase;
    font-size: 0.7rem;
    letter-spacing: 0.04em;
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
    color: #8ab4f8;
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.75rem;
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
    background: #3b1d1d;
    border: 1px solid #8b3a3a;
  }
  .ok {
    color: #81c995;
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
