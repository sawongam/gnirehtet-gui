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
    warning: "border-warning/40 bg-warning/10 text-fg",
    destructive: "border-danger/40 bg-danger/10 text-fg",
    info: "border-info/40 bg-info/10 text-fg",
  };
</script>

<div
  role="alert"
  class={cn(
    "rounded-md border px-3 py-2.5 text-sm shadow-[var(--shadow)]",
    variants[variant],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</div>
