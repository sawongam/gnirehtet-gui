<script lang="ts">
  import "@fontsource-variable/bricolage-grotesque";
  import "../app.css";
  import { onMount } from "svelte";
  import { theme } from "$lib/theme.svelte";
  import { release } from "$lib/release.svelte";

  let { children } = $props();

  onMount(() => {
    theme.watchSystem();
    void release.load();
  });
</script>

{@render children()}
