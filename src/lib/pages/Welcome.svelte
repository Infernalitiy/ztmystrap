<script lang="ts">
  import { api } from "../api";
  import { app, loadAll, toast } from "../store.svelte";

  let busy = $state(false);
  let error = $state("");
  let importDone = $state("");

  async function install() {
    busy = true;
    error = "";
    try {
      await api.install();
      await loadAll();
      toast("ztmystrap installed");
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }

  async function doImport() {
    busy = true;
    error = "";
    try {
      const r = await api.importFishstrap();
      importDone = `Imported ${r.settings ? "settings, " : ""}${r.flags} FastFlags and ${r.modFiles} mod files.`;
      await loadAll();
    } catch (e) {
      error = String(e);
    }
    busy = false;
  }
</script>

<h1>Welcome to ztmystrap</h1>
<p class="dim">A fast Roblox bootstrapper written in Rust. Built on the work of Bloxstrap and Fishstrap.</p>

{#if !app.info?.installed}
  <h2>Install</h2>
  <div class="card">
    <p>ztmystrap will install to <span class="mono">{app.info?.baseDir}</span>, add Start menu and desktop shortcuts, and handle Roblox links (<span class="mono">roblox-player:</span>). Fishstrap's files are left alone.</p>
    <p style="margin-top: 0.8rem"><button class="primary" disabled={busy} onclick={install}>Install ztmystrap</button></p>
  </div>
{:else}
  <div class="banner info">ztmystrap is installed{app.info?.handlerIsOurs ? " and handles Roblox links." : ", but another app currently handles Roblox links. Launch Roblox once through ztmystrap to take them back."}</div>
{/if}

{#if app.info?.fishstrapPresent}
  <h2>Import from Fishstrap</h2>
  <div class="card">
    <p>Copy your Fishstrap settings, FastFlags and mods into ztmystrap. Fishstrap itself is not changed.</p>
    {#if importDone}<p class="dim">{importDone}</p>{/if}
    {#if app.info?.importedFromFishstrap && !importDone}<p class="dim">You've imported before. Importing again merges on top of your current setup.</p>{/if}
    <p style="margin-top: 0.8rem"><button disabled={busy} onclick={doImport}>Import from Fishstrap</button></p>
  </div>
{/if}

{#if error}<div class="banner warn">{error}</div>{/if}
