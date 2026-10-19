<script lang="ts">
  import { fly } from "svelte/transition";
  import { app } from "$lib/app.svelte";
  import { errorHelp } from "$lib/errors";

  let { onShowLog }: { onShowLog: () => void } = $props();

  const help = $derived(app.error ? errorHelp(app.error.code) : null);
  let showDetails = $state(false);

  $effect(() => {
    void app.error;
    showDetails = false;
  });
</script>

{#if app.error && help}
  <div class="banner error" role="alert" transition:fly={{ y: 16, duration: 220 }}>
    <div class="text">
      <p class="title">{help.title}</p>
      <p class="hint">{help.hint}</p>
      {#if showDetails}
        <p class="details mono selectable">{app.error.message}</p>
      {/if}
    </div>
    <div class="actions">
      <button type="button" class="btn small flat" onclick={() => (showDetails = !showDetails)}>
        {showDetails ? "Hide details" : "Details"}
      </button>
      <button type="button" class="btn small flat" onclick={onShowLog}>Log</button>
      <button type="button" class="btn small" onclick={() => app.dismissMessages()}>Dismiss</button>
    </div>
  </div>
{:else if app.notice}
  <div class="banner info" role="status" transition:fly={{ y: 16, duration: 220 }}>
    <div class="text">
      <p class="hint">{app.notice}</p>
    </div>
    <div class="actions">
      <button type="button" class="btn small" onclick={() => app.dismissMessages()}>Dismiss</button>
    </div>
  </div>
{/if}

<style>
  /* A card at the bottom of the window; the connection screen shrinks above it. */
  .banner {
    flex-shrink: 0;
    display: flex;
    align-items: flex-start;
    gap: 16px;
    width: min(600px, calc(100% - 32px));
    margin: 0 auto 16px;
    padding: 10px 12px 10px 18px;
    border-radius: 14px;
    box-shadow:
      0 0 0 1px var(--line),
      0 10px 30px -8px rgb(0 0 6 / 0.3);
  }

  .error {
    background: var(--banner-error);
  }

  .info {
    background: var(--banner-info);
  }

  .text {
    flex: 1;
    min-width: 0;
    padding: 4px 0;
  }

  .title {
    font-weight: 700;
  }

  .hint {
    font-size: 13.5px;
  }

  .details {
    margin-top: 6px;
    color: var(--fg-dim);
    overflow-wrap: anywhere;
  }

  .actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
</style>
