<script lang="ts" module>
  export type Panel = "log" | "preferences";
</script>

<script lang="ts">
  import { app } from "$lib/app.svelte";

  let { panel = $bindable(null) }: { panel?: Panel | null } = $props();

  const hasErrors = $derived(app.logs.some((l) => l.level === "error"));

  // Ctrl+L opens the log and Ctrl+, opens preferences.
  function onKeydown(event: KeyboardEvent) {
    if (!event.ctrlKey || event.altKey || event.metaKey) return;
    const target = event.key === "l" ? "log" : event.key === "," ? "preferences" : null;
    if (target) {
      event.preventDefault();
      panel = panel === target ? null : target;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<header>
  <button type="button" class="btn small flat" title="Ctrl+L" onclick={() => (panel = "log")}>
    Log
    {#if hasErrors}<span class="badge" aria-label="contains errors"></span>{/if}
  </button>
  <button type="button" class="btn small flat" title="Ctrl+," onclick={() => (panel = "preferences")}>
    Preferences
  </button>
</header>

<style>
  /* Floats over the connection screen so its glow is never cut off. */
  header {
    position: absolute;
    top: 0;
    right: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
    height: 44px;
    padding: 0 10px;
  }

  .btn {
    font-weight: 600;
    color: var(--fg-dim);
  }
  .btn:hover {
    color: var(--fg);
  }

  .badge {
    width: 6px;
    height: 6px;
    margin-left: 6px;
    border-radius: 50%;
    background: var(--danger-dot);
  }
</style>
