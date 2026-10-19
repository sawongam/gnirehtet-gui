<script lang="ts">
  import { fly } from "svelte/transition";
  import { app } from "$lib/app.svelte";
  import { deviceStateInfo, errorHelp } from "$lib/errors";
  import { deviceName, formatDuration, isReady } from "$lib/session-logic";
  import PhoneStage, { type Scene } from "./PhoneStage.svelte";

  /** After this long without a connection, suggest checking the phone. */
  const SLOW_WAIT_MS = 30_000;

  let now = $state(Date.now());
  let refreshing = $state(false);

  // Ticks every second: drives the session timer and the phone's clock.
  $effect(() => {
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const elapsed = $derived(Math.max(0, now - app.phaseSince));
  const clock = $derived(
    new Date(now).toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit", hour12: false }),
  );
  const device = $derived(
    app.sessionSerial
      ? (app.devices.find((d) => d.serial === app.sessionSerial) ?? app.sessionDevice)
      : app.selectedDevice,
  );
  const name = $derived(device ? deviceName(device) : (app.sessionSerial ?? "the phone"));

  type Status = { scene: Scene; title: string; detail: string };

  const status = $derived.by((): Status => {
    switch (app.phase) {
      case "starting":
        return {
          scene: "starting",
          title: "Starting",
          detail: `Starting the relay and the client on ${name}. The first connection also installs the client.`,
        };
      case "waiting":
        return {
          scene: "waiting",
          title: "Accept on your phone",
          detail:
            elapsed > SLOW_WAIT_MS
              ? `Still waiting. Unlock ${name} and tap OK on “Connection request”. If nothing appears, disconnect and try again.`
              : `Tap OK on the “Connection request” prompt on ${name}.`,
        };
      case "connected":
        return {
          scene: "connected",
          title: "Connected",
          detail: `${name} is using this computer’s internet connection.`,
        };
      case "interrupted":
        return {
          scene: "interrupted",
          title: "Cable unplugged",
          detail: `Plug ${name} back in and the connection resumes by itself.`,
        };
      case "failed":
        return {
          scene: "failed",
          title: "Relay stopped",
          detail: "Reconnect to start it again, or disconnect to also turn off the VPN on the phone.",
        };
      case "stopping":
        return { scene: "stopping", title: "Disconnecting", detail: `Turning off the VPN on ${name}.` };
      default:
        if (!app.loaded) return { scene: "absent", title: "Looking for adb", detail: "" };
        if (!app.adb) {
          const help = errorHelp(app.adbError?.code ?? "ADB_MISSING");
          return { scene: "absent", title: help.title, detail: help.hint };
        }
        if (!device) {
          return {
            scene: "absent",
            title: "Plug in your phone",
            detail: "Connect an Android phone with USB debugging turned on. Use a cable that carries data.",
          };
        }
        if (!isReady(device)) {
          const info = deviceStateInfo(device.state);
          return {
            scene: device.state === "unauthorized" ? "unauthorized" : "unusable",
            title: device.state === "unauthorized" ? "Allow USB debugging" : info.label,
            detail: info.help ?? "This phone cannot be used yet.",
          };
        }
        return {
          scene: "ready",
          title: "Ready to connect",
          detail: `Share this computer’s internet with ${name} over USB.`,
        };
    }
  });

  const meta = $derived.by(() => {
    if (app.phase === "connected") return `Connected for ${formatDuration(elapsed)}`;
    if (app.phase === "waiting") return `Waiting ${formatDuration(elapsed)}`;
    if (device) return name;
    return "";
  });

  const canConnect = $derived(app.phase === "idle" && !!app.adb && isReady(app.selectedDevice) && !app.busy);
  const showPicker = $derived(app.devices.length > 1 && !app.sessionActive);

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

<div class="view">
  <PhoneStage scene={status.scene} {clock} />

  <section class="status" aria-live="polite">
    {#if showPicker}
      <div class="segmented picker" role="radiogroup" aria-label="Phone" transition:fly={{ y: -6, duration: 200 }}>
        {#each app.devices as d (d.serial)}
          <button
            type="button"
            role="radio"
            aria-checked={app.selectedSerial === d.serial}
            title={d.serial}
            onclick={() => app.select(d.serial)}
          >
            {deviceName(d)}
          </button>
        {/each}
      </div>
    {:else}
      <p class="meta numeric">{meta}</p>
    {/if}

    {#key status.title}
      <div class="text" in:fly={{ y: 8, duration: 320, delay: 80 }}>
        <h1>{status.title}</h1>
        {#if status.detail}<p class="detail">{status.detail}</p>{/if}
      </div>
    {/key}

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
</div>

<style>
  .view {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100%;
    padding: 44px 24px 24px;
  }

  .status {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    width: 100%;
    max-width: 440px;
    margin-top: 8px;
  }

  .meta {
    min-height: 22px;
    font-size: 13.5px;
    font-weight: 700;
    color: var(--fg-dim);
  }

  .picker {
    max-width: 100%;
    overflow-x: auto;
  }
  .picker button {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .text {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-height: 92px;
  }

  h1 {
    margin-top: 6px;
    font-size: 28px;
    font-weight: 800;
    line-height: 1.15;
    letter-spacing: -0.015em;
    text-wrap: balance;
  }

  .detail {
    max-width: 380px;
    margin-top: 8px;
    color: var(--fg-dim);
    text-wrap: pretty;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-top: 18px;
    min-height: 40px;
  }
</style>
