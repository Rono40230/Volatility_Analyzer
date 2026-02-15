<template>
  <div class="backtest-view">
    <div class="placeholder">
      <h2>🔧 Backtest</h2>
      <p>Module de backtest en cours de construction.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
// Placeholder view to avoid build break when imported in App.vue.
</script>

<style scoped>
.backtest-view {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #e6edf3;
}
.placeholder {
  text-align: center;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 24px 32px;
}
.placeholder h2 { margin: 0 0 8px; }
.placeholder p { margin: 0; color: #8b949e; }
</style>
