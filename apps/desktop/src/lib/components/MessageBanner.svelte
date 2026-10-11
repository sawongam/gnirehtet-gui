<script lang="ts">
  import { X } from "@lucide/svelte";
  import Alert from "$lib/components/ui/alert.svelte";
  import { app } from "$lib/app.svelte";
  import { errorHelp } from "$lib/errors";

  const help = $derived(app.error ? errorHelp(app.error.code) : null);
</script>

{#if app.error && help}
  <Alert variant="danger" class="flex items-start gap-3">
    <div class="min-w-0 flex-1">
      <p class="font-semibold">{help.title}</p>
      <p class="mt-1 text-fg-body">{help.hint}</p>
      <p class="mt-2 break-words font-mono text-xs text-fg-muted">{app.error.message}</p>
    </div>
    <button
      type="button"
      class="rounded p-1 text-fg-muted hover:bg-bg-muted hover:text-fg"
      aria-label="Dismiss"
      onclick={() => app.dismissMessages()}
    >
      <X class="size-4" />
    </button>
  </Alert>
{:else if app.notice}
  <Alert variant="info" class="flex items-start gap-3">
    <p class="min-w-0 flex-1">{app.notice}</p>
    <button
      type="button"
      class="rounded p-1 text-fg-muted hover:bg-bg-muted hover:text-fg"
      aria-label="Dismiss"
      onclick={() => app.dismissMessages()}
    >
      <X class="size-4" />
    </button>
  </Alert>
{/if}
