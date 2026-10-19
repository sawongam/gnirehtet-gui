<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import HeaderBar, { type Page } from "$lib/components/HeaderBar.svelte";
  import Banner from "$lib/components/Banner.svelte";
  import ConnectionView from "$lib/components/ConnectionView.svelte";
  import LogsView from "$lib/components/LogsView.svelte";
  import PreferencesView from "$lib/components/PreferencesView.svelte";
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

<div class="window">
  <HeaderBar bind:page />
  <Banner onShowLog={() => (page = "logs")} />

  <main class:fill={page === "logs"}>
    {#if page === "connection"}
      <ConnectionView />
    {:else if page === "logs"}
      <LogsView />
    {:else}
      <PreferencesView />
    {/if}
  </main>
</div>

<Dialog
  bind:open={quitDialogOpen}
  title="Disconnect and quit?"
  description="The phone is still using this connection. Quitting turns off its VPN and stops the relay."
  confirmLabel={quitting ? "Disconnecting…" : "Disconnect and quit"}
  busy={quitting}
  onConfirm={quit}
/>

<style>
  .window {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  main {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 0 36px;
  }

  main.fill {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0;
  }
</style>
