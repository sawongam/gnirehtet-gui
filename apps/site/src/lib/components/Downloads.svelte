<script lang="ts">
  import { onMount } from "svelte";
  import { reveal } from "$lib/reveal";
  import { DOCS, DOWNLOADS, RELEASES, VERSION, detectOs, fileName, fileUrl, type Os } from "$lib/site";

  let os = $state<Os | null>(null);
  onMount(() => (os = detectOs()));
</script>

<section id="download" class="download">
  <div class="wrap grid">
    <header>
      <p class="eyebrow" use:reveal>Download</p>
      <h2 class="section-title" use:reveal={60}>Version {VERSION}</h2>
      <p class="lede" use:reveal={120}>
        Free and open source. The relay and the Android client are built in, so this is the only thing to download.
      </p>
      <dl class="needs" use:reveal={180}>
        <div>
          <dt>Phone</dt>
          <dd>Android 5.0 or newer, no root</dd>
        </div>
        <div>
          <dt>Computer</dt>
          <dd>adb, from Android platform-tools</dd>
        </div>
      </dl>
      <p class="more" use:reveal={240}>
        <a href="{DOCS}/INSTALL.md">Installation guide</a>
        <a href={RELEASES}>All files and checksums</a>
      </p>
    </header>

    <ul class="boxed-list files" use:reveal={100}>
      {#each DOWNLOADS as d (d.suffix)}
        <li class:yours={d.os === os}>
          <div class="file">
            <span class="system">
              {d.system}
              {#if d.os === os}<span class="tag">Your system</span>{/if}
            </span>
            <span class="detail">{d.detail}</span>
            <span class="name mono">{fileName(d)}</span>
          </div>
          <a class="btn small" class:suggested={d.os === os} href={fileUrl(d)} aria-label="Download {fileName(d)}">
            Download
          </a>
        </li>
      {/each}
    </ul>
  </div>
</section>

<style>
  .download {
    padding: clamp(64px, 10vw, 140px) 0;
  }

  .grid {
    display: grid;
    grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr);
    gap: clamp(32px, 6vw, 80px);
    align-items: start;
  }

  header {
    display: grid;
    gap: 20px;
  }
  @media (min-width: 861px) {
    header {
      position: sticky;
      top: 100px;
    }
  }

  .needs {
    display: grid;
    gap: 10px;
    margin: 8px 0 0;
  }
  .needs div {
    display: grid;
    grid-template-columns: 96px 1fr;
    gap: 12px;
  }
  dt {
    font-weight: 650;
  }
  dd {
    margin: 0;
    color: var(--fg-dim);
  }

  .more {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 24px;
    font-weight: 600;
  }

  .files li {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 18px;
    transition: background 200ms var(--ease);
  }
  .files li:hover {
    background: color-mix(in srgb, var(--fill) 50%, transparent);
  }

  .file {
    display: grid;
    flex: 1;
    min-width: 0;
    gap: 1px;
  }
  .system {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    font-weight: 650;
  }
  .tag {
    padding: 1px 8px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
    background: color-mix(in oklab, var(--accent-bg) 14%, transparent);
  }
  .detail {
    font-size: 14.5px;
    color: var(--fg-dim);
  }
  .name {
    overflow: hidden;
    font-size: 12.5px;
    color: var(--fg-faint);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  @media (max-width: 860px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
