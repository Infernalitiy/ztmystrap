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
  /* the switch never steps; its knob slides 16px and stretches 4px while pressed */
  .switch,
  .switch:hover,
  .switch:active {
    width: 36px;
    height: 20px;
    padding: 0;
    border-radius: 999px;
    position: relative;
    background: var(--bg);
    flex-shrink: 0;
    transform: none;
  }
  .switch.on,
  .switch.on:hover {
    background: var(--accent);
    border-color: var(--accent);
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 999px;
    background: var(--text-dim);
    transition:
      transform 180ms var(--ease),
      width 90ms var(--ease),
      background-color 180ms var(--ease);
  }
  .switch:active:not(:disabled) .knob {
    width: 18px;
  }
  .on .knob {
    transform: translateX(16px);
    background: #fff;
  }
  .on:active:not(:disabled) .knob {
    transform: translateX(12px);
  }
</style>
