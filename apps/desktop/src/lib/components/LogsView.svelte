<script lang="ts">
  import { tick } from "svelte";
  import { Copy, Check, Trash2 } from "@lucide/svelte";
  import Button from "$lib/components/ui/button.svelte";
  import Card from "$lib/components/ui/card.svelte";
  import { app } from "$lib/app.svelte";
  import type { LogEntry } from "$lib/types";
  import { cn } from "$lib/utils";

  let problemsOnly = $state(false);
  let copied = $state(false);
  let container = $state<HTMLDivElement | null>(null);
  let followTail = $state(true);

  const visible = $derived(
    problemsOnly ? app.logs.filter((l) => l.level === "warn" || l.level === "error") : app.logs,
  );

  function formatTime(ms: number): string {
    return new Date(ms).toLocaleTimeString(undefined, { hour12: false });
  }

  function format(entry: LogEntry): string {
    return `${new Date(entry.timestampMs).toISOString()} ${entry.level.toUpperCase()} [${entry.source}] ${entry.message}`;
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(visible.map(format).join("\n"));
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      copied = false;
    }
  }

  function onScroll() {
    if (!container) return;
    followTail = container.scrollHeight - container.scrollTop - container.clientHeight < 24;
  }

  $effect(() => {
    void visible.length;
    if (!followTail) return;
    void tick().then(() => {
      if (container) container.scrollTop = container.scrollHeight;
    });
  });

  const levelStyle: Record<LogEntry["level"], string> = {
    debug: "text-fg-subtle",
    info: "text-fg-body",
    warn: "text-warning-strong",
    error: "text-danger-strong",
  };
</script>

<div class="mx-auto flex h-full max-w-5xl flex-col gap-4 p-6">
  <div class="flex flex-wrap items-center justify-between gap-3">
    <div>
      <h1 class="text-2xl font-semibold tracking-tight text-fg">Logs</h1>
      <p class="text-sm text-fg-muted">Activity of this app and of the relay, for troubleshooting.</p>
    </div>
    <div class="flex items-center gap-2">
      <label class="flex items-center gap-2 text-sm text-fg-body">
        <input type="checkbox" bind:checked={problemsOnly} class="size-4 accent-[var(--accent)]" />
        Warnings and errors only
      </label>
      <Button variant="secondary" size="sm" onclick={copy} disabled={visible.length === 0}>
        {#if copied}<Check class="size-3.5" /> Copied{:else}<Copy class="size-3.5" /> Copy{/if}
      </Button>
      <Button variant="ghost" size="sm" onclick={() => app.clearLogs()} disabled={app.logs.length === 0}>
        <Trash2 class="size-3.5" /> Clear
      </Button>
    </div>
  </div>

  <Card class="min-h-0 flex-1 overflow-hidden">
    <!-- Focusable so the log can be scrolled with the keyboard. -->
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <div
      bind:this={container}
      onscroll={onScroll}
      class="h-full overflow-auto p-3 font-mono text-xs leading-relaxed"
      role="log"
      aria-label="Log output"
      tabindex="0"
    >
      {#if visible.length === 0}
        <p class="p-6 text-center font-sans text-sm text-fg-muted">No log entries yet.</p>
      {:else}
        {#each visible as entry, i (i)}
          <div class={cn("whitespace-pre-wrap break-words", levelStyle[entry.level])}>
            <span class="text-fg-subtle">{formatTime(entry.timestampMs)}</span>
            <span class="text-fg-subtle">{entry.source === "relay" ? "relay" : "app  "}</span>
            {entry.message}
          </div>
        {/each}
      {/if}
    </div>
  </Card>
</div>
