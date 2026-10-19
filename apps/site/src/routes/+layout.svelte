<script lang="ts">
  import "@fontsource-variable/bricolage-grotesque";
  import "../app.css";
  import { onMount } from "svelte";
  import { release } from "$lib/release.svelte";

  let { children } = $props();

  onMount(() => {
    void release.load();
  });
</script>

{@render children()}
