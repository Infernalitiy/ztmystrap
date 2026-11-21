<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "./api";
  import Icon from "./components/Icon.svelte";
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
    { id: "welcome", label: "Home", icon: "house" },
    { id: "behaviour", label: "Behaviour", icon: "sliders-horizontal" },
    { id: "deployment", label: "Deployment", icon: "package" },
    { id: "fastflags", label: "FastFlags", icon: "flag" },
    { id: "gbs", label: "Global Settings", icon: "gauge" },
    { id: "about", label: "About", icon: "info" },
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
  <nav aria-label="Pages">
    <div class="brand"><Logo size={26} /><span>ztmystrap</span></div>
    {#each PAGES as p}
      {@const active = page === p.id || (p.id === "fastflags" && page === "editor")}
      <button class="nav" class:active aria-current={active ? "page" : undefined} onclick={() => go(p.id)}>
        <Icon name={p.icon} />{p.label}
      </button>
    {/each}
    <div class="grow"></div>
    <div class="ver mono">v{app.info?.version ?? ""}</div>
  </nav>

  <main>
    <div class="content">
      <div class="page">
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
    </div>

    <footer>
      <span class="status" class:dirty={!app.toast && app.dirty}>{app.toast || (app.dirty ? "Unsaved changes" : "All changes saved")}</span>
      <div class="grow"></div>
      <button disabled={saving || !app.info?.installed} onclick={() => save(true)}><Icon name="rocket" />Save and launch Roblox</button>
      <button class="primary" disabled={saving || !app.loaded} onclick={() => save()}>Save</button>
    </footer>
  </main>
</div>

<style>
  .shell {
    display: flex;
    height: 100vh;
  }
  .grow {
    flex: 1;
  }
  nav {
    width: 216px;
    flex: none;
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 1.1rem 0.7rem 0.9rem;
    gap: 2px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 1.1rem;
    letter-spacing: -0.03em;
    padding: 0 0.5rem 1.3rem;
  }
  /* nav items rest bare and take the surface on hover; they never step */
  .nav,
  .nav:hover,
  .nav:active {
    justify-content: flex-start;
    gap: 0.65rem;
    border: none;
    background: transparent;
    padding: 0.5rem 0.65rem;
    color: var(--text-dim);
    position: relative;
    transform: none;
  }
  .nav:hover:not(.active) {
    background: var(--bg-elev);
    color: var(--text);
  }
  .nav.active {
    background: var(--bg-elev-2);
    color: var(--text);
  }
  .nav.active::before {
    content: "";
    position: absolute;
    left: -0.7rem;
    top: 8px;
    bottom: 8px;
    width: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .ver {
    padding: 0 0.65rem;
    font-size: 0.75rem;
    color: var(--text-dim);
  }
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    background: var(--bg);
  }
  .content {
    flex: 1;
    overflow-y: auto;
    padding: 2.2rem 2.4rem 2.4rem;
  }
  .page {
    max-width: 780px;
  }
  footer {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.7rem 1.2rem 0.7rem 2.4rem;
    border-top: 1px solid var(--border);
  }
  .status {
    font-size: 0.85rem;
    color: var(--text-dim);
  }
  .status.dirty {
    color: var(--text);
  }
  .status.dirty::before {
    content: "";
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent);
    margin-right: 0.5rem;
    vertical-align: 1px;
  }
</style>
