<script lang="ts">
  import Row from "../components/Row.svelte";
  import Toggle from "../components/Toggle.svelte";
  import { app, flagValue, markDirty, setFlag } from "../store.svelte";

  let { openEditor }: { openEditor: () => void } = $props();

  const RENDER_FLAGS: Record<string, string> = {
    Vulkan: "FFlagDebugGraphicsPreferVulkan",
    D3D11: "FFlagDebugGraphicsPreferD3D11",
    OpenGL: "FFlagDebugGraphicsPreferOpenGL",
  };
  const LOD = ["DFIntCSGLevelOfDetailSwitchingDistance", "DFIntCSGLevelOfDetailSwitchingDistanceL12", "DFIntCSGLevelOfDetailSwitchingDistanceL23", "DFIntCSGLevelOfDetailSwitchingDistanceL34"];
  const LOD_STATIC = "DFIntCSGLevelOfDetailSwitchingDistanceStatic";

  // Re-read derived preset state whenever the flag map changes.
  let msaa = $derived(flagValue("FIntDebugForceMSAASamples") ?? "");
  let renderMode = $derived(Object.entries(RENDER_FLAGS).find(([, f]) => flagValue(f)?.toLowerCase() === "true")?.[0] ?? "");
  let dpi = $derived(flagValue("DFFlagDisableDPIScale")?.toLowerCase() === "true");
  let frm = $derived(flagValue("DFIntDebugFRMQualityLevelOverride"));
  let mesh = $derived(flagValue(LOD_STATIC));
  let texQ = $derived(flagValue("DFFlagTextureQualityOverrideEnabled")?.toLowerCase() === "true" ? (flagValue("DFIntTextureQualityOverride") ?? "") : "");
  let graySky = $derived(flagValue("FFlagDebugSkyGray")?.toLowerCase() === "true");
  let pauseVox = $derived(flagValue("DFFlagDebugPauseVoxelizer")?.toLowerCase() === "true");
  let grass = $derived(flagValue("FIntFRMMaxGrassDistance"));

  function setRender(mode: string) {
    for (const [m, f] of Object.entries(RENDER_FLAGS)) setFlag(f, m === mode ? "True" : null);
    if (mode !== "Vulkan" && app.settings) app.settings.FakeBorderlessFullscreen = false;
  }

  function setMesh(level: number | null) {
    if (level === null) {
      setFlag(LOD_STATIC, null);
      LOD.forEach((f) => setFlag(f, null));
      return;
    }
    setFlag(LOD_STATIC, String(level));
    LOD.forEach((f, i) => setFlag(f, String((Math.min(Math.max(level - i, 0), 3) + 1) * 250)));
  }

  function setTex(v: string) {
    setFlag("DFFlagTextureQualityOverrideEnabled", v === "" ? null : "True");
    setFlag("DFIntTextureQualityOverride", v === "" ? null : v);
  }

  function bool(name: string, on: boolean) {
    setFlag(name, on ? "True" : null);
  }

  let total = $derived(Object.keys(app.flags.Flags).length);
</script>

{#if app.settings}
  <h1>FastFlags</h1>
  <p class="dim">Presets below only use flags on Roblox's allowlist. Use the editor for everything else.</p>

  <Row title="Use FastFlag manager" desc="Write ztmystrap's FastFlags to Roblox on launch. Turn off to leave ClientAppSettings.json alone.">
    <Toggle bind:checked={app.settings.UseFastFlagManager} onchange={markDirty} />
  </Row>
  <Row title="Apply non-allowlisted flags" desc="Also write flags that aren't on Roblox's allowlist. Roblox ignores most of them, and you use them at your own risk.">
    <Toggle bind:checked={app.settings.ApplyNonAllowlistedFlags} onchange={markDirty} />
  </Row>
  <div class="card row">
    <div class="label">
      <div>FastFlag editor</div>
      <div class="dim">{total} flags configured. Browse every Roblox flag, toggle, import and export.</div>
    </div>
    <button class="primary" onclick={openEditor}>Open editor</button>
  </div>

  <h2>Rendering</h2>
  <Row title="Rendering API" desc="Force the graphics API Roblox uses.">
    <select value={renderMode} onchange={(e) => setRender(e.currentTarget.value)}>
      <option value="">Automatic</option>
      <option value="D3D11">Direct3D 11</option>
      <option value="Vulkan">Vulkan</option>
      <option value="OpenGL">OpenGL</option>
    </select>
  </Row>
  <Row title="Anti-aliasing (MSAA)" desc="Force a multisample count.">
    <select value={msaa} onchange={(e) => setFlag("FIntDebugForceMSAASamples", e.currentTarget.value || null)}>
      <option value="">Automatic</option>
      <option value="1">1x</option>
      <option value="2">2x</option>
      <option value="4">4x</option>
    </select>
  </Row>
  <Row title="Texture quality" desc="Override texture quality regardless of graphics level.">
    <select value={texQ} onchange={(e) => setTex(e.currentTarget.value)}>
      <option value="">Automatic</option>
      <option value="0">Lowest</option>
      <option value="1">Low</option>
      <option value="2">Medium</option>
      <option value="3">Highest</option>
    </select>
  </Row>
  <Row title="Fix display scaling" desc="Render at native resolution on high-DPI displays.">
    <Toggle checked={dpi} onchange={(v) => bool("DFFlagDisableDPIScale", v)} />
  </Row>
  <Row title="Graphics quality override" desc="Lock the automatic quality level (1–21).">
    <div style="display: flex; gap: 0.5rem; align-items: center">
      {#if frm !== null}
        <input type="number" min="1" max="21" value={frm} style="width: 70px" oninput={(e) => setFlag("DFIntDebugFRMQualityLevelOverride", e.currentTarget.value)} />
      {/if}
      <Toggle checked={frm !== null} onchange={(v) => setFlag("DFIntDebugFRMQualityLevelOverride", v ? "21" : null)} />
    </div>
  </Row>
  <Row title="Mesh quality" desc="Level-of-detail distance for meshes. Lower is faster.">
    <select value={mesh ?? ""} onchange={(e) => setMesh(e.currentTarget.value === "" ? null : Number(e.currentTarget.value))}>
      <option value="">Automatic</option>
      <option value="0">Lowest</option>
      <option value="1">Low</option>
      <option value="2">Medium</option>
      <option value="3">High</option>
    </select>
  </Row>
  <Row title="Grass distance" desc="Maximum distance grass is drawn (0 removes grass).">
    <div style="display: flex; gap: 0.5rem; align-items: center">
      {#if grass !== null}
        <input type="number" min="0" value={grass} style="width: 80px" oninput={(e) => { setFlag("FIntFRMMaxGrassDistance", e.currentTarget.value); setFlag("FIntFRMMinGrassDistance", "0"); }} />
      {/if}
      <Toggle checked={grass !== null} onchange={(v) => { setFlag("FIntFRMMaxGrassDistance", v ? "0" : null); setFlag("FIntFRMMinGrassDistance", v ? "0" : null); }} />
    </div>
  </Row>
  <Row title="Gray sky" desc="Replace the skybox with a flat gray sky.">
    <Toggle checked={graySky} onchange={(v) => bool("FFlagDebugSkyGray", v)} />
  </Row>
  <Row title="Pause voxelizer" desc="Stop lighting voxel updates. Can raise FPS but breaks some lighting.">
    <Toggle checked={pauseVox} onchange={(v) => bool("DFFlagDebugPauseVoxelizer", v)} />
  </Row>
  <p class="dim" style="margin-top: 0.8rem">Set your FPS cap on the Global Settings page.</p>
{/if}
