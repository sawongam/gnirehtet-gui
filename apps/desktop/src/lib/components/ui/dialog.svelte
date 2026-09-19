<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import Button from "./button.svelte";

  let {
    open = $bindable(false),
    title,
    description,
    confirmLabel = "Confirm",
    cancelLabel = "Cancel",
    confirmVariant = "destructive" as "default" | "destructive",
    onConfirm,
    class: className = "",
  }: {
    open?: boolean;
    title: string;
    description?: string;
    confirmLabel?: string;
    cancelLabel?: string;
    confirmVariant?: "default" | "destructive";
    onConfirm: () => void | Promise<void>;
    class?: string;
  } = $props();

  async function confirm() {
    await onConfirm();
    open = false;
  }

  function cancel() {
    open = false;
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") cancel();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
    role="presentation"
    onkeydown={onKey}
    onclick={(e) => {
      if (e.target === e.currentTarget) cancel();
    }}
  >
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="dialog-title"
      class={cn(
        "w-full max-w-sm rounded-md border border-border bg-bg-elevated p-4 shadow-[var(--shadow)]",
        className,
      )}
    >
      <h2 id="dialog-title" class="text-base font-semibold text-fg">{title}</h2>
      {#if description}
        <p class="mt-1.5 text-sm text-fg-muted">{description}</p>
      {/if}
      <div class="mt-4 flex justify-end gap-2">
        <Button variant="ghost" onclick={cancel}>{cancelLabel}</Button>
        <Button variant={confirmVariant} onclick={confirm}>{confirmLabel}</Button>
      </div>
    </div>
  </div>
{/if}
