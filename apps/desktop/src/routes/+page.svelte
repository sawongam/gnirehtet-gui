<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Sidebar, { type Page } from "$lib/components/Sidebar.svelte";
  import StatusPanel from "$lib/components/StatusPanel.svelte";
  import MessageBanner from "$lib/components/MessageBanner.svelte";
  import DeviceList from "$lib/components/DeviceList.svelte";
  import SetupChecklist from "$lib/components/SetupChecklist.svelte";
  import LogsView from "$lib/components/LogsView.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Dialog from "$lib/components/ui/dialog.svelte";
  import { app } from "$lib/app.svelte";
  import { isTauri } from "$lib/api";

  let page = $state<Page>("connection");
  let quitDialogOpen = $state(false);
  let quitting = $state(false);

  async function quit() {
    quitting = true;
    await app.disconnect();
    await getCurrentWindow().destroy();
  }

  onMount(() => {
    void app.start();
    let unlistenClose: (() => void) | null = null;
    if (isTauri()) {
      void getCurrentWindow()
        .onCloseRequested((event) => {
          // Closing with an active session would leave the phone's VPN
          // pointing at a relay that no longer exists: confirm first.
          if (app.sessionActive && !quitting) {
            event.preventDefault();
            quitDialogOpen = true;
          }
        })
        .then((unlisten) => (unlistenClose = unlisten));
    }
    return () => {
      app.stop();
      unlistenClose?.();
    };
  });
</script>

<div class="flex h-screen min-h-0 overflow-hidden bg-bg text-fg">
  <Sidebar bind:page />

  <main class="min-w-0 flex-1 overflow-y-auto">
    {#if page === "connection"}
      <div class="mx-auto flex max-w-5xl flex-col gap-5 p-6 pb-10">
        <div>
          <h1 class="text-2xl font-semibold tracking-tight text-fg">Connection</h1>
          <p class="text-sm text-fg-muted">Share this computer’s internet with an Android phone over USB.</p>
        </div>
        <StatusPanel />
        <MessageBanner />
        <div class="grid grid-cols-1 gap-5 lg:grid-cols-[minmax(0,1.4fr)_minmax(0,1fr)]">
          <DeviceList />
          <SetupChecklist />
        </div>
      </div>
    {:else if page === "logs"}
      <LogsView />
    {:else}
      <SettingsView />
    {/if}
  </main>
</div>

<Dialog
  bind:open={quitDialogOpen}
  title="Disconnect and quit?"
  description="A connection is active. Quitting stops the VPN on the phone and the relay."
  confirmLabel={quitting ? "Disconnecting…" : "Disconnect and quit"}
  busy={quitting}
  onConfirm={quit}
/>
