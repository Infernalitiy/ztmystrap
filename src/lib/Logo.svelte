<script lang="ts">
  let { size = 32 }: { size?: number } = $props();
</script>

<!-- Placeholder mark: a rounded square with a lightning "Z". -->
<svg width={size} height={size} viewBox="0 0 64 64" aria-label="ztmystrap">
  <defs>
    <linearGradient id="zg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#9b82ff" />
      <stop offset="1" stop-color="#5b3df0" />
    </linearGradient>
  </defs>
  <rect x="4" y="4" width="56" height="56" rx="14" fill="url(#zg)" />
  <path d="M20 19h24l-15 17h11l-16 15 5-11h-9l14-15H20z" fill="white" />
</svg>
