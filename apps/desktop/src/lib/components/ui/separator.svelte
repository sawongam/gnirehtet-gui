<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLAttributes } from "svelte/elements";

  let {
    class: className = "",
    ...rest
  }: HTMLAttributes<HTMLDivElement> = $props();
</script>

<div
  role="separator"
  class={cn("h-px w-full bg-border-subtle", className)}
  {...rest}
></div>
