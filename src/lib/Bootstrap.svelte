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
    <Logo size={44} />
    <div class="name" data-tauri-drag-region>ztmystrap</div>
    <div class="grow" data-tauri-drag-region></div>
    <span class="pct mono" data-tauri-drag-region>{!error && value !== null ? `${Math.round(value * 100)}%` : ""}</span>
  </div>

  {#if error}
    <div class="err">Roblox couldn't be launched</div>
    <pre class="msg">{error}</pre>
    <div class="actions">
      <div class="grow"></div>
      <button onclick={() => api.openFolder("logs")}>Open logs</button>
      <button class="primary" onclick={close}>Close</button>
    </div>
  {:else}
    <div class="status" data-tauri-drag-region>{status}</div>
    <div class="bar" class:indeterminate={value === null} role="progressbar" aria-label={status} aria-valuemin={0} aria-valuemax={100} aria-valuenow={value === null ? undefined : Math.round(value * 100)}>
      <div class="fill" style:width={value === null ? "30%" : `${Math.round(value * 100)}%`}></div>
    </div>
    <div class="actions">
      <span class="dim small" data-tauri-drag-region>Roblox bootstrapper</span>
      <div class="grow"></div>
      <button onclick={close}>Cancel</button>
    </div>
  {/if}
</div>

<style>
  .wrap {
    height: 100vh;
    padding: 1.4rem 1.6rem 1.3rem;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border: 1px solid var(--border-strong);
  }
  .grow {
    flex: 1;
  }
  .top {
    display: flex;
    gap: 0.8rem;
    align-items: center;
  }
  .name {
    font-family: var(--font-display);
    font-weight: 600;
    font-size: 1.2rem;
    letter-spacing: -0.03em;
  }
  .pct {
    color: var(--text-dim);
    font-size: 0.85rem;
  }
  .status {
    margin-top: auto;
    font-family: var(--font-display);
    font-weight: var(--display-weight);
    letter-spacing: -0.025em;
    font-size: 1.45rem;
    line-height: 1.15;
    margin-bottom: 0.9rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .err {
    margin-top: 1.1rem;
    font-family: var(--font-display);
    font-size: 1.2rem;
    letter-spacing: -0.02em;
    color: var(--danger);
  }
  .bar {
    height: 4px;
    border-radius: 999px;
    background: var(--bg-elev-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
    transition: width 200ms var(--ease);
  }
  .indeterminate .fill {
    animation: slide 1.4s cubic-bezier(0.65, 0, 0.35, 1) infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    margin-top: 1rem;
  }
  .small {
    font-size: 0.8rem;
  }
  .msg {
    flex: 1;
    overflow: auto;
    margin: 0.6rem 0 0;
    font-family: var(--font-mono);
    font-size: 0.75rem;
    white-space: pre-wrap;
    color: var(--text-dim);
    user-select: text;
  }
</style>
