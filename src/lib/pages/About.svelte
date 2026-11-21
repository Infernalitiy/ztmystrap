<script lang="ts">
  import Icon from "../components/Icon.svelte";
  import { api } from "../api";
  import { app } from "../store.svelte";
</script>

<h1>About ztmystrap</h1>
<p>An open-source Roblox bootstrapper written in Rust, focused on performance. Version <span class="mono">{app.info?.version}</span>.</p>

<h2>Links</h2>
<div class="links">
  <button class="card linked" onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap")}>
    <img class="ic-ink" src="/icons/github.svg" alt="" width="22" height="22" />
    <span><b>Source code</b><span class="dim">Infernalitiy/ztmystrap on GitHub</span></span>
  </button>
  <button class="card linked" onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap/issues/new")}>
    <Icon name="triangle-alert" size={22} />
    <span><b>Report a problem</b><span class="dim">Open a GitHub issue</span></span>
  </button>
  <button class="card linked" onclick={() => api.openUrl("https://github.com/Infernalitiy/ztmystrap/releases")}>
    <Icon name="download" size={22} />
    <span><b>Releases</b><span class="dim">Download the latest build</span></span>
  </button>
</div>

<h2>Credits</h2>
<div class="card">
  <p>ztmystrap reimplements features from <b>Fishstrap</b>, which is a fork of <b>Bloxstrap</b>. Both are MIT-licensed.</p>
  <p style="margin-top: 0.6rem; display: flex; gap: 0.5rem">
    <button class="small" onclick={() => api.openUrl("https://github.com/fishstrap/fishstrap")}><img class="ic-ink" src="/icons/github.svg" alt="" />fishstrap/fishstrap</button>
    <button class="small" onclick={() => api.openUrl("https://github.com/bloxstraplabs/bloxstrap")}><img class="ic-ink" src="/icons/github.svg" alt="" />bloxstraplabs/bloxstrap</button>
  </p>
  <p class="dim" style="margin-top: 0.5rem">The FastFlag list comes from MaximumADHD's Roblox-Client-Tracker. ztmystrap is not affiliated with Roblox Corporation.</p>
</div>

<h2>License</h2>
<div class="card dim">MIT License. See LICENSE in the repository.</div>

<style>
  .links {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.5rem;
  }
  .links .card {
    margin: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.8rem;
    text-align: left;
    padding: 1rem 1.1rem;
    background: var(--bg-elev);
    border-color: var(--border);
  }
  .links .card:hover {
    border-color: var(--border-strong);
  }
  .links .card:active {
    transform: translateY(0) scale(0.985);
  }
  .links .card span {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
  }
  .links .card b {
    font-weight: 500;
  }
  .links .card .dim {
    font-size: 0.82rem;
  }
  .links :global(svg) {
    width: 22px;
    height: 22px;
    color: var(--text-dim);
  }
  .links img {
    width: 22px;
    height: 22px;
  }
</style>
