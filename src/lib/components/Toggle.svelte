<script lang="ts">
  let { checked = $bindable(false), disabled = false, onchange }: { checked?: boolean; disabled?: boolean; onchange?: (v: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={checked ? "On" : "Off"}
  class="switch"
  class:on={checked}
  {disabled}
  onclick={() => {
    checked = !checked;
    onchange?.(checked);
  }}
>
  <span class="knob"></span>
</button>

<style>
  .switch {
    width: 42px;
    height: 22px;
    padding: 0;
    border-radius: 999px;
    position: relative;
    background: var(--bg);
    flex-shrink: 0;
  }
  .switch.on {
    background: var(--accent);
    border-color: var(--accent);
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--text-dim);
    transition: transform 0.15s;
  }
  .on .knob {
    transform: translateX(20px);
    background: white;
  }
</style>
