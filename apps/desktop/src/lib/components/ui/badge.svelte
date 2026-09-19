<script lang="ts">
  import { cn } from "$lib/utils";
  import type { Snippet } from "svelte";
  import type { HTMLAttributes } from "svelte/elements";

  type Tone = "default" | "success" | "warning" | "danger" | "info" | "muted";

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
    success: "bg-success/15 text-success border-success/25",
    warning: "bg-warning/15 text-warning border-warning/25",
    danger: "bg-danger/15 text-danger border-danger/25",
    info: "bg-info/15 text-info border-info/25",
    muted: "bg-bg-muted text-fg-muted border-border-subtle",
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
