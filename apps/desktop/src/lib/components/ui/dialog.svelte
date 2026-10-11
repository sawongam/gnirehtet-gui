<script lang="ts">
  import Button from "./button.svelte";

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

  let confirmButton = $state<HTMLButtonElement | null>(null);

  $effect(() => {
    if (open) confirmButton?.focus();
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
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) cancel();
    }}
  >
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      aria-describedby={description ? "dialog-description" : undefined}
      class="w-full max-w-sm rounded-[var(--radius-lg)] border border-border bg-bg-elevated p-5 shadow-[var(--shadow)]"
    >
      <h2 id="dialog-title" class="text-base font-semibold text-fg">{title}</h2>
      {#if description}
        <p id="dialog-description" class="mt-1.5 text-sm text-fg-body">{description}</p>
      {/if}
      <div class="mt-5 flex justify-end gap-2">
        <Button variant="ghost" onclick={cancel} disabled={busy}>{cancelLabel}</Button>
        <button
          bind:this={confirmButton}
          type="button"
          class="inline-flex h-9 items-center justify-center rounded-[var(--radius)] border border-danger/30 bg-danger-bg px-4 text-sm font-medium text-danger-strong hover:bg-danger-soft disabled:opacity-50"
          onclick={() => onConfirm()}
          disabled={busy}
        >
          {confirmLabel}
        </button>
      </div>
    </div>
  </div>
{/if}
