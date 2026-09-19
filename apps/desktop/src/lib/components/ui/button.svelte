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
      "bg-accent text-accent-fg hover:bg-accent-600 active:bg-accent-700 shadow-sm border border-transparent",
    secondary:
      "bg-bg-elevated text-fg border border-border hover:bg-bg-muted",
    /* V3: soft red fill — not solid red blocks */
    destructive:
      "bg-danger-bg text-danger border border-danger-soft hover:bg-danger-soft",
    ghost: "bg-transparent text-fg-muted hover:bg-bg-muted hover:text-fg border border-transparent",
    outline:
      "bg-bg-elevated text-accent border border-accent/40 hover:bg-accent-50",
  };

  const sizes: Record<Size, string> = {
    default: "h-10 px-4 text-sm", /* 40px primary */
    sm: "h-8 px-3 text-xs",
    lg: "h-11 px-5 text-base",
  };
</script>

<button
  type="button"
  class={cn(
    "inline-flex items-center justify-center gap-1.5 rounded-[var(--radius)] font-medium transition-colors",
    variants[variant],
    sizes[size],
    "disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-bg-muted disabled:text-fg-subtle disabled:shadow-none disabled:border-border-subtle disabled:hover:bg-bg-muted disabled:hover:text-fg-subtle disabled:hover:opacity-100",
    className,
  )}
  {...rest}
>
  {@render children?.()}
</button>
