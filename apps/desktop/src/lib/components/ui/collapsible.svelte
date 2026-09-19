<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    class: className = "",
    trigger,
    children,
  }: {
    open?: boolean;
    class?: string;
    trigger: Snippet<[{ open: boolean; toggle: () => void }]>;
    children: Snippet;
  } = $props();

  function toggle() {
    open = !open;
  }
</script>

<div class={cn("w-full", className)} data-state={open ? "open" : "closed"}>
  {@render trigger({ open, toggle })}
  {#if open}
    <div class="mt-2">
      {@render children()}
    </div>
  {/if}
</div>
