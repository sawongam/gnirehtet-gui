<script lang="ts">
  import { tick } from "svelte";
  import { app } from "$lib/app.svelte";
  import type { LogEntry } from "$lib/types";

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
</script>

<div class="toolbar">
  <div class="segmented" role="radiogroup" aria-label="Filter">
    <button type="button" role="radio" aria-checked={!problemsOnly} onclick={() => (problemsOnly = false)}>
      All
    </button>
    <button type="button" role="radio" aria-checked={problemsOnly} onclick={() => (problemsOnly = true)}>
      Problems
    </button>
  </div>
  <div class="tools">
    <button type="button" class="btn small" onclick={copy} disabled={visible.length === 0}>
      {copied ? "Copied" : "Copy"}
    </button>
    <button type="button" class="btn small flat" onclick={() => app.clearLogs()} disabled={app.logs.length === 0}>
      Clear
    </button>
  </div>
</div>

<!-- Focusable so the log can be scrolled with the keyboard. -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  bind:this={container}
  onscroll={onScroll}
  class="log selectable"
  role="log"
  aria-label="Log output"
  tabindex="0"
>
  {#if visible.length === 0}
    <p class="empty">{problemsOnly ? "No warnings or errors." : "Nothing logged yet."}</p>
  {:else}
    {#each visible as entry, i (i)}
      <div class="line {entry.level}">
        <span class="time numeric">{formatTime(entry.timestampMs)}</span>
        <span class="source">{entry.source}</span>
        <span class="message">{entry.message}</span>
      </div>
    {/each}
  {/if}
</div>

<style>
  .toolbar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 14px;
  }

  .tools {
    display: flex;
    gap: 4px;
  }

  .log {
    flex: 1;
    min-height: 0;
    overflow: auto;
    margin: 0 14px 14px;
    padding: 8px 0;
    border-radius: var(--radius-row);
    background: var(--view);
    box-shadow: var(--card-shadow);
    font-family: var(--font-mono);
    font-size: 12.5px;
    line-height: 1.6;
  }

  .line {
    display: grid;
    grid-template-columns: auto 3.6em 1fr;
    gap: 12px;
    padding: 0 14px;
  }
  .line:hover {
    background: var(--fill);
  }

  .time,
  .source {
    color: var(--fg-faint);
  }

  .message {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .debug .message {
    color: var(--fg-dim);
  }
  .warn .message {
    color: var(--warning);
  }
  .error .message {
    color: var(--danger);
  }

  .empty {
    padding: 48px 16px;
    text-align: center;
    font-family: var(--font-ui);
    font-size: 14.5px;
    color: var(--fg-dim);
  }
</style>
