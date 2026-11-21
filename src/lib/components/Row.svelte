<script lang="ts">
  import type { Snippet } from "svelte";
  let { title, desc = "", children }: { title: string; desc?: string; children?: Snippet } = $props();
</script>

<div class="card row">
  <div class="label">
    <div>{title}</div>
    {#if desc}<div class="dim">{desc}</div>{/if}
  </div>
  {@render children?.()}
</div>
