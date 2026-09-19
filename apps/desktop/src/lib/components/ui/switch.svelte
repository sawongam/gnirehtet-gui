<script lang="ts">
  import { cn } from "$lib/utils";
  import type { HTMLButtonAttributes } from "svelte/elements";

  let {
    class: className = "",
    checked = $bindable(false),
    disabled = false,
    label = "",
    ...rest
  }: HTMLButtonAttributes & {
    checked?: boolean;
    label?: string;
  } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label || undefined}
  {disabled}
  class={cn(
    "relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border transition-colors",
    checked ? "border-accent bg-accent" : "border-border bg-bg-muted",
    "disabled:cursor-not-allowed disabled:opacity-50",
    className,
  )}
  onclick={() => {
    if (!disabled) checked = !checked;
  }}
  {...rest}
>
  <span
    class={cn(
      "pointer-events-none inline-block size-4 rounded-full bg-white shadow transition-transform",
      checked ? "translate-x-4" : "translate-x-0.5",
    )}
  ></span>
</button>
