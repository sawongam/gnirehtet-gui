<script lang="ts">
  import { onMount } from "svelte";
  import HeaderBar, { type Panel } from "$lib/components/HeaderBar.svelte";
  import Banner from "$lib/components/Banner.svelte";
  import ConnectionView from "$lib/components/ConnectionView.svelte";
  import LogsView from "$lib/components/LogsView.svelte";
  import PreferencesView from "$lib/components/PreferencesView.svelte";
  import Dialog from "$lib/components/ui/dialog.svelte";
  import Sheet from "$lib/components/ui/sheet.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { app } from "$lib/app.svelte";
  import { api, isTauri } from "$lib/api";

  let panel = $state<Panel | null>(null);
  let quitDialogOpen = $state(false);
  let quitting = $state(false);

  async function quit() {
    quitting = true;
    await app.disconnect();
    try {
      await api.quit();
    } catch {
      quitting = false;
      quitDialogOpen = false;
    }
  }

  onMount(() => {
    void app.start();
    // Sent when the window is closed while connected and there is no system
    // tray to keep the app in: quitting would turn off the phone's VPN.
    let unlistenQuit: (() => void) | null = null;
    let unmounted = false;
    if (isTauri()) {
      void listen("quit-requested", () => {
        if (!quitting) quitDialogOpen = true;
      }).then((unlisten) => {
        if (unmounted) unlisten();
        else unlistenQuit = unlisten;
      });
    }
    return () => {
      unmounted = true;
      app.stop();
      unlistenQuit?.();
    };
  });
</script>

<div class="window">
  <HeaderBar bind:panel />
  <main class:covered={panel !== null || quitDialogOpen}>
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
  description="The phone is still using this connection. Quitting turns off its VPN and stops the relay. To keep it running with the window closed, install a system tray (on Linux: libayatana-appindicator3 and a panel that shows tray icons)."
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

  /* Hold the phone's animations while an overlay covers them. In WebKitGTK,
     running layers underneath make the overlay drop out for a frame when its
     slide-in ends. (!important: each `animation` shorthand resets it.) */
  main.covered :global(*) {
    animation-play-state: paused !important;
  }
</style>
