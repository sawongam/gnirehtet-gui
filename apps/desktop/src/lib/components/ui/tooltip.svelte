<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    content,
    class: className = "",
    children,
  }: {
    content: string;
    class?: string;
    children: Snippet;
  } = $props();
</script>

<span class={cn("group relative inline-flex", className)} title={content}>
  {@render children()}
  <span
    role="tooltip"
    class="pointer-events-none absolute bottom-full left-1/2 z-50 mb-1.5 hidden w-max max-w-[220px] -translate-x-1/2 rounded-md border border-border bg-bg-elevated px-2 py-1 text-[12px] leading-snug text-fg shadow-[var(--shadow)] group-hover:block group-focus-within:block"
  >
    {content}
  </span>
</span>
