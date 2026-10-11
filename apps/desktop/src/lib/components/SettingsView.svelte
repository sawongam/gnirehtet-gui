<script lang="ts">
  import { onMount } from "svelte";
  import { CircleCheck, CircleX } from "@lucide/svelte";
  import Button from "$lib/components/ui/button.svelte";
  import Card from "$lib/components/ui/card.svelte";
  import Input from "$lib/components/ui/input.svelte";
  import { api, isTauri } from "$lib/api";
  import { app } from "$lib/app.svelte";
  import { settings, type Theme } from "$lib/settings.svelte";
  import type { Environment } from "$lib/types";
  import { DEFAULT_PORT, validateDnsServers, validatePort, validateRoutes } from "$lib/validation";

  let port = $state(String(settings.port));
  let dnsServers = $state(settings.dnsServers);
  let routes = $state(settings.routes);
  let saved = $state(false);
  let environment = $state<Environment | null>(null);

  const portError = $derived(validatePort(port));
  const dnsError = $derived(validateDnsServers(dnsServers));
  const routesError = $derived(validateRoutes(routes));
  const valid = $derived(!portError && !dnsError && !routesError);
  const dirty = $derived(
    Number(port) !== settings.port || dnsServers !== settings.dnsServers || routes !== settings.routes,
  );

  function save(event: SubmitEvent) {
    event.preventDefault();
    if (!valid) return;
    settings.update({ port: Number(port), dnsServers: dnsServers.trim(), routes: routes.trim() });
    saved = true;
    setTimeout(() => (saved = false), 2000);
  }

  function restoreDefaults() {
    port = String(DEFAULT_PORT);
    dnsServers = "";
    routes = "";
  }

  onMount(async () => {
    if (isTauri()) environment = await api.getEnvironment().catch(() => null);
  });

  const themes: { value: Theme; label: string }[] = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
</script>

<div class="mx-auto flex max-w-3xl flex-col gap-5 p-6 pb-10">
  <div>
    <h1 class="text-2xl font-semibold tracking-tight text-fg">Settings</h1>
    <p class="text-sm text-fg-muted">Connection options apply the next time you connect.</p>
  </div>

  <Card class="p-5">
    <h2 class="text-[15px] font-semibold text-fg">Connection</h2>
    <form class="mt-4 space-y-4" onsubmit={save} novalidate>
      <div>
        <label for="port" class="block text-sm font-medium text-fg">Relay port</label>
        <Input id="port" inputmode="numeric" bind:value={port} invalid={!!portError} class="mt-1 max-w-40" />
        <p class="mt-1 text-xs {portError ? 'text-danger' : 'text-fg-muted'}">
          {portError ?? `Local port used by the relay. Default: ${DEFAULT_PORT}.`}
        </p>
      </div>
      <div>
        <label for="dns" class="block text-sm font-medium text-fg">DNS servers</label>
        <Input id="dns" placeholder="8.8.8.8" bind:value={dnsServers} invalid={!!dnsError} class="mt-1" />
        <p class="mt-1 text-xs {dnsError ? 'text-danger' : 'text-fg-muted'}">
          {dnsError ?? "Comma-separated IPv4 addresses used by the phone. Empty: 8.8.8.8."}
        </p>
      </div>
      <div>
        <label for="routes" class="block text-sm font-medium text-fg">Routes</label>
        <Input id="routes" placeholder="0.0.0.0/0" bind:value={routes} invalid={!!routesError} class="mt-1" />
        <p class="mt-1 text-xs {routesError ? 'text-danger' : 'text-fg-muted'}">
          {routesError ??
            "Comma-separated IPv4 networks sent through the computer (e.g. 10.0.0.0/8). Empty: all traffic."}
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Button type="submit" disabled={!valid || !dirty}>Save</Button>
        <Button variant="ghost" onclick={restoreDefaults}>Restore defaults</Button>
        {#if saved}<span class="text-xs text-success-strong">Saved</span>{/if}
        {#if app.sessionActive && dirty}
          <span class="text-xs text-fg-muted">Reconnect to apply.</span>
        {/if}
      </div>
    </form>
  </Card>

  <Card class="p-5">
    <h2 class="text-[15px] font-semibold text-fg">Appearance</h2>
    <div class="mt-3 flex gap-2" role="radiogroup" aria-label="Theme">
      {#each themes as theme (theme.value)}
        <Button
          variant={settings.theme === theme.value ? "primary" : "secondary"}
          size="sm"
          role="radio"
          aria-checked={settings.theme === theme.value}
          onclick={() => settings.update({ theme: theme.value })}
        >
          {theme.label}
        </Button>
      {/each}
    </div>
  </Card>

  <Card class="p-5">
    <h2 class="text-[15px] font-semibold text-fg">Diagnostics</h2>
    {#if environment}
      <dl class="mt-3 space-y-3 text-sm">
        <div>
          <dt class="flex items-center gap-1.5 font-medium text-fg">
            {#if app.adb}<CircleCheck class="size-4 text-success" />{:else}<CircleX class="size-4 text-danger" />{/if}
            adb
          </dt>
          <dd class="break-all font-mono text-xs text-fg-muted">
            {environment.adbPath}{app.adb ? ` · ${app.adb.version}` : " · not working"}
          </dd>
        </div>
        <div>
          <dt class="flex items-center gap-1.5 font-medium text-fg">
            {#if environment.relayFound}<CircleCheck class="size-4 text-success" />{:else}<CircleX class="size-4 text-danger" />{/if}
            Relay
          </dt>
          <dd class="break-all font-mono text-xs text-fg-muted">{environment.relayPath}</dd>
        </div>
        <div>
          <dt class="flex items-center gap-1.5 font-medium text-fg">
            {#if environment.apkFound}<CircleCheck class="size-4 text-success" />{:else}<CircleX class="size-4 text-danger" />{/if}
            Client APK
          </dt>
          <dd class="break-all font-mono text-xs text-fg-muted">{environment.apkPath}</dd>
        </div>
      </dl>
      <p class="mt-4 text-xs text-fg-muted">
        Override with the <code>ADB</code>, <code>GNIREHTET_BIN</code> and <code>GNIREHTET_APK</code> environment
        variables.
      </p>
    {:else}
      <p class="mt-2 text-sm text-fg-muted">Unavailable outside the desktop app.</p>
    {/if}
  </Card>

  <Card class="p-5">
    <h2 class="text-[15px] font-semibold text-fg">About</h2>
    <p class="mt-2 text-sm text-fg-body">
      gnirehtet-gui {environment ? `v${environment.appVersion}` : ""} is a desktop front-end for
      gnirehtet by Genymobile (relay and client v2.5.1). Licensed under the Apache License 2.0.
    </p>
    <p class="mt-1 font-mono text-xs text-fg-muted">https://github.com/Genymobile/gnirehtet</p>
  </Card>
</div>
