<template>
  <div class="heatmap-header">
    <div class="heatmap-scale">
      <span class="scale-item"><span class="scale-color heat-very-low" />≥12 pips</span>
      <span class="scale-item"><span class="scale-color heat-low" />9-11 pips</span>
      <span class="scale-item"><span class="scale-color heat-medium" />6-8 pips</span>
      <span class="scale-item"><span class="scale-color heat-high" />3-5 pips</span>
      <span class="scale-item"><span class="scale-color heat-very-high" />&lt;3 pips</span>
    </div>
  </div>
</template>

<script setup lang="ts">
</script>

<style scoped>
.heatmap-header { display: flex; justify-content: center; align-items: center; margin-bottom: 20px; padding-bottom: 15px; border-bottom: 2px solid #2d3748; }
.heatmap-scale { display: flex; gap: 20px; align-items: center; font-size: 0.85em; color: #cbd5e0; }
.scale-item { display: flex; align-items: center; gap: 6px; white-space: nowrap; }
.scale-color { display: inline-block; width: 16px; height: 16px; border-radius: 3px; border: 1px solid #1a202c; }
.heat-very-low { background: #52c41a; }
.heat-low { background: #95de64; }
.heat-medium { background: #fadb14; }
.heat-high { background: #ff7a45; }
.heat-very-high { background: #f5222d; }
</style>
