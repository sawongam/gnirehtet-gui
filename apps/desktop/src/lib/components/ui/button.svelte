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
      "bg-bg-muted text-fg border border-border hover:bg-border-subtle",
    destructive:
      "bg-danger text-white hover:opacity-90 border border-transparent",
    ghost: "bg-transparent text-fg-muted hover:bg-bg-muted hover:text-fg border border-transparent",
    outline:
      "bg-bg-elevated text-fg border border-border hover:bg-bg-muted",
  };

  const sizes: Record<Size, string> = {
    default: "h-8 px-3 text-sm",
    sm: "h-7 px-2.5 text-xs",
    lg: "h-9 px-4 text-base",
  };
</script>

<button
  type="button"
  class={cn(
    "inline-flex items-center justify-center gap-1.5 rounded-md font-medium transition-colors disabled:pointer-events-none disabled:cursor-not-allowed disabled:bg-bg-muted disabled:text-fg-muted disabled:opacity-100 disabled:shadow-none disabled:border-border disabled:hover:bg-bg-muted disabled:hover:opacity-100",
    variants[variant],
    sizes[size],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</button>
