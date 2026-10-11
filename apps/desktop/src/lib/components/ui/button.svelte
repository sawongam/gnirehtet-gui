<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "destructive" | "ghost";
  type Size = "md" | "sm";

  let {
    class: className = "",
    variant = "primary",
    size = "md",
    children,
    ...rest
  }: HTMLButtonAttributes & {
    variant?: Variant;
    size?: Size;
    children?: Snippet;
  } = $props();

  const variants: Record<Variant, string> = {
    primary: "border-transparent bg-accent text-accent-fg shadow-sm hover:bg-accent-600 active:bg-accent-700",
    secondary: "border-border bg-bg-elevated text-fg hover:bg-bg-muted",
    destructive: "border-danger/30 bg-danger-bg text-danger-strong hover:bg-danger-soft",
    ghost: "border-transparent bg-transparent text-fg-muted hover:bg-bg-muted hover:text-fg",
  };

  const sizes: Record<Size, string> = {
    md: "h-9 px-4 text-sm",
    sm: "h-8 px-3 text-xs",
  };
</script>

<button
  type="button"
  class={cn(
    "inline-flex items-center justify-center gap-1.5 rounded-[var(--radius)] border font-medium transition-colors",
    "disabled:pointer-events-none disabled:opacity-50",
    variants[variant],
    sizes[size],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</button>
