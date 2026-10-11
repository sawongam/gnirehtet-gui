<script lang="ts">
  import {
    CircleCheck,
    LoaderCircle,
    Smartphone,
    TriangleAlert,
    Unplug,
    Usb,
    Wifi,
  } from "@lucide/svelte";
  import Button from "$lib/components/ui/button.svelte";
  import Card from "$lib/components/ui/card.svelte";
  import { app } from "$lib/app.svelte";
  import { deviceStateInfo, errorHelp } from "$lib/errors";
  import { deviceName, formatDuration, isReady } from "$lib/session-logic";
  import { cn } from "$lib/utils";

  /** After this long without a connection, suggest checking the phone. */
  const SLOW_WAIT_MS = 30_000;

  let now = $state(Date.now());

  $effect(() => {
    if (app.phase !== "waiting" && app.phase !== "connected") return;
    const timer = setInterval(() => (now = Date.now()), 1000);
    return () => clearInterval(timer);
  });

  const elapsed = $derived(Math.max(0, now - app.phaseSince));
  const device = $derived(
    app.sessionSerial
      ? (app.devices.find((d) => d.serial === app.sessionSerial) ?? app.sessionDevice)
      : app.selectedDevice,
  );
  const name = $derived(device ? deviceName(device) : (app.sessionSerial ?? "your phone"));

  type View = {
    tone: "neutral" | "progress" | "success" | "warning" | "danger";
    title: string;
    detail: string;
  };

  const view = $derived.by((): View => {
    switch (app.phase) {
      case "starting":
        return {
          tone: "progress",
          title: "Starting…",
          detail: `Starting the relay and the gnirehtet client on ${name}. The first connection also installs the client.`,
        };
      case "waiting":
        return {
          tone: "warning",
          title: "Waiting for the phone",
          detail:
            elapsed > SLOW_WAIT_MS
              ? `Still waiting. Unlock ${name} and accept the “Connection request” (VPN) prompt. If there is no prompt, disconnect and connect again.`
              : `Accept the “Connection request” (VPN) prompt on ${name}.`,
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
          detail: `${name} was disconnected. Plug the USB cable back in; the connection resumes automatically.`,
        };
      case "failed":
        return {
          tone: "danger",
          title: "The relay stopped",
          detail: "Reconnect to start it again, or disconnect to also stop the VPN on the phone.",
        };
      case "stopping":
        return { tone: "progress", title: "Disconnecting…", detail: `Stopping the VPN on ${name}.` };
      default:
        if (!app.adb) {
          const help = errorHelp(app.adbError?.code ?? "ADB_MISSING");
          return { tone: "neutral", title: help.title, detail: help.hint };
        }
        if (!device) {
          return {
            tone: "neutral",
            title: "Connect your phone",
            detail: "Plug in an Android phone with USB debugging enabled.",
          };
        }
        if (!isReady(device)) {
          return {
            tone: "warning",
            title: deviceStateInfo(device.state).label,
            detail: deviceStateInfo(device.state).help ?? "This device cannot be used yet.",
          };
        }
        return {
          tone: "neutral",
          title: "Ready to connect",
          detail: `Share this computer’s internet connection with ${name} over USB.`,
        };
    }
  });

  const canConnect = $derived(
    app.phase === "idle" && !!app.adb && isReady(app.selectedDevice) && !app.busy,
  );

  const iconStyles: Record<View["tone"], string> = {
    neutral: "bg-accent-50 text-accent",
    progress: "bg-accent-50 text-accent",
    success: "bg-success-soft text-success-strong",
    warning: "bg-warning-soft text-warning-strong",
    danger: "bg-danger-soft text-danger-strong",
  };
</script>

<Card class={cn("p-6", view.tone === "success" && "border-success/30")}>
  <div class="flex flex-wrap items-center gap-5">
    <div class={cn("flex size-14 shrink-0 items-center justify-center rounded-full", iconStyles[view.tone])}>
      {#if view.tone === "progress"}
        <LoaderCircle class="size-7 animate-spin" />
      {:else if view.tone === "success"}
        <Wifi class="size-7" />
      {:else if app.phase === "interrupted"}
        <Unplug class="size-7" />
      {:else if view.tone === "danger" || view.tone === "warning"}
        {#if app.phase === "waiting"}
          <Smartphone class="size-7" />
        {:else}
          <TriangleAlert class="size-7" />
        {/if}
      {:else}
        <Usb class="size-7" />
      {/if}
    </div>

    <div class="min-w-0 flex-1" aria-live="polite">
      <div class="flex flex-wrap items-center gap-2">
        <h2 class="text-lg font-semibold text-fg">{view.title}</h2>
        {#if app.phase === "connected" || app.phase === "waiting"}
          <span class="font-mono text-sm text-fg-muted" title="Elapsed time">{formatDuration(elapsed)}</span>
        {/if}
      </div>
      <p class="mt-0.5 text-sm text-fg-body">{view.detail}</p>
      {#if app.relay.state === "running" && app.relay.port}
        <p class="mt-2 flex items-center gap-1.5 text-xs text-fg-muted">
          <CircleCheck class="size-3.5 text-success" />
          Relay listening on port {app.relay.port}
        </p>
      {/if}
    </div>

    <div class="flex shrink-0 flex-col gap-2">
      {#if app.phase === "idle"}
        <Button onclick={() => app.connect()} disabled={!canConnect}>Connect</Button>
      {:else if app.phase === "failed"}
        <Button onclick={() => app.connect()} disabled={app.busy || !isReady(app.selectedDevice)}>
          Reconnect
        </Button>
        <Button variant="destructive" onclick={() => app.disconnect()}>Disconnect</Button>
      {:else if app.phase === "interrupted"}
        <Button
          variant="secondary"
          onclick={() => app.repairTunnel()}
          disabled={app.task !== null || !isReady(device)}
        >
          {app.task === "repair" ? "Restoring…" : "Restore tunnel"}
        </Button>
        <Button variant="destructive" onclick={() => app.disconnect()}>Disconnect</Button>
      {:else}
        <Button
          variant="destructive"
          onclick={() => app.disconnect()}
          disabled={app.phase === "starting" || app.phase === "stopping"}
        >
          Disconnect
        </Button>
      {/if}
    </div>
  </div>
</Card>
