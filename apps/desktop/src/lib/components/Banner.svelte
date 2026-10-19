<script lang="ts">
  import { slide } from "svelte/transition";
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
  <div class="banner error" role="alert" transition:slide={{ duration: 160 }}>
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
  <div class="banner info" role="status" transition:slide={{ duration: 160 }}>
    <div class="text">
      <p class="hint">{app.notice}</p>
    </div>
    <div class="actions">
      <button type="button" class="btn small" onclick={() => app.dismissMessages()}>Dismiss</button>
    </div>
  </div>
{/if}

<style>
  .banner {
    flex-shrink: 0;
    display: flex;
    align-items: flex-start;
    gap: 16px;
    padding: 10px 14px 10px 18px;
    box-shadow: inset 0 -1px var(--line);
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
