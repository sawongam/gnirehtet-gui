<script lang="ts">
  import { onMount } from "svelte";
  import { api, isTauri } from "$lib/api";
  import { initialStartupState, StartupPreference } from "$lib/startup";

  let startup = $state({ ...initialStartupState });
  const preference = new StartupPreference(api, (state) => (startup = state));
  let available = $state(false);

  onMount(() => {
    available = isTauri();
    if (available) void preference.load();
  });
</script>

<section class="group">
  <div class="group-head"><h2 class="group-title">Startup</h2></div>
  <div class="list">
    <label class="row" for="run-at-startup">
      <span class="row-text">
        <span class="row-title">Run at startup</span>
        <span class="row-sub">
          {#if !available}
            Available in the desktop app.
          {:else if startup.busy}
            Checking startup settings…
          {:else}
            Start Gnirehtet when you sign in. Connect your phone manually.
          {/if}
        </span>
      </span>
      <input
        id="run-at-startup"
        type="checkbox"
        checked={startup.enabled}
        disabled={!available || !startup.loaded || startup.busy}
        onchange={(event) => preference.setEnabled(event.currentTarget.checked)}
      />
    </label>
  </div>
  {#if startup.error}
    <div class="startup-error">
      <p class="group-note tone-danger" role="alert">{startup.error}</p>
      <button class="btn small" type="button" disabled={startup.busy} onclick={() => preference.load()}>
        Try again
      </button>
    </div>
  {/if}
</section>

<style>
  input {
    accent-color: var(--accent);
  }
  .startup-error {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 12px;
  }
  .startup-error .group-note {
    flex: 1;
    margin: 0;
  }
</style>
