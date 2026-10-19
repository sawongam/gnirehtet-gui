<script lang="ts">
  import { onMount } from "svelte";
  import { DOWNLOADS, OS_NAMES, detectOs, fileUrl, type Os } from "$lib/site";

  let os = $state<Os | null>(null);
  onMount(() => (os = detectOs()));

  // On Linux the AppImage runs everywhere; the list below has the packages.
  const primary = $derived(
    os === "linux" ? DOWNLOADS.find((d) => d.suffix.endsWith("AppImage")) : DOWNLOADS.find((d) => d.os === os),
  );
</script>

<div class="cta">
  {#if os && primary && os !== "mac"}
    <a class="btn suggested big" href={fileUrl(primary)}>Download for {OS_NAMES[os]}</a>
    <a class="btn flat big" href="#download">Other systems</a>
  {:else if os === "mac"}
    <a class="btn suggested big" href="#download">Download for macOS</a>
    <a class="btn flat big" href="#download">Other systems</a>
  {:else}
    <a class="btn suggested big" href="#download">Download</a>
    <a class="btn flat big" href="#how">How it works</a>
  {/if}
</div>

<style>
  .cta {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
  }
  .big {
    height: 50px;
    padding: 0 26px;
    font-size: 16px;
  }
</style>
