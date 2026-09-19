<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Tone = "default" | "success" | "warning" | "danger" | "info" | "muted" | "primary";

  let {
    class: className = "",
    tone = "default" as Tone,
    children,
    ...rest
  }: HTMLAttributes<HTMLSpanElement> & {
    tone?: Tone;
    children?: Snippet;
  } = $props();

  const tones: Record<Tone, string> = {
    default: "bg-bg-muted text-fg border-border",
    success: "bg-success-soft text-success-strong border-success/25",
    warning: "bg-warning-soft text-warning-strong border-warning/25",
    danger: "bg-danger-soft text-danger-strong border-danger/25",
    info: "bg-accent-50 text-accent border-accent/20",
    muted: "bg-bg-muted text-fg-muted border-border-subtle",
    primary: "bg-accent-50 text-accent border-accent/25",
  };
</script>

<span
  class={cn(
    "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium leading-none",
    tones[tone],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</span>
