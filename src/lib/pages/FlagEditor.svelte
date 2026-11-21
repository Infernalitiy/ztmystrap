<script lang="ts">
  import { api, FLAG_NAME_RE } from "../api";
  import Toggle from "../components/Toggle.svelte";
  import { allowlisted, app, markDirty, toast } from "../store.svelte";

  let { back }: { back: () => void } = $props();

  let search = $state("");
  let newName = $state("");
  let newValue = $state("");
  let importText = $state("");
  let showImport = $state(false);
  let catalog = $state<string[] | null>(null);
  let catalogError = $state("");
  let catalogLoading = $state(false);
  let refreshing = $state(false);

  const q = $derived(search.trim().toLowerCase());
  const entries = $derived(Object.entries(app.flags.Flags).sort(([a], [b]) => a.localeCompare(b)));
  const safe = $derived(entries.filter(([n]) => allowlisted(n) && n.toLowerCase().includes(q)));
  const unsafe = $derived(entries.filter(([n]) => !allowlisted(n) && n.toLowerCase().includes(q)));
  const unusedAllowlist = $derived(app.allowlist.flags.filter((f) => !(f.name in app.flags.Flags) && f.name.toLowerCase().includes(q)));
  const suggestions = $derived(
    catalog && newName.length >= 3
      ? catalog.filter((n) => n.toLowerCase().includes(newName.toLowerCase()) && !(n in app.flags.Flags)).slice(0, 40)
      : [],
  );
  const nameValid = $derived(FLAG_NAME_RE.test(newName.trim()));

  async function loadCatalog() {
    catalogLoading = true;
    catalogError = "";
    try {
      catalog = await api.flagCatalog();
    } catch (e) {
      catalogError = String(e);
    }
    catalogLoading = false;
  }
  $effect(() => {
    if (catalog === null && !catalogLoading && !catalogError) loadCatalog();
  });

  function defaultValue(name: string) {
    if (/^(D|S)?FFlag/.test(name)) return "True";
    if (/^(D|S)?FInt/.test(name)) return "0";
    return "";
  }

  function add(name = newName.trim(), value = newValue) {
    if (!FLAG_NAME_RE.test(name)) return;
    app.flags.Flags[name] = { Value: value || defaultValue(name), Enabled: true };
    newName = "";
    newValue = "";
    markDirty();
  }

  function remove(name: string) {
    delete app.flags.Flags[name];
    markDirty();
  }

  function rename(oldName: string, next: string) {
    next = next.trim();
    if (next === oldName || !FLAG_NAME_RE.test(next)) return;
    app.flags.Flags[next] = app.flags.Flags[oldName];
    delete app.flags.Flags[oldName];
    markDirty();
  }

  async function doImport() {
    try {
      const parsed = await api.parseFlagImport(importText);
      const keys = Object.keys(parsed);
      if (keys.length > 16 && !confirm(`Import ${keys.length} flags?`)) return;
      const conflicts = keys.filter((k) => k in app.flags.Flags && app.flags.Flags[k].Value !== parsed[k]);
      let overwrite = true;
      if (conflicts.length) {
        overwrite = confirm(`${conflicts.length} flags already exist with different values:\n${conflicts.slice(0, 25).join("\n")}${conflicts.length > 25 ? "\n..." : ""}\n\nOverwrite them?`);
      }
      let n = 0;
      for (const k of keys) {
        if (!overwrite && k in app.flags.Flags) continue;
        app.flags.Flags[k] = { Value: parsed[k], Enabled: true };
        n++;
      }
      markDirty();
      toast(`Imported ${n} flags`);
      importText = "";
      showImport = false;
    } catch (e) {
      alert(`That isn't valid FastFlag JSON:\n${e}`);
    }
  }

  async function exportFlags() {
    const out: Record<string, string> = {};
    for (const [n, e] of entries) if (e.Enabled) out[n] = e.Value;
    await navigator.clipboard.writeText(JSON.stringify(out, null, 2));
    toast(`Copied ${Object.keys(out).length} enabled flags to the clipboard`);
  }

  async function refresh() {
    refreshing = true;
    try {
      app.allowlist = await api.refreshAllowlist();
      toast(`Allowlist updated: ${app.allowlist.flags.length} flags`);
    } catch (e) {
      toast(`Couldn't refresh the allowlist: ${e}`);
    }
    refreshing = false;
  }

  function setAll(list: [string, unknown][], on: boolean) {
    for (const [n] of list) app.flags.Flags[n].Enabled = on;
    markDirty();
  }
</script>

<div class="head">
  <button onclick={back}>← Back</button>
  <h1 style="margin: 0">FastFlag editor</h1>
  <div style="flex: 1"></div>
  <button onclick={() => (showImport = !showImport)}>Import JSON</button>
  <button onclick={exportFlags}>Export</button>
</div>

{#if showImport}
  <div class="card">
    <p class="dim">Paste a ClientAppSettings.json or any list of flags. Comments, trailing commas and missing braces are fine.</p>
    <textarea rows="8" bind:value={importText} placeholder={'{\n  "FIntDebugForceMSAASamples": "4"\n}'}></textarea>
    <p style="margin-top: 0.5rem"><button class="primary" disabled={!importText.trim()} onclick={doImport}>Import</button></p>
  </div>
{/if}

<div class="card">
  <div style="display: flex; gap: 0.5rem; align-items: center">
    <input type="text" bind:value={newName} placeholder={catalog ? `Add a flag: search ${catalog.length.toLocaleString()} Roblox flags` : "Flag name"} style="flex: 1" onkeydown={(e) => e.key === "Enter" && add()} />
    <input type="text" bind:value={newValue} placeholder="Value" style="width: 140px" onkeydown={(e) => e.key === "Enter" && add()} />
    <button class="primary" disabled={!nameValid} onclick={() => add()}>Add</button>
  </div>
  {#if catalogLoading}<p class="dim" style="margin-top: 0.4rem">Loading the Roblox flag list...</p>{/if}
  {#if catalogError}<p class="dim" style="margin-top: 0.4rem">Couldn't load the Roblox flag list ({catalogError}). You can still type names manually.</p>{/if}
  {#if suggestions.length}
    <div class="suggest">
      {#each suggestions as s (s)}
        <button class="small" onclick={() => (newName = s)}>
          {s}{#if allowlisted(s)}<span class="tag safe">allowlisted</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<input type="text" bind:value={search} placeholder="Filter configured flags" style="width: 100%; margin: 0.4rem 0 0.2rem" />

<section>
  <div class="sec-head">
    <h2>Allowlisted <span class="count">{safe.length}</span></h2>
    <span class="dim" style="font-size: 0.85rem">Roblox applies these. {app.allowlist.flags.length} flags on the list, from {app.allowlist.retrieved}.</span>
    <div style="flex: 1"></div>
    <button class="small" disabled={refreshing} onclick={refresh}>Refresh list</button>
    <button class="small" onclick={() => setAll(safe, true)}>All on</button>
    <button class="small" onclick={() => setAll(safe, false)}>All off</button>
  </div>
  {#each safe as [name, e] (name)}
    {@render flagRow(name, e)}
  {/each}
  {#if unusedAllowlist.length}
    <details class="card">
      <summary>Add an allowlisted flag ({unusedAllowlist.length} unused)</summary>
      {#each unusedAllowlist as f (f.name)}
        <div class="row" style="padding: 0.3rem 0">
          <span class="mono">{f.name}</span><span class="dim">{f.category}</span>
          <button class="small" onclick={() => add(f.name, "")}>Add</button>
        </div>
      {/each}
    </details>
  {/if}
</section>

<section>
  <div class="sec-head">
    <h2>Non-allowlisted <span class="count">{unsafe.length}</span></h2>
    <div style="flex: 1"></div>
    <button class="small" onclick={() => setAll(unsafe, true)}>All on</button>
    <button class="small" onclick={() => setAll(unsafe, false)}>All off</button>
  </div>
  <div class="banner warn">
    Roblox ignores flags that aren't on its allowlist, so most of these do nothing. They're only written when <b>Apply non-allowlisted flags</b> is on
    ({app.settings?.ApplyNonAllowlistedFlags ? "currently on" : "currently off"}). Use them at your own risk.
  </div>
  {#each unsafe as [name, e] (name)}
    {@render flagRow(name, e)}
  {/each}
  {#if !unsafe.length}<p class="dim">No non-allowlisted flags configured.</p>{/if}
</section>

{#snippet flagRow(name: string, e: { Value: string; Enabled: boolean })}
  <div class="flag" class:off={!e.Enabled}>
    <Toggle bind:checked={e.Enabled} onchange={markDirty} />
    <input class="mono name" type="text" value={name} onchange={(ev) => rename(name, ev.currentTarget.value)} />
    <input class="mono" type="text" bind:value={e.Value} oninput={markDirty} style="width: 180px" />
    <button class="small danger" onclick={() => remove(name)} title="Delete">✕</button>
  </div>
{/snippet}

<style>
  .head {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    margin-bottom: 0.8rem;
  }
  .sec-head {
    display: flex;
    gap: 0.6rem;
    align-items: baseline;
    margin: 1.2rem 0 0.5rem;
  }
  .sec-head h2 {
    margin: 0;
  }
  .count {
    color: var(--text-dim);
    font-weight: 400;
  }
  .flag {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    padding: 0.35rem 0.6rem;
    border-bottom: 1px solid var(--border);
  }
  .flag.off .name {
    opacity: 0.5;
  }
  .flag .name {
    flex: 1;
    min-width: 0;
    background: transparent;
    border-color: transparent;
  }
  .flag .name:focus {
    border-color: var(--accent);
  }
  .suggest {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-top: 0.5rem;
    max-height: 160px;
    overflow: auto;
  }
  .suggest button {
    font-family: "Cascadia Code", Consolas, monospace;
  }
  .tag {
    margin-left: 0.4rem;
    font-size: 0.7rem;
    padding: 0 0.3rem;
    border-radius: 4px;
  }
  .tag.safe {
    background: rgba(52, 211, 153, 0.15);
    color: var(--ok);
  }
  details summary {
    cursor: pointer;
  }
</style>
