<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Variant = "default" | "warning" | "destructive" | "info";

  let {
    class: className = "",
    variant = "default" as Variant,
    children,
    ...rest
  }: HTMLAttributes<HTMLDivElement> & {
    variant?: Variant;
    children?: Snippet;
  } = $props();

  const variants: Record<Variant, string> = {
    default: "border-border bg-bg-elevated text-fg",
    warning: "border-warning/35 bg-warning/10 text-fg",
    destructive: "border-danger/35 bg-danger/10 text-fg",
    info: "border-info/35 bg-info/10 text-fg",
  };
</script>

<div
  role="alert"
  class={cn(
    "rounded-[var(--radius)] border px-4 py-3.5 text-sm shadow-[var(--shadow)]",
    variants[variant],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</div>
