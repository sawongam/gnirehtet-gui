<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import HeaderBar, { type Panel } from "$lib/components/HeaderBar.svelte";
  import Banner from "$lib/components/Banner.svelte";
  import ConnectionView from "$lib/components/ConnectionView.svelte";
  import LogsView from "$lib/components/LogsView.svelte";
  import PreferencesView from "$lib/components/PreferencesView.svelte";
  import Dialog from "$lib/components/ui/dialog.svelte";
  import Sheet from "$lib/components/ui/sheet.svelte";
  import { app } from "$lib/app.svelte";
  import { isTauri } from "$lib/api";

  let panel = $state<Panel | null>(null);
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
  <HeaderBar bind:panel />
  <main>
    <ConnectionView />
  </main>
  <Banner onShowLog={() => (panel = "log")} />
</div>

<Sheet bind:open={() => panel === "log", (v) => (panel = v ? "log" : null)} title="Log" fill>
  <LogsView />
</Sheet>

<Sheet
  bind:open={() => panel === "preferences", (v) => (panel = v ? "preferences" : null)}
  title="Preferences"
>
  <PreferencesView />
</Sheet>

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
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  main {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    /* The phone illustration scales with the height available here. */
    container-type: size;
  }
</style>
