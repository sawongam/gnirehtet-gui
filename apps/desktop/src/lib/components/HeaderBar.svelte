<script lang="ts" module>
  export type Page = "connection" | "logs" | "preferences";
</script>

<script lang="ts">
  import { app } from "$lib/app.svelte";

  let { page = $bindable("connection") }: { page?: Page } = $props();

  const pages: { id: Page; label: string }[] = [
    { id: "connection", label: "Connection" },
    { id: "logs", label: "Log" },
    { id: "preferences", label: "Preferences" },
  ];

  const hasErrors = $derived(app.logs.some((l) => l.level === "error"));

  // Alt+1..3 switches views, as in other GNOME apps with a view switcher.
  function onKeydown(event: KeyboardEvent) {
    if (!event.altKey || event.ctrlKey || event.metaKey) return;
    const index = Number(event.key) - 1;
    if (index >= 0 && index < pages.length) {
      event.preventDefault();
      page = pages[index].id;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<header>
  <div class="segmented" role="tablist" aria-label="View">
    {#each pages as item, i (item.id)}
      <button
        type="button"
        role="tab"
        aria-selected={page === item.id}
        title="Alt+{i + 1}"
        onclick={() => (page = item.id)}
      >
        {item.label}
        {#if item.id === "logs" && hasErrors}
          <span class="badge" aria-label="contains errors"></span>
        {/if}
      </button>
    {/each}
  </div>
</header>

<style>
  header {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    height: 47px;
    padding: 0 8px;
    background: var(--headerbar);
    box-shadow: inset 0 -1px var(--headerbar-shade);
  }

  .segmented button {
    min-width: 104px;
  }

  .badge {
    display: inline-block;
    width: 6px;
    height: 6px;
    margin-left: 5px;
    vertical-align: 0.45em;
    border-radius: 50%;
    background: var(--danger-dot);
  }
</style>
