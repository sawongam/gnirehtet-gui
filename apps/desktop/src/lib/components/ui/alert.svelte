<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Variant = "info" | "success" | "warning" | "danger";

  let {
    class: className = "",
    variant = "info",
    children,
    ...rest
  }: HTMLAttributes<HTMLDivElement> & {
    variant?: Variant;
    children?: Snippet;
  } = $props();

  const variants: Record<Variant, string> = {
    info: "border-accent/30 bg-accent-50",
    success: "border-success/30 bg-success-bg",
    warning: "border-warning/35 bg-warning-bg",
    danger: "border-danger/35 bg-danger-bg",
  };
</script>

<div
  role={variant === "danger" ? "alert" : "status"}
  class={cn("rounded-[var(--radius)] border px-4 py-3 text-sm text-fg", variants[variant], className)}
  {...rest}
>
  {@render children?.()}
</div>
