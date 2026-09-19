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
    success: "bg-success/15 text-success border-success/30",
    warning: "bg-warning/15 text-warning border-warning/30",
    danger: "bg-danger/15 text-danger border-danger/30",
    info: "bg-info/15 text-info border-info/30",
    muted: "bg-bg-muted text-fg-muted border-border-subtle",
  };
</script>

<span
  class={cn(
    "inline-flex items-center rounded-sm border px-1.5 py-0.5 text-[12px] font-medium leading-none",
    tones[tone],
    className,
  )}
  {...rest}
>
  {@render children?.()}
</span>
