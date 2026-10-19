<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade, fly } from "svelte/transition";
  import { cubicOut } from "svelte/easing";

  let {
    open = $bindable(false),
    title,
    fill = false,
    children,
  }: {
    open?: boolean;
    title: string;
    /** The body manages its own scrolling (e.g. the log). */
    fill?: boolean;
    children: Snippet;
  } = $props();

  let panel = $state<HTMLElement | null>(null);

  $effect(() => {
    if (open) panel?.focus();
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (open && e.key === "Escape") open = false;
  }}
/>

{#if open}
  <div
    class="scrim"
    role="presentation"
    transition:fade={{ duration: 180 }}
    onclick={(e) => {
      if (e.target === e.currentTarget) open = false;
    }}
  >
    <div
      bind:this={panel}
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-label={title}
      tabindex="-1"
      transition:fly={{ x: 48, duration: 260, easing: cubicOut, opacity: 0 }}
    >
      <div class="bar">
        <h2>{title}</h2>
        <button type="button" class="btn small" onclick={() => (open = false)}>Done</button>
      </div>
      <div class="content" class:fill>
        {@render children()}
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: flex;
    justify-content: flex-end;
    padding: 8px;
    background: var(--scrim);
  }

  .panel {
    display: flex;
    flex-direction: column;
    width: min(580px, 100%);
    height: 100%;
    border-radius: 14px;
    background: var(--window);
    box-shadow:
      0 0 0 1px rgb(0 0 6 / 0.06),
      0 16px 48px rgb(0 0 6 / 0.28);
    overflow: hidden;
    outline: none;
  }

  .bar {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px;
    padding: 0 12px 0 20px;
  }

  h2 {
    font-size: 15px;
    font-weight: 800;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-bottom: 28px;
  }
  .content.fill {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding-bottom: 0;
  }
</style>
