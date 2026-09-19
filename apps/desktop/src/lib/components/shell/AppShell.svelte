<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    LayoutDashboard,
    Smartphone,
    Activity,
    ScrollText,
    Settings,
    Search,
    Sun,
    Moon,
    Radio,
    Info,
  } from "@lucide/svelte";
  import Badge from "$lib/components/ui/badge.svelte";
  import Input from "$lib/components/ui/input.svelte";
  import { cn } from "$lib/utils";
  import type { NavId } from "./types";

  let {
    activeNav = $bindable("dashboard" as NavId),
    adbOk = false,
    adbLabel = "ADB …",
    serviceRunning = false,
    deviceCount = 0,
    version = "0.1.0",
    children,
  }: {
    activeNav?: NavId;
    adbOk?: boolean;
    adbLabel?: string;
    serviceRunning?: boolean;
    deviceCount?: number;
    version?: string;
    children?: Snippet;
  } = $props();

  let search = $state("");
  let dark = $state(false);

  const navItems: { id: NavId; label: string; icon: typeof LayoutDashboard }[] = [
    { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
    { id: "devices", label: "Devices", icon: Smartphone },
    { id: "traffic", label: "Traffic", icon: Activity },
    { id: "logs", label: "Logs", icon: ScrollText },
    { id: "settings", label: "Settings", icon: Settings },
  ];

  function toggleTheme() {
    dark = !dark;
    document.documentElement.classList.toggle("dark", dark);
  }
</script>

<div class="flex h-screen min-h-0 overflow-hidden bg-bg text-fg">
  <!-- Sidebar ~240px -->
  <aside
    class="flex w-[var(--sidebar-width)] shrink-0 flex-col border-r border-border bg-bg-elevated"
    aria-label="Main navigation"
  >
    <div class="flex items-center gap-3 px-5 py-5">
      <div
        class="flex size-9 items-center justify-center rounded-[var(--radius)] bg-accent text-accent-fg shadow-sm"
      >
        <Radio class="size-5" strokeWidth={2} />
      </div>
      <div class="min-w-0">
        <p class="truncate text-[15px] font-semibold leading-tight text-fg">Gnirehtet</p>
        <p class="truncate text-xs text-fg-muted">Reverse Tethering</p>
      </div>
    </div>

    <nav class="flex flex-1 flex-col gap-1 px-3">
      {#each navItems as item}
        {@const Icon = item.icon}
        {@const active = activeNav === item.id}
        <button
          type="button"
          class={cn(
            "flex items-center gap-3 rounded-[var(--radius)] px-3 py-2.5 text-sm font-medium transition-colors",
            active
              ? "bg-accent-50 text-accent"
              : "text-fg-body hover:bg-bg-muted hover:text-fg",
          )}
          aria-current={active ? "page" : undefined}
          onclick={() => (activeNav = item.id)}
        >
          <Icon class="size-4 shrink-0" strokeWidth={active ? 2.25 : 2} />
          <span class="flex-1 text-left">{item.label}</span>
          {#if item.id === "devices" && deviceCount > 0}
            <span
              class="inline-flex min-w-5 items-center justify-center rounded-full bg-accent px-1.5 py-0.5 text-[11px] font-semibold text-accent-fg"
            >
              {deviceCount}
            </span>
          {/if}
        </button>
      {/each}
    </nav>

    <div class="mx-3 mb-3 rounded-[var(--radius-lg)] border border-accent/15 bg-accent-50 p-3.5">
      <div class="flex items-start gap-2">
        <Info class="mt-0.5 size-4 shrink-0 text-accent" strokeWidth={2} />
        <p class="text-xs leading-relaxed text-fg-body">
          Android internet via your computer.
        </p>
      </div>
      <div class="mt-3 flex items-end justify-center gap-2 opacity-80" aria-hidden="true">
        <div class="rounded-md border border-accent/20 bg-bg-elevated px-2 py-1.5">
          <div class="h-3 w-8 rounded-sm bg-accent/20"></div>
          <div class="mt-1 h-2 w-6 rounded-sm bg-accent/10"></div>
        </div>
        <div class="mb-2 h-px w-4 border-t border-dashed border-accent/40"></div>
        <div class="rounded-lg border border-accent/20 bg-bg-elevated px-1.5 py-2">
          <div class="h-4 w-3 rounded-sm bg-accent/20"></div>
        </div>
      </div>
    </div>

    <div class="flex items-center justify-between border-t border-border px-4 py-3">
      <div class="flex items-center gap-2 text-xs">
        <span
          class={cn(
            "inline-block size-1.5 rounded-full",
            adbOk ? "bg-success" : "bg-fg-subtle",
          )}
          aria-hidden="true"
        ></span>
        <span class={adbOk ? "text-success-strong" : "text-fg-muted"}>{adbLabel}</span>
      </div>
      <span class="font-mono text-[11px] text-fg-subtle">v{version}</span>
    </div>
  </aside>

  <!-- Main column -->
  <div class="flex min-w-0 flex-1 flex-col">
    <header
      class="flex h-14 shrink-0 items-center gap-3 border-b border-border bg-bg-elevated px-5"
    >
      <div class="relative mx-auto w-full max-w-xl">
        <Search
          class="pointer-events-none absolute left-3.5 top-1/2 size-4 -translate-y-1/2 text-fg-subtle"
          strokeWidth={2}
        />
        <Input
          class="pl-10"
          placeholder="Search devices, logs, or settings…"
          bind:value={search}
          aria-label="Search (visual)"
        />
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <button
          type="button"
          class="inline-flex size-9 items-center justify-center rounded-[var(--radius)] text-fg-muted hover:bg-bg-muted hover:text-fg"
          aria-label={dark ? "Switch to light theme" : "Switch to dark theme"}
          onclick={toggleTheme}
        >
          {#if dark}
            <Moon class="size-4" />
          {:else}
            <Sun class="size-4" />
          {/if}
        </button>
        <Badge tone={serviceRunning ? "success" : "muted"}>
          <span
            class={cn(
              "inline-block size-1.5 rounded-full",
              serviceRunning ? "bg-success" : "bg-fg-subtle",
            )}
            aria-hidden="true"
          ></span>
          {serviceRunning ? "Service running" : "Service idle"}
        </Badge>
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto">
      {@render children?.()}
    </div>
  </div>
</div>
