<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { api } from "./api";
  import Logo from "./Logo.svelte";

  let status = $state("Starting...");
  let value = $state<number | null>(null);
  let error = $state("");

  onMount(() => {
    const un1 = listen<{ status: string; value: number | null }>("progress", (e) => {
      status = e.payload.status;
      value = e.payload.value;
    });
    const un2 = listen<string>("bootstrap-error", (e) => (error = e.payload));
    return () => {
      un1.then((f) => f());
      un2.then((f) => f());
    };
  });

  const close = () => getCurrentWindow().close();
</script>

<div class="wrap" data-tauri-drag-region>
  <div class="top" data-tauri-drag-region>
    <Logo size={56} />
    <div data-tauri-drag-region>
      <div class="title">ztmystrap</div>
      {#if error}
        <div class="err">Roblox couldn't be launched.</div>
      {:else}
        <div class="dim">{status}</div>
      {/if}
    </div>
  </div>

  {#if error}
    <pre class="msg">{error}</pre>
    <div class="actions">
      <button onclick={() => api.openFolder("logs")}>Open logs</button>
      <button class="primary" onclick={close}>Close</button>
    </div>
  {:else}
    <div class="bar" class:indeterminate={value === null}>
      <div class="fill" style:width={value === null ? "35%" : `${Math.round(value * 100)}%`}></div>
    </div>
    <div class="actions">
      <span class="dim">{value === null ? "" : `${Math.round(value * 100)}%`}</span>
      <div style="flex: 1"></div>
      <button onclick={close}>Cancel</button>
    </div>
  {/if}
</div>

<style>
  .wrap {
    height: 100vh;
    padding: 1.6rem 1.8rem;
    display: flex;
    flex-direction: column;
    gap: 1.2rem;
    background: radial-gradient(circle at 15% 0%, rgba(124, 92, 255, 0.25), transparent 60%), var(--bg);
    border: 1px solid var(--border);
  }
  .top {
    display: flex;
    gap: 1rem;
    align-items: center;
  }
  .title {
    font-size: 1.4rem;
    font-weight: 700;
  }
  .err {
    color: var(--danger);
  }
  .bar {
    height: 8px;
    border-radius: 999px;
    background: var(--bg-elev-2);
    overflow: hidden;
    margin-top: auto;
  }
  .fill {
    height: 100%;
    background: linear-gradient(90deg, #9b82ff, var(--accent));
    border-radius: 999px;
    transition: width 0.2s;
  }
  .indeterminate .fill {
    animation: slide 1.2s ease-in-out infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(300%);
    }
  }
  .actions {
    display: flex;
    gap: 0.6rem;
    align-items: center;
  }
  .msg {
    flex: 1;
    overflow: auto;
    margin: 0;
    font-size: 0.8rem;
    white-space: pre-wrap;
    color: var(--text-dim);
    user-select: text;
  }
</style>
