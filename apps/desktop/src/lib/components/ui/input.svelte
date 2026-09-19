<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    class: className = "",
    value = $bindable(""),
    ...rest
  }: HTMLInputAttributes & {
    value?: string;
  } = $props();
</script>

<input
  class={cn(
    "h-10 w-full rounded-full border border-border bg-bg-elevated px-4 text-sm text-fg placeholder:text-fg-subtle",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent/30 focus-visible:border-accent",
    "disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  bind:value
  {...rest}
/>
