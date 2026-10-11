<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  let {
    class: className = "",
    children,
    ...rest
  }: HTMLAttributes<HTMLElement> & { children?: Snippet } = $props();
</script>

<section
  class={cn("rounded-[var(--radius-lg)] border border-border bg-bg-elevated shadow-[var(--shadow)]", className)}
  {...rest}
>
  {@render children?.()}
</section>
