<script lang="ts">
  import "../app.css";
  import { applyTheme, settings } from "$lib/settings.svelte";

  let { children } = $props();

  applyTheme(settings.theme);
</script>

{@render children()}
