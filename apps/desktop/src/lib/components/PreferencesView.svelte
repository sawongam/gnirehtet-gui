<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "$lib/api";
  import { app } from "$lib/app.svelte";
  import { settings, type Theme } from "$lib/settings.svelte";
  import { isReady } from "$lib/session-logic";
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
  const isDefault = $derived(Number(port) === DEFAULT_PORT && dnsServers === "" && routes === "");

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

  const relayValue = $derived(
    app.relay.state === "running" && app.relay.port
      ? `Running on port ${app.relay.port}`
      : app.relay.state === "crashed"
        ? "Stopped unexpectedly"
        : "Not running",
  );

  const themes: { value: Theme; label: string }[] = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
</script>

<div class="clamp">
  <form class="group first" onsubmit={save} novalidate>
    <div class="group-head"><h2 class="group-title">Connection</h2></div>
    <div class="list">
      <label class="row" for="port">
        <span class="row-text">
          <span class="row-title">Relay port</span>
          <span class="row-sub" class:tone-danger={portError}>{portError ?? `Default ${DEFAULT_PORT}`}</span>
        </span>
        <input id="port" class="entry numeric narrow" inputmode="numeric" bind:value={port} aria-invalid={!!portError} />
      </label>
      <label class="row" for="dns">
        <span class="row-text">
          <span class="row-title">DNS servers</span>
          <span class="row-sub" class:tone-danger={dnsError}>{dnsError ?? "IPv4, comma-separated. Empty uses 8.8.8.8."}</span>
        </span>
        <input id="dns" class="entry wide" placeholder="8.8.8.8" bind:value={dnsServers} aria-invalid={!!dnsError} />
      </label>
      <label class="row" for="routes">
        <span class="row-text">
          <span class="row-title">Routes</span>
          <span class="row-sub" class:tone-danger={routesError}>{routesError ?? "Networks sent through this computer. Empty sends everything."}</span>
        </span>
        <input id="routes" class="entry wide" placeholder="0.0.0.0/0" bind:value={routes} aria-invalid={!!routesError} />
      </label>
    </div>
    <div class="form-actions">
      <p class="group-note">
        {#if saved}
          <span class="tone-success">Saved.</span>
        {:else if app.sessionActive && dirty}
          Changes apply after you reconnect.
        {:else}
          Applies the next time you connect.
        {/if}
      </p>
      <button type="button" class="btn flat" onclick={restoreDefaults} disabled={isDefault}>Reset</button>
      <button type="submit" class="btn suggested" disabled={!valid || !dirty}>Save</button>
    </div>
  </form>

  <section class="group">
    <div class="group-head"><h2 class="group-title">Phone</h2></div>
    <div class="list">
      <div class="row">
        <span class="row-text">
          <span class="row-title">Client app</span>
          <span class="row-sub">Installed on the phone automatically the first time you connect.</span>
        </span>
        <button
          type="button"
          class="btn small"
          onclick={() => app.installClient()}
          disabled={app.sessionActive || app.busy || !isReady(app.selectedDevice)}
        >
          {app.task === "install" ? "Installing…" : "Reinstall"}
        </button>
      </div>
    </div>
  </section>

  <section class="group">
    <div class="group-head"><h2 class="group-title">Appearance</h2></div>
    <div class="list">
      <div class="row">
        <span class="row-text"><span class="row-title">Style</span></span>
        <div class="segmented" role="radiogroup" aria-label="Style">
          {#each themes as theme (theme.value)}
            <button
              type="button"
              role="radio"
              aria-checked={settings.theme === theme.value}
              onclick={() => settings.update({ theme: theme.value })}
            >
              {theme.label}
            </button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section class="group">
    <div class="group-head"><h2 class="group-title">Diagnostics</h2></div>
    {#if environment}
      <div class="list">
        <div class="row">
          <span class="row-text">
            <span class="row-title">adb</span>
            <span class="row-sub mono selectable">{environment.adbPath}{app.adb ? ` · ${app.adb.version}` : ""}</span>
          </span>
          <span class="row-end {app.adb ? 'tone-success' : 'tone-danger'}">{app.adb ? "Working" : "Not working"}</span>
        </div>
        <div class="row">
          <span class="row-text">
            <span class="row-title">Relay</span>
            <span class="row-sub mono selectable">{environment.relayPath}</span>
            <span class="row-sub numeric" class:tone-danger={app.relay.state === "crashed"}>{relayValue}</span>
          </span>
          <span class="row-end {environment.relayFound ? 'tone-success' : 'tone-danger'}">
            {environment.relayFound ? "Found" : "Missing"}
          </span>
        </div>
        <div class="row">
          <span class="row-text">
            <span class="row-title">Client APK</span>
            <span class="row-sub mono selectable">{environment.apkPath}</span>
          </span>
          <span class="row-end {environment.apkFound ? 'tone-success' : 'tone-danger'}">
            {environment.apkFound ? "Found" : "Missing"}
          </span>
        </div>
      </div>
      <p class="group-note">
        Override these with the <span class="mono">ADB</span>, <span class="mono">GNIREHTET_BIN</span> and
        <span class="mono">GNIREHTET_APK</span> environment variables.
      </p>
    {:else}
      <div class="list">
        <div class="row"><span class="row-text row-sub">Only available in the desktop app.</span></div>
      </div>
    {/if}
  </section>

  <section class="group">
    <div class="group-head"><h2 class="group-title">About</h2></div>
    <div class="list">
      <div class="row">
        <span class="row-text">
          <span class="row-title">Gnirehtet{environment ? ` ${environment.appVersion}` : ""}</span>
          <span class="row-sub">
            Desktop app for gnirehtet 2.5.1 by Genymobile. Apache License 2.0.
          </span>
        </span>
      </div>
      <div class="row">
        <span class="row-text"><span class="row-title">Upstream</span></span>
        <span class="row-end mono selectable">github.com/Genymobile/gnirehtet</span>
      </div>
    </div>
  </section>
</div>

<style>
  .first {
    margin-top: 4px;
  }

  label.row {
    cursor: default;
  }
  label.row:hover {
    background: none;
  }

  .entry.narrow {
    width: 96px;
    text-align: right;
  }
  .entry.wide {
    width: 200px;
  }

  .form-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
  }
  .form-actions .group-note {
    flex: 1;
    margin: 0 2px;
  }
</style>
