<script lang="ts">
  import { CircleCheck, Circle, CircleAlert } from "@lucide/svelte";
  import Card from "$lib/components/ui/card.svelte";
  import { app } from "$lib/app.svelte";
  import { isReady } from "$lib/session-logic";

  type Step = { done: boolean; problem: boolean; title: string; detail: string };

  const steps = $derived.by((): Step[] => {
    const hasDevice = app.devices.length > 0;
    const ready = app.devices.some(isReady);
    return [
      {
        done: !!app.adb,
        problem: app.loaded && !app.adb,
        title: "Install adb",
        detail: app.adb
          ? app.adb.version
          : "Install Android platform-tools (e.g. “sudo apt install adb”) or set the ADB environment variable.",
      },
      {
        done: ready,
        problem: hasDevice && !ready,
        title: "Connect and authorize the phone",
        detail: ready
          ? "USB debugging is authorized."
          : "Enable USB debugging in developer options, plug the phone in and accept the prompt.",
      },
      {
        done: app.phase === "connected",
        problem: false,
        title: "Connect and accept the VPN request",
        detail: "Press Connect, then accept the “Connection request” shown on the phone.",
      },
    ];
  });
</script>

<Card class="p-5">
  <h2 class="text-[15px] font-semibold text-fg">Getting started</h2>
  <ol class="mt-3 space-y-3">
    {#each steps as step, i (step.title)}
      <li class="flex gap-3">
        {#if step.done}
          <CircleCheck class="mt-0.5 size-4 shrink-0 text-success" aria-label="Done" />
        {:else if step.problem}
          <CircleAlert class="mt-0.5 size-4 shrink-0 text-warning" aria-label="Action needed" />
        {:else}
          <Circle class="mt-0.5 size-4 shrink-0 text-fg-subtle" aria-label="To do" />
        {/if}
        <div class="min-w-0">
          <p class="text-sm font-medium text-fg">{i + 1}. {step.title}</p>
          <p class="break-words text-xs text-fg-muted">{step.detail}</p>
        </div>
      </li>
    {/each}
  </ol>
</Card>
