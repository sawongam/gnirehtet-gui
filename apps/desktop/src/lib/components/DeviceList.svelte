<script lang="ts">
  import { Download, RefreshCw, Smartphone } from "@lucide/svelte";
  import Badge from "$lib/components/ui/badge.svelte";
  import Button from "$lib/components/ui/button.svelte";
  import Card from "$lib/components/ui/card.svelte";
  import { app } from "$lib/app.svelte";
  import { deviceStateInfo } from "$lib/errors";
  import { deviceName, isReady } from "$lib/session-logic";
  import { cn } from "$lib/utils";

  let refreshing = $state(false);

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

<Card class="p-5">
  <div class="mb-3 flex items-center justify-between gap-2">
    <h2 class="text-[15px] font-semibold text-fg">Devices</h2>
    <Button variant="ghost" size="sm" onclick={refresh} disabled={refreshing}>
      <RefreshCw class={cn("size-3.5", refreshing && "animate-spin")} />
      Refresh
    </Button>
  </div>

  {#if !app.adb}
    <p class="py-6 text-center text-sm text-fg-muted">Devices appear here once adb is available.</p>
  {:else if app.devices.length === 0}
    <div class="rounded-[var(--radius)] border border-dashed border-border px-4 py-6 text-center">
      <p class="text-sm font-medium text-fg">No device detected</p>
      <p class="mx-auto mt-1 max-w-sm text-xs text-fg-muted">
        Connect the phone with a data-capable USB cable and enable USB debugging in its developer options.
      </p>
    </div>
  {:else}
    <ul class="flex flex-col gap-2" role="radiogroup" aria-label="Device">
      {#each app.devices as device (device.serial)}
        {@const info = deviceStateInfo(device.state)}
        {@const selected = app.selectedSerial === device.serial}
        <li>
          <button
            type="button"
            role="radio"
            aria-checked={selected}
            disabled={app.sessionActive && !selected}
            class={cn(
              "flex w-full items-start gap-3 rounded-[var(--radius)] border px-3 py-2.5 text-left transition-colors",
              selected ? "border-accent bg-accent-50" : "border-border hover:bg-bg-muted",
              "disabled:cursor-not-allowed disabled:opacity-60",
            )}
            onclick={() => app.select(device.serial)}
          >
            <Smartphone class="mt-0.5 size-4 shrink-0 text-fg-muted" />
            <div class="min-w-0 flex-1">
              <div class="flex flex-wrap items-center gap-2">
                <span class="truncate text-sm font-medium text-fg">{deviceName(device)}</span>
                <Badge tone={info.tone}>{info.label}</Badge>
              </div>
              <p class="truncate font-mono text-xs text-fg-muted">{device.serial}</p>
              {#if info.help}
                <p class="mt-1 text-xs text-fg-body">{info.help}</p>
              {/if}
            </div>
          </button>
        </li>
      {/each}
    </ul>

    {#if app.devices.length > 1 && !app.sessionActive}
      <p class="mt-2 text-xs text-fg-muted">Select the device to share the connection with.</p>
    {/if}

    <div class="mt-4 flex items-center justify-between gap-3 border-t border-border pt-4">
      <p class="text-xs text-fg-muted">
        The client app is installed automatically on first connect.
      </p>
      <Button
        variant="secondary"
        size="sm"
        onclick={() => app.installClient()}
        disabled={app.sessionActive || app.busy || !isReady(app.selectedDevice)}
        title="Install or reinstall the gnirehtet client on the selected device"
      >
        <Download class="size-3.5" />
        {app.task === "install" ? "Installing…" : "Reinstall client"}
      </Button>
    </div>
  {/if}
</Card>
