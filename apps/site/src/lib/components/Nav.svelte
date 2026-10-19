<script lang="ts">
  import { base } from "$app/paths";
  import { theme } from "$lib/theme.svelte";
  import { REPO } from "$lib/site";

  let scrolled = $state(false);
</script>

<svelte:window onscroll={() => (scrolled = scrollY > 8)} />

<nav class:scrolled>
  <div class="wrap bar">
    <a class="brand" href="{base}/">
      <img src="{base}/icon.svg" alt="" width="28" height="28" />
      <span>Gnirehtet</span>
    </a>
    <div class="links">
      <a class="hide-sm" href="#how">How it works</a>
      <a class="hide-sm" href="#download">Download</a>
      <a href={REPO}>GitHub</a>
      <button
        type="button"
        class="theme"
        onclick={() => theme.toggle()}
        aria-label={theme.dark ? "Use light theme" : "Use dark theme"}
        title={theme.dark ? "Light theme" : "Dark theme"}
      >
        <svg viewBox="0 0 20 20" aria-hidden="true">
          <circle cx="10" cy="10" r="7" />
          <path d="M10 3a7 7 0 0 1 0 14Z" />
        </svg>
      </button>
    </div>
  </div>
</nav>

<style>
  nav {
    position: sticky;
    top: 0;
    z-index: 10;
    border-bottom: 1px solid transparent;
    transition:
      background 250ms var(--ease),
      border-color 250ms var(--ease);
  }
  nav.scrolled {
    background: color-mix(in srgb, var(--page) 78%, transparent);
    border-bottom-color: var(--line);
    backdrop-filter: saturate(1.6) blur(16px);
    -webkit-backdrop-filter: saturate(1.6) blur(16px);
  }

  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 60px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--fg);
    font-family: var(--font-display);
    font-size: 19px;
    font-weight: 650;
    letter-spacing: -0.02em;
    text-decoration: none !important;
  }

  .links {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .links a {
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 14.5px;
    font-weight: 600;
    color: var(--fg-dim);
    text-decoration: none;
    transition: background 150ms var(--ease), color 150ms var(--ease);
  }
  .links a:hover {
    color: var(--fg);
    background: var(--fill);
  }

  .theme {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    margin-left: 4px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    color: var(--fg-dim);
    background: transparent;
    cursor: pointer;
    transition: background 150ms var(--ease), color 150ms var(--ease);
  }
  .theme:hover {
    color: var(--fg);
    background: var(--fill);
  }
  .theme svg {
    width: 18px;
    height: 18px;
    transition: rotate 500ms var(--ease-out);
  }
  :global(:root.dark) .theme svg {
    rotate: 180deg;
  }
  .theme circle {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
  }
  .theme path {
    fill: currentColor;
  }

  @media (max-width: 560px) {
    .hide-sm {
      display: none;
    }
  }
</style>
