<script lang="ts" module>
  export type Page = "connection" | "logs" | "settings";
</script>

<script lang="ts">
  import { Cable, ScrollText, Settings, Radio } from "@lucide/svelte";
  import { cn } from "$lib/utils";
  import { app } from "$lib/app.svelte";

  let { page = $bindable("connection") }: { page?: Page } = $props();

  const items = [
    { id: "connection", label: "Connection", icon: Cable },
    { id: "logs", label: "Logs", icon: ScrollText },
    { id: "settings", label: "Settings", icon: Settings },
  ] as const;

  const statusText = $derived.by(() => {
    switch (app.phase) {
      case "connected":
        return "Connected";
      case "waiting":
      case "starting":
        return "Connecting";
      case "interrupted":
        return "Interrupted";
      case "failed":
        return "Relay stopped";
      default:
        return "Not connected";
    }
  });

  const statusDot = $derived(
    app.phase === "connected"
      ? "bg-success"
      : app.phase === "interrupted" || app.phase === "failed"
        ? "bg-danger"
        : app.sessionActive
          ? "bg-warning"
          : "bg-fg-subtle",
  );
</script>

<aside class="flex w-56 shrink-0 flex-col border-r border-border bg-bg-elevated" aria-label="Main navigation">
  <div class="flex items-center gap-3 px-5 py-5">
    <div class="flex size-9 items-center justify-center rounded-[var(--radius)] bg-accent text-accent-fg">
      <Radio class="size-5" />
    </div>
    <div class="min-w-0">
      <p class="truncate text-[15px] font-semibold leading-tight text-fg">Gnirehtet</p>
      <p class="truncate text-xs text-fg-muted">Reverse tethering</p>
    </div>
  </div>

  <nav class="flex flex-1 flex-col gap-1 px-3">
    {#each items as item (item.id)}
      {@const Icon = item.icon}
      {@const active = page === item.id}
      <button
        type="button"
        class={cn(
          "flex h-9 items-center gap-3 rounded-[var(--radius)] px-3 text-sm font-medium transition-colors",
          active ? "bg-accent-50 text-accent" : "text-fg-body hover:bg-bg-muted hover:text-fg",
        )}
        aria-current={active ? "page" : undefined}
        onclick={() => (page = item.id)}
      >
        <Icon class="size-4 shrink-0" />
        <span>{item.label}</span>
        {#if item.id === "logs" && app.logs.some((l) => l.level === "error")}
          <span class="ml-auto size-1.5 rounded-full bg-danger" aria-label="Contains errors"></span>
        {/if}
      </button>
    {/each}
  </nav>

  <div class="space-y-1.5 border-t border-border px-5 py-4 text-xs">
    <p class="flex items-center gap-2 text-fg-body">
      <span class={cn("inline-block size-2 rounded-full", statusDot)} aria-hidden="true"></span>
      {statusText}
    </p>
    <p class="flex items-center gap-2 text-fg-muted">
      <span
        class={cn("inline-block size-2 rounded-full", app.adb ? "bg-success" : "bg-danger")}
        aria-hidden="true"
      ></span>
      {app.adb ? "adb available" : app.loaded ? "adb not found" : "Checking adb…"}
    </p>
  </div>
</aside>
