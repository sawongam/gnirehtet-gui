<script lang="ts">
  import { fly } from "svelte/transition";
  import { api } from "$lib/api";
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

  /** Error codes after which "Install adb" is the way forward. */
  const INSTALL_CODES = ["ADB_MISSING", "ADB_DOWNLOAD_FAILED", "ADB_INSTALL_FAILED"];
  const offerInstall = $derived(
    app.adbInstallable && INSTALL_CODES.includes(app.adbError?.code ?? "ADB_MISSING"),
  );

  const mb = (bytes: number) => (bytes / 1_000_000).toFixed(1);
  const installProgress = $derived.by(() => {
    const p = app.adbInstall;
    if (!p) return null;
    const fraction = p.total ? Math.min(1, p.received / p.total) : null;
    const label = p.total ? `${mb(p.received)} of ${mb(p.total)} MB` : p.received ? `${mb(p.received)} MB` : "Connecting…";
    return { fraction, label };
  });

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
          if (app.adbInstall) {
            return {
              scene: "absent",
              title: "Installing adb",
              detail: "Downloading Android platform-tools from Google. This takes a moment.",
            };
          }
          if (offerInstall && (app.adbError?.code ?? "ADB_MISSING") === "ADB_MISSING") {
            return {
              scene: "absent",
              title: "One more thing: adb",
              detail:
                "adb is the small Android tool this app uses to talk to your phone over USB. Install it here with one click. No admin rights needed.",
            };
          }
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
      {#if app.phase === "idle" && app.loaded && !app.adb && installProgress}
        <div class="install-progress">
          <div
            class="bar"
            class:indeterminate={installProgress.fraction === null}
            role="progressbar"
            aria-label="Downloading adb"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={installProgress.fraction === null ? undefined : Math.round(installProgress.fraction * 100)}
          >
            <span style:width={installProgress.fraction === null ? null : `${installProgress.fraction * 100}%`}></span>
          </div>
          <p class="numeric">{installProgress.label}</p>
        </div>
      {:else if app.phase === "idle" && app.loaded && !app.adb}
        {#if offerInstall}
          <button type="button" class="btn pill suggested" onclick={() => app.installAdb()}>
            {app.adbError?.code === "ADB_MISSING" || !app.adbError ? "Install adb" : "Try again"}
          </button>
        {/if}
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

    {#if app.phase === "idle" && app.loaded && !app.adb && offerInstall && !installProgress}
      <p class="fineprint">
        Downloads Android platform-tools from Google, under
        <button type="button" class="link" onclick={() => api.openAdbTerms()}>Google’s terms</button>.
      </p>
    {/if}
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

  .install-progress {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 240px;
    padding-top: 6px;
  }
  .install-progress p {
    font-size: 13px;
    font-weight: 700;
    color: var(--fg-dim);
  }
  .bar {
    position: relative;
    width: 100%;
    height: 6px;
    overflow: hidden;
    border-radius: 999px;
    background: color-mix(in srgb, currentColor 12%, transparent);
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: inherit;
    background: var(--accent-bg);
    transition: width 200ms ease-out;
  }
  .bar.indeterminate span {
    width: 35%;
    animation: slide 1.1s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(290%);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .bar.indeterminate span {
      animation: none;
      width: 100%;
      opacity: 0.4;
    }
  }

  .fineprint {
    margin-top: 12px;
    font-size: 12.5px;
    color: var(--fg-dim);
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    font: inherit;
    color: var(--accent);
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }
</style>
