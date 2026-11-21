<script lang="ts">
  import { api } from "../api";
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app, markDirty } from "../store.svelte";

  let info = $state<{ channel: string; version: string; guid: string } | null>(null);
  let infoError = $state("");
  let checking = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const DOMAIN_RE = /^([a-zA-Z0-9.-]+)\.([a-zA-Z0-9]+)$/;

  async function check() {
    if (!app.settings) return;
    checking = true;
    infoError = "";
    try {
      info = await api.channelInfo(app.settings.Channel, app.settings.RobloxDomain);
    } catch (e) {
      info = null;
      infoError = String(e);
    }
    checking = false;
  }

  function onChannelInput() {
    markDirty();
    clearTimeout(timer);
    timer = setTimeout(check, 600);
  }

  $effect(() => {
    if (app.settings && !info && !checking && !infoError) check();
  });
</script>

{#if app.settings && app.state}
  <h1>Deployment</h1>

  <Row title="Channel" desc="The Roblox release channel to install. Leave as production unless you know you need another.">
    <input type="text" bind:value={app.settings.Channel} oninput={onChannelInput} style="width: 200px" />
  </Row>
  <div class="card dim" style="font-size: 0.85rem">
    {#if checking}Checking channel...
    {:else if info}Version <span class="mono">{info.version}</span> · <span class="mono">{info.guid}</span>
    {:else if infoError}<span style="color: var(--danger)">{infoError}</span>{/if}
  </div>

  <Row title="Channel change mode" desc="What to do when a Roblox link asks for a different channel.">
    <select bind:value={app.settings.ChannelChangeMode} onchange={markDirty}>
      <option value={0}>Change automatically</option>
      <option value={1}>Ask me</option>
      <option value={2}>Ignore</option>
    </select>
  </Row>

  <Row title="Roblox domain" desc="Only change this for alternative Roblox domains.">
    <input
      type="text"
      bind:value={app.settings.RobloxDomain}
      oninput={markDirty}
      style="width: 200px; {DOMAIN_RE.test(app.settings.RobloxDomain) ? '' : 'border-color: var(--danger)'}"
    />
  </Row>

  <Row title="Static version folder" desc="Install Roblox to Versions\WindowsPlayer instead of a new folder per version (useful for firewall rules).">
    <Toggle bind:checked={app.settings.StaticDirectory} onchange={markDirty} />
  </Row>

  <Row title="Force reinstall" desc="Redownload Roblox on the next launch.">
    <Toggle bind:checked={app.state.ForceReinstall} onchange={markDirty} />
  </Row>

  <Row title="Check for ztmystrap updates" desc="Self-updating ships in v0.4. This setting is saved now.">
    <Toggle bind:checked={app.settings.CheckForUpdates} onchange={markDirty} />
  </Row>

  <Row title="Keep old versions" desc="Don't delete old Roblox versions and cached packages after an update.">
    <Toggle bind:checked={app.settings.DebugDisableVersionPackageCleanup} onchange={markDirty} />
  </Row>

  <h2>Folders</h2>
  <div class="card" style="display: flex; gap: 0.5rem; flex-wrap: wrap">
    <button onclick={() => api.openFolder("versions")}>Roblox versions</button>
    <button onclick={() => api.openFolder("logs")}>Logs</button>
    <button onclick={() => api.openFolder("base")}>ztmystrap folder</button>
    <button onclick={() => api.openFolder("roblox")}>Roblox data</button>
  </div>
{/if}
