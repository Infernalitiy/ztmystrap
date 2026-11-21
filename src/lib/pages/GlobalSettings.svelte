<script lang="ts">
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app, markDirty } from "../store.svelte";

  const FPS_PRESETS = [30, 60, 75, 120, 144, 165, 240, 360];
</script>

<h1>Global Settings</h1>
<p class="dim">These edit Roblox's own in-game settings file (GlobalBasicSettings_13.xml).</p>

{#if app.gbsError}
  <div class="banner warn">Couldn't read Roblox's settings file: {app.gbsError}</div>
{:else if app.gbs}
  <Row title="FPS cap" desc="Roblox's built-in framerate limit. 0 means uncapped (sets -1).">
    <div style="display: flex; gap: 0.4rem; align-items: center">
      <select value={FPS_PRESETS.includes(app.gbs.framerateCap) ? app.gbs.framerateCap : ""} onchange={(e) => { if (e.currentTarget.value && app.gbs) { app.gbs.framerateCap = Number(e.currentTarget.value); markDirty(); } }}>
        <option value="">Custom</option>
        {#each FPS_PRESETS as f}<option value={f}>{f}</option>{/each}
      </select>
      <input type="number" bind:value={app.gbs.framerateCap} oninput={markDirty} style="width: 80px" />
    </div>
  </Row>
  <Row title="Graphics quality" desc="0 is automatic, 1–10 is manual.">
    <div style="display: flex; gap: 0.6rem; align-items: center">
      <input type="range" min="0" max="10" bind:value={app.gbs.graphicsQuality} oninput={markDirty} />
      <span class="mono" style="width: 2.5rem">{app.gbs.graphicsQuality === 0 ? "Auto" : app.gbs.graphicsQuality}</span>
    </div>
  </Row>
  <Row title="Mouse sensitivity">
    <input type="number" step="0.01" min="0" max="10" bind:value={app.gbs.mouseSensitivity} oninput={markDirty} style="width: 90px" />
  </Row>
  <Row title="UI transparency" desc="Background transparency of menus (0–1).">
    <input type="number" step="0.05" min="0" max="1" bind:value={app.gbs.transparency} oninput={markDirty} style="width: 90px" />
  </Row>
  <Row title="Text size">
    <select bind:value={app.gbs.fontSize} onchange={markDirty}>
      {#each [1, 2, 3, 4] as s}<option value={s}>{s}x</option>{/each}
    </select>
  </Row>
  <Row title="Reduced motion">
    <Toggle bind:checked={app.gbs.reducedMotion} onchange={markDirty} />
  </Row>
  <Row title="VR enabled">
    <Toggle bind:checked={app.gbs.vrEnabled} onchange={markDirty} />
  </Row>
  <Row title="Lock file (read-only)" desc="Stop Roblox from overwriting these values from the in-game menu.">
    <Toggle bind:checked={app.gbs.readOnly} onchange={markDirty} />
  </Row>
{:else}
  <p class="dim">Loading...</p>
{/if}
