<script lang="ts">
  import { api } from "../api";
  import { app } from "../store.svelte";
</script>

<h1>About ztmystrap</h1>
<p>Version <span class="mono">{app.info?.version}</span></p>
<p class="dim">An open-source Roblox bootstrapper written in Rust, focused on performance.</p>

<h2>Links</h2>
<div class="card" style="display: flex; gap: 0.5rem; flex-wrap: wrap">
  <button onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap")}>GitHub</button>
  <button onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap/issues/new")}>Report a problem</button>
  <button onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap/releases")}>Releases</button>
</div>

<h2>Credits</h2>
<div class="card">
  <p>ztmystrap reimplements features from <b>Fishstrap</b>, which is a fork of <b>Bloxstrap</b>. Both are MIT-licensed.</p>
  <p style="margin-top: 0.5rem; display: flex; gap: 0.5rem">
    <button class="small" onclick={() => api.openUrl("https://github.com/fishstrap/fishstrap")}>fishstrap/fishstrap</button>
    <button class="small" onclick={() => api.openUrl("https://github.com/bloxstraplabs/bloxstrap")}>bloxstraplabs/bloxstrap</button>
  </p>
  <p class="dim" style="margin-top: 0.5rem">The FastFlag list comes from MaximumADHD's Roblox-Client-Tracker. ztmystrap is not affiliated with Roblox Corporation.</p>
</div>

<h2>License</h2>
<div class="card dim">MIT License. See LICENSE in the repository.</div>
