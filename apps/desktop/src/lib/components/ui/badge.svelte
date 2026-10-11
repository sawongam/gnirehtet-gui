<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Tone = "accent" | "success" | "warning" | "danger" | "muted";

  let {
    class: className = "",
    tone = "muted",
    children,
    ...rest
  }: HTMLAttributes<HTMLSpanElement> & {
    tone?: Tone;
    children?: Snippet;
  } = $props();

  const tones: Record<Tone, string> = {
    accent: "bg-accent-50 text-accent border-accent/25",
    success: "bg-success-soft text-success-strong border-success/25",
    warning: "bg-warning-soft text-warning-strong border-warning/25",
    danger: "bg-danger-soft text-danger-strong border-danger/25",
    muted: "bg-bg-muted text-fg-muted border-border",
  };
</script>

<span
  class={cn(
    "inline-flex items-center gap-1.5 whitespace-nowrap rounded-full border px-2 py-0.5 text-xs font-medium",
    tones[tone],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</span>
