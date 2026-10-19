<script lang="ts">
  import { base } from "$app/paths";
  import { REPO } from "$lib/site";

  let scrolled = $state(false);
</script>

<svelte:window onscroll={() => (scrolled = scrollY > 8)} />

<nav class:scrolled>
  <div class="wrap bar">
    <a class="brand" href="{base}/">
      <img src="{base}/icon.svg" alt="" width="28" height="28" />
      <span>Gnirehtet GUI</span>
    </a>
    <div class="links">
      <a class="hide-sm" href="#how">How it works</a>
      <a class="hide-sm" href="#download">Download</a>
      <a href={REPO}>GitHub</a>
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

  @media (max-width: 560px) {
    .hide-sm {
      display: none;
    }
  }
</style>
