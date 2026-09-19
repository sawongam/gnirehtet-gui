<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "default" | "secondary" | "destructive" | "ghost" | "outline";
  type Size = "default" | "sm" | "lg";

  let {
    class: className = "",
    variant = "default" as Variant,
    size = "default" as Size,
    children,
    ...rest
  }: HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  } = $props();

  const variants: Record<Variant, string> = {
    default:
      "bg-accent text-accent-fg hover:opacity-90 shadow-sm border border-transparent",
    secondary:
      "bg-bg-elevated text-fg border border-border hover:bg-bg-muted",
    destructive:
      "bg-danger text-white hover:opacity-90 border border-transparent shadow-sm",
    ghost: "bg-transparent text-fg-muted hover:bg-bg-muted hover:text-fg border border-transparent",
    outline:
      "bg-bg-elevated text-accent border border-accent/40 hover:bg-accent-muted",
  };

  const sizes: Record<Size, string> = {
    default: "h-9 px-3.5 text-sm",
    sm: "h-8 px-3 text-xs",
    lg: "h-11 px-5 text-base",
  };
</script>

<button
  type="button"
  class={cn(
    "inline-flex items-center justify-center gap-1.5 rounded-[var(--radius-sm)] font-medium transition-colors",
    variants[variant],
    sizes[size],
    // After variants so disabled look wins for secondary/ghost (same gate as Run).
    "disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-bg-muted disabled:text-fg-subtle disabled:shadow-none disabled:border-border-subtle disabled:hover:bg-bg-muted disabled:hover:text-fg-subtle disabled:hover:opacity-100",
    className,
  )}
  {...rest}
>
  {@render children?.()}
</button>
