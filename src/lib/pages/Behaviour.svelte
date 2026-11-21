<script lang="ts">
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app, markDirty } from "../store.svelte";
</script>

{#if app.settings}
  <h1>Behaviour</h1>
  <Row title="Confirm launches" desc="Ask before launching when Roblox is already open (launching again closes it).">
    <Toggle bind:checked={app.settings.ConfirmLaunches} onchange={markDirty} />
  </Row>
  <Row title="Force Roblox language" desc="When launching from the website, use the game's language instead of the Roblox app language.">
    <Toggle bind:checked={app.settings.ForceRobloxLanguage} onchange={markDirty} />
  </Row>
  <p class="dim" style="margin-top: 1rem">Better matchmaking, background updates and the cache cleaner arrive in later versions.</p>
{/if}
