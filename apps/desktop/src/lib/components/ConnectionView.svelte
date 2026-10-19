<script lang="ts">
  import { app } from "$lib/app.svelte";
  import { deviceStateInfo, errorHelp } from "$lib/errors";
  import { deviceName, formatDuration, isReady } from "$lib/session-logic";

  /** After this long without a connection, suggest checking the phone. */
  const SLOW_WAIT_MS = 30_000;

  let now = $state(Date.now());
  let refreshing = $state(false);

  $effect(() => {
    if (app.phase !== "waiting" && app.phase !== "connected") return;
    now = Date.now();
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const elapsed = $derived(Math.max(0, now - app.phaseSince));
  const device = $derived(
    app.sessionSerial
      ? (app.devices.find((d) => d.serial === app.sessionSerial) ?? app.sessionDevice)
      : app.selectedDevice,
  );
  const name = $derived(device ? deviceName(device) : (app.sessionSerial ?? "the phone"));

  type Tone = "idle" | "progress" | "success" | "warning" | "danger";
  type Status = { tone: Tone; title: string; detail: string };

  const status = $derived.by((): Status => {
    switch (app.phase) {
      case "starting":
        return {
          tone: "progress",
          title: "Starting",
          detail: `Starting the relay and the client on ${name}. The first connection also installs the client.`,
        };
      case "waiting":
        return {
          tone: "warning",
          title: "Waiting for the phone",
          detail:
            elapsed > SLOW_WAIT_MS
              ? `Still waiting. Unlock ${name} and accept the “Connection request” prompt. If no prompt appears, disconnect and try again.`
              : `Accept the “Connection request” prompt on ${name}.`,
        };
      case "connected":
        return {
          tone: "success",
          title: "Connected",
          detail: `${name} is using this computer’s internet connection.`,
        };
      case "interrupted":
        return {
          tone: "danger",
          title: "Connection interrupted",
          detail: `${name} was unplugged. Plug the cable back in and the connection resumes by itself.`,
        };
      case "failed":
        return {
          tone: "danger",
          title: "Relay stopped",
          detail: "Reconnect to start it again, or disconnect to also turn off the VPN on the phone.",
        };
      case "stopping":
        return { tone: "progress", title: "Disconnecting", detail: `Turning off the VPN on ${name}.` };
      default:
        if (!app.loaded) return { tone: "progress", title: "Looking for adb", detail: "" };
        if (!app.adb) {
          const help = errorHelp(app.adbError?.code ?? "ADB_MISSING");
          return { tone: "danger", title: help.title, detail: help.hint };
        }
        if (!device) {
          return {
            tone: "idle",
            title: "No phone connected",
            detail: "Plug in an Android phone with USB debugging turned on.",
          };
        }
        if (!isReady(device)) {
          const info = deviceStateInfo(device.state);
          return {
            tone: "warning",
            title: info.label,
            detail: info.help ?? "This phone cannot be used yet.",
          };
        }
        return {
          tone: "idle",
          title: "Ready to connect",
          detail: `Share this computer’s internet connection with ${name} over USB.`,
        };
    }
  });

  const dot = $derived(
    { idle: "", progress: "", success: "success", warning: "warning", danger: "danger" }[status.tone],
  );
  const showTimer = $derived(app.phase === "connected" || app.phase === "waiting");
  const working = $derived(app.phase === "starting" || app.phase === "stopping" || app.phase === "waiting");
  const canConnect = $derived(
    app.phase === "idle" && !!app.adb && isReady(app.selectedDevice) && !app.busy,
  );

  const readyCount = $derived(app.devices.filter(isReady).length);
  const showSetup = $derived(app.loaded && app.phase === "idle" && (!app.adb || readyCount === 0));
  const setup = $derived([
    {
      title: "Install adb",
      detail: app.adb ? app.adb.version : "Android platform-tools, for example “sudo apt install adb”.",
      done: !!app.adb,
    },
    {
      title: "Allow USB debugging",
      detail:
        readyCount > 0
          ? "The phone has authorized this computer."
          : "Turn on USB debugging in developer options, plug in the phone and accept the prompt.",
      done: readyCount > 0,
    },
  ]);

  const relayValue = $derived(
    app.relay.state === "running" && app.relay.port
      ? `Port ${app.relay.port}`
      : app.relay.state === "crashed"
        ? "Stopped unexpectedly"
        : "Not running",
  );

  async function refresh() {
    refreshing = true;
    try {
      if (!app.adb) await app.checkAdb();
      await app.refreshDevices();
    } finally {
      refreshing = false;
    }
  }
</script>

<div class="clamp">
  <section class="status" aria-live="polite">
    <p class="device">
      <span class="dot {dot}" class:pulse={app.phase === "waiting"}></span>
      {device ? name : app.adb ? "No phone" : "adb unavailable"}
      {#if showTimer}<span class="timer numeric" title="Time in this state">{formatDuration(elapsed)}</span>{/if}
    </p>
    <h1>{status.title}</h1>
    <div class="progress" class:active={working} aria-hidden="true"><span></span></div>
    {#if status.detail}<p class="detail">{status.detail}</p>{/if}

    <div class="actions">
      {#if app.phase === "idle" && app.loaded && !app.adb}
        <button type="button" class="btn pill" onclick={refresh} disabled={refreshing}>
          {refreshing ? "Checking…" : "Check again"}
        </button>
      {:else if app.phase === "idle"}
        <button type="button" class="btn pill suggested" onclick={() => app.connect()} disabled={!canConnect}>
          Connect
        </button>
      {:else if app.phase === "failed"}
        <button
          type="button"
          class="btn pill suggested"
          onclick={() => app.connect()}
          disabled={app.busy || !isReady(app.selectedDevice)}
        >
          Reconnect
        </button>
        <button type="button" class="btn pill destructive" onclick={() => app.disconnect()}>Disconnect</button>
      {:else if app.phase === "interrupted"}
        <button
          type="button"
          class="btn pill"
          onclick={() => app.repairTunnel()}
          disabled={app.task !== null || !isReady(device)}
        >
          {app.task === "repair" ? "Restoring…" : "Restore tunnel"}
        </button>
        <button type="button" class="btn pill destructive" onclick={() => app.disconnect()}>Disconnect</button>
      {:else}
        <button
          type="button"
          class="btn pill destructive"
          onclick={() => app.disconnect()}
          disabled={app.phase === "starting" || app.phase === "stopping"}
        >
          {app.phase === "stopping" ? "Disconnecting…" : "Disconnect"}
        </button>
      {/if}
    </div>
  </section>

  {#if showSetup}
    <section class="group">
      <div class="group-head"><h2 class="group-title">Before you connect</h2></div>
      <ol class="list">
        {#each setup as step, i (step.title)}
          <li class="row">
            <span class="step numeric" class:done={step.done}>{i + 1}</span>
            <span class="row-text">
              <span class="row-title">{step.title}</span>
              <span class="row-sub">{step.detail}</span>
            </span>
            <span class="row-end {step.done ? 'tone-success' : 'tone-warning'}">{step.done ? "Done" : "To do"}</span>
          </li>
        {/each}
      </ol>
    </section>
  {/if}

  {#if app.adb}
    <section class="group">
      <div class="group-head">
        <h2 class="group-title">{app.devices.length > 1 ? "Phones" : "Phone"}</h2>
        <button type="button" class="btn small flat" onclick={refresh} disabled={refreshing}>
          {refreshing ? "Refreshing…" : "Refresh"}
        </button>
      </div>

      {#if app.devices.length === 0}
        <div class="list">
          <div class="row">
            <span class="row-text">
              {#if app.phase === "interrupted"}
                <span class="row-title">Waiting for {name}</span>
                <span class="row-sub">It reappears here when the cable is plugged back in.</span>
              {:else}
                <span class="row-title">No phone detected</span>
                <span class="row-sub">Use a USB cable that carries data, not one that only charges.</span>
              {/if}
            </span>
          </div>
        </div>
      {:else}
        <div class="list" role="radiogroup" aria-label="Phone">
          {#each app.devices as d (d.serial)}
            {@const info = deviceStateInfo(d.state)}
            {@const locked = app.sessionActive && app.selectedSerial !== d.serial}
            <label class="row" class:disabled={locked}>
              <input
                type="radio"
                name="device"
                class="radio"
                value={d.serial}
                checked={app.selectedSerial === d.serial}
                disabled={locked}
                onchange={() => app.select(d.serial)}
              />
              <span class="row-text">
                <span class="row-title">{deviceName(d)}</span>
                <span class="row-sub"><span class="mono selectable">{d.serial}</span></span>
                {#if info.help && d.serial !== device?.serial}<span class="row-sub help">{info.help}</span>{/if}
              </span>
              <span class="row-end tone-{info.tone}">{info.label}</span>
            </label>
          {/each}
        </div>
      {/if}
    </section>

    <section class="group">
      <div class="group-head"><h2 class="group-title">Details</h2></div>
      <div class="list">
        <div class="row">
          <span class="row-text"><span class="row-title">Relay</span></span>
          <span class="row-end numeric" class:tone-danger={app.relay.state === "crashed"}>{relayValue}</span>
        </div>
        <div class="row">
          <span class="row-text">
            <span class="row-title">Client app</span>
            <span class="row-sub">Installed on the phone automatically the first time you connect.</span>
          </span>
          <button
            type="button"
            class="btn small"
            onclick={() => app.installClient()}
            disabled={app.sessionActive || app.busy || !isReady(app.selectedDevice)}
          >
            {app.task === "install" ? "Installing…" : "Reinstall"}
          </button>
        </div>
      </div>
    </section>
  {/if}
</div>

<style>
  .status {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 52px 0 6px;
  }

  .device {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13.5px;
    font-weight: 700;
    color: var(--fg-dim);
  }

  .timer {
    font-weight: 400;
  }
  .timer::before {
    content: "·";
    margin-right: 8px;
  }

  h1 {
    margin-top: 10px;
    font-size: 28px;
    font-weight: 800;
    line-height: 1.15;
    letter-spacing: -0.01em;
    max-width: 460px;
    text-wrap: balance;
  }

  .detail {
    max-width: 400px;
    color: var(--fg-dim);
    text-wrap: pretty;
  }

  /* Thin indeterminate bar under the title while something is in progress. */
  .progress {
    position: relative;
    width: 120px;
    height: 2px;
    margin: 12px 0;
    border-radius: 2px;
    overflow: hidden;
    background: transparent;
    transition: background-color 200ms var(--ease);
  }
  .progress.active {
    background: var(--fill);
  }
  .progress span {
    position: absolute;
    inset: 0 auto 0 0;
    width: 40%;
    border-radius: 2px;
    background: var(--accent-bg);
    opacity: 0;
    transform: translateX(-100%);
  }
  .progress.active span {
    opacity: 1;
    animation: sweep 1.3s var(--ease) infinite;
  }
  @keyframes sweep {
    to {
      transform: translateX(260%);
    }
  }

  .pulse {
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-top: 22px;
    min-height: 40px;
  }

  .step {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    flex-shrink: 0;
    font-size: 12.5px;
    font-weight: 700;
    background: var(--fill);
    color: var(--fg-dim);
  }
  .step.done {
    background: var(--success-dot);
    color: #fff;
  }

  .help {
    margin-top: 2px;
    color: var(--fg);
  }

  .radio {
    appearance: none;
    flex-shrink: 0;
    width: 18px;
    height: 18px;
    margin: 0;
    border-radius: 50%;
    box-shadow: inset 0 0 0 2px var(--fg-faint);
    transition:
      box-shadow 120ms var(--ease),
      background-color 120ms var(--ease);
  }
  .radio:checked {
    background: var(--accent-bg);
    box-shadow: inset 0 0 0 5px var(--accent-bg), inset 0 0 0 9px #fff;
  }
  .radio:focus-visible {
    outline: 2px solid var(--accent-ring);
    outline-offset: 2px;
  }
</style>
