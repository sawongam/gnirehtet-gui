<script lang="ts">
  import { fade, scale } from "svelte/transition";

  let {
    open = $bindable(false),
    title,
    description,
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    busy = false,
    onConfirm,
  }: {
    open?: boolean;
    title: string;
    description?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    busy?: boolean;
    onConfirm: () => void | Promise<void>;
  } = $props();

  let cancelButton = $state<HTMLButtonElement | null>(null);

  // Focus the safe choice, as alert dialogs do for destructive actions.
  $effect(() => {
    if (open) cancelButton?.focus();
  });

  function cancel() {
    if (!busy) open = false;
  }
</script>

<svelte:window
  onkeydown={(e) => {
    if (open && e.key === "Escape") cancel();
  }}
/>

{#if open}
  <div
    class="scrim"
    role="presentation"
    out:fade={{ duration: 120 }}
    onclick={(e) => {
      if (e.target === e.currentTarget) cancel();
    }}
  >
    <div
      class="dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      aria-describedby={description ? "dialog-description" : undefined}
      out:scale={{ duration: 120, start: 0.96 }}
    >
      <div class="body">
        <h2 id="dialog-title">{title}</h2>
        {#if description}
          <p id="dialog-description">{description}</p>
        {/if}
      </div>
      <div class="buttons">
        <button bind:this={cancelButton} type="button" class="btn" onclick={cancel} disabled={busy}>
          {cancelLabel}
        </button>
        <button type="button" class="btn destructive" onclick={() => onConfirm()} disabled={busy}>
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: grid;
    place-items: center;
    padding: 24px;
    background: var(--scrim);
    animation: scrim-in 120ms ease-out;
  }

  .dialog {
    width: 100%;
    max-width: 372px;
    border-radius: 14px;
    background: var(--dialog);
    box-shadow:
      0 0 0 1px rgb(0 0 6 / 0.05),
      0 12px 32px rgb(0 0 6 / 0.25);
    animation: dialog-in 160ms cubic-bezier(0.22, 1, 0.36, 1);
  }

  /* CSS intros, as in the sheet: no flash of the end state on mount. */
  @keyframes scrim-in {
    from {
      opacity: 0;
    }
  }
  @keyframes dialog-in {
    from {
      opacity: 0;
      scale: 0.96;
    }
  }

  .body {
    padding: 28px 28px 22px;
    text-align: center;
  }

  h2 {
    font-size: 20px;
    font-weight: 800;
    line-height: 1.25;
  }

  p {
    margin-top: 10px;
    color: var(--fg-dim);
  }

  .buttons {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    padding: 0 24px 24px;
  }

  .buttons .btn {
    min-height: 40px;
    border-radius: 10px;
  }
</style>
