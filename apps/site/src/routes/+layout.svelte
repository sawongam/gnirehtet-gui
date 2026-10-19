<script lang="ts">
  import "@fontsource-variable/bricolage-grotesque";
  import "../app.css";
  import { onMount } from "svelte";
  import { theme } from "$lib/theme.svelte";

  let { children } = $props();

  onMount(() => theme.watchSystem());
</script>

{@render children()}
