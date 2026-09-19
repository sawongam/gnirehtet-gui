<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  let {
    class: className = "",
    selected = false,
    children,
    ...rest
  }: HTMLAttributes<HTMLDivElement> & {
    selected?: boolean;
    children?: Snippet;
  } = $props();
</script>

<div
  class={cn(
    "rounded-md border bg-bg-elevated shadow-[var(--shadow)] transition-colors",
    selected
      ? "border-accent bg-accent-muted ring-1 ring-accent/40"
      : "border-border",
    className,
  )}
  {...rest}
>
  {@render children?.()}
</div>
