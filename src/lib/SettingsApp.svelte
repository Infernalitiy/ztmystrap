<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import Logo from "./Logo.svelte";
  import About from "./pages/About.svelte";
  import Behaviour from "./pages/Behaviour.svelte";
  import Deployment from "./pages/Deployment.svelte";
  import FastFlags from "./pages/FastFlags.svelte";
  import FlagEditor from "./pages/FlagEditor.svelte";
  import GlobalSettings from "./pages/GlobalSettings.svelte";
  import Welcome from "./pages/Welcome.svelte";
  import { app, loadAll, saveAll, toast } from "./store.svelte";

  const PAGES = [
    { id: "welcome", label: "Home" },
    { id: "behaviour", label: "Behaviour" },
    { id: "deployment", label: "Deployment" },
    { id: "fastflags", label: "FastFlags" },
    { id: "gbs", label: "Global Settings" },
    { id: "about", label: "About" },
  ] as const;
  type PageId = (typeof PAGES)[number]["id"] | "editor";

  let page = $state<PageId>("welcome");
  let loadError = $state("");
  let saving = $state(false);

  onMount(async () => {
    try {
      await loadAll();
      const last = app.state?.LastPage as PageId | null;
      if (app.info?.installed && last && PAGES.some((p) => p.id === last)) page = last;
    } catch (e) {
      loadError = String(e);
    }
  });

  function go(p: PageId) {
    page = p;
    if (app.state && p !== "editor") app.state.LastPage = p;
  }

  async function save(launch = false) {
    saving = true;
    try {
      await saveAll();
      if (launch) {
        await api.launchRoblox();
        toast("Launching Roblox...");
      }
    } catch (e) {
      toast(`Save failed: ${e}`);
    }
    saving = false;
  }
</script>

<div class="shell">
  <nav>
    <div class="brand"><Logo size={28} /><span>ztmystrap</span></div>
    {#each PAGES as p}
      <button class="nav" class:active={page === p.id || (p.id === "fastflags" && page === "editor")} onclick={() => go(p.id)}>{p.label}</button>
    {/each}
    <div style="flex: 1"></div>
    <div class="dim ver">v{app.info?.version ?? ""}</div>
  </nav>

  <main>
    <div class="content">
      {#if loadError}
        <div class="banner warn">Failed to load settings: {loadError}</div>
      {:else if !app.loaded}
        <p class="dim">Loading...</p>
      {:else if page === "welcome"}<Welcome />
      {:else if page === "behaviour"}<Behaviour />
      {:else if page === "deployment"}<Deployment />
      {:else if page === "fastflags"}<FastFlags openEditor={() => go("editor")} />
      {:else if page === "editor"}<FlagEditor back={() => go("fastflags")} />
      {:else if page === "gbs"}<GlobalSettings />
      {:else if page === "about"}<About />{/if}
    </div>

    <footer>
      <span class="dim">{app.toast || (app.dirty ? "You have unsaved changes" : "")}</span>
      <div style="flex: 1"></div>
      <button disabled={saving || !app.info?.installed} onclick={() => save(true)}>Save and launch Roblox</button>
      <button class="primary" disabled={saving || !app.loaded} onclick={() => save()}>Save</button>
    </footer>
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
  }
  nav {
    width: 210px;
    background: var(--bg-elev);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1rem 0.6rem;
    gap: 0.15rem;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-weight: 700;
    font-size: 1.1rem;
    padding: 0 0.4rem 1rem;
  }
  .nav {
    text-align: left;
    border: none;
    background: transparent;
    padding: 0.55rem 0.8rem;
  }
  .nav:hover {
    background: var(--bg-elev-2);
  }
  .nav.active {
    background: var(--accent-soft);
    color: white;
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .ver {
    padding: 0 0.8rem;
    font-size: 0.8rem;
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 2rem 2rem;
  }
  footer {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    padding: 0.7rem 1.2rem;
    border-top: 1px solid var(--border);
    background: var(--bg-elev);
  }
</style>
