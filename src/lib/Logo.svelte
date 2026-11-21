<script lang="ts">
  let { size = 32 }: { size?: number } = $props();
</script>

<!-- Placeholder mark: a crimson rounded square with a cut "Z". Flat, no gradient. -->
<svg width={size} height={size} viewBox="0 0 64 64" role="img" aria-label="ztmystrap">
  <rect x="2" y="2" width="60" height="60" rx="14" fill="#b11226" />
  <path d="M18 17h28v7L29 40h17v7H18v-7l17-16H18z" fill="#fff" />
</svg>
