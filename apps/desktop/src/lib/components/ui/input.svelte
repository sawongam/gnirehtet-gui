<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    class: className = "",
    value = $bindable(""),
    invalid = false,
    ...rest
  }: HTMLInputAttributes & {
    value?: string;
    invalid?: boolean;
  } = $props();
</script>

<input
  class={cn(
    "h-9 w-full rounded-[var(--radius)] border bg-bg-elevated px-3 text-sm text-fg placeholder:text-fg-subtle",
    "focus-visible:border-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/25",
    "disabled:cursor-not-allowed disabled:opacity-50",
    invalid ? "border-danger" : "border-border",
    className,
  )}
  aria-invalid={invalid || undefined}
  bind:value
  {...rest}
/>
