<script lang="ts">
  import "../app.css";
  import Bootstrap from "$lib/Bootstrap.svelte";
  import SettingsApp from "$lib/SettingsApp.svelte";

  const mode = (globalThis as any).__ZTMY_MODE__ ?? "settings";
</script>

{#if mode === "bootstrap"}
  <Bootstrap />
{:else}
  <SettingsApp />
{/if}
