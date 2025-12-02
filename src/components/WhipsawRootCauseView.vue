<template>
  <div class="container">
    <div class="controls">
      <div><label>Early Whipsaws: <input v-model.number="earlyCount" type="number" class="input"/></label></div>
      <div><label>Early Avg Loss: <input v-model.number="earlyLoss" type="number" class="input"/></label></div>
      <div><label>Late Whipsaws: <input v-model.number="lateCount" type="number" class="input"/></label></div>
      <div><label>Late Avg Loss: <input v-model.number="lateLoss" type="number" class="input"/></label></div>
      <button @click="analyze" class="btn">Analyser</button>
    </div>
    <div v-if="whipsawLoading" class="spinner">⏳</div>
    <div v-else-if="whipsawError" class="error">{{ whipsawError }}</div>
    <div v-else-if="!whipsawResults" class="empty">📭 Entrez les données</div>
    <div v-else class="result">
      <div class="row"><span>Early Whipsaws:</span><strong>{{ whipsawResults.early_count }} ({{ whipsawResults.early_percentage.toFixed(1) }}%)</strong></div>
      <div class="row"><span>Early Avg Loss:</span><strong>{{ whipsawResults.early_avg_loss_pips.toFixed(2) }} pips</strong></div>
      <div class="row"><span>Late Whipsaws:</span><strong>{{ whipsawResults.late_count }} ({{ whipsawResults.late_percentage.toFixed(1) }}%)</strong></div>
      <div class="row"><span>Late Avg Loss:</span><strong>{{ whipsawResults.late_avg_loss_pips.toFixed(2) }} pips</strong></div>
      <div class="row"><span>Type Dominant:</span><strong>{{ whipsawResults.dominant_type }}</strong></div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

const { whipsawLoading, whipsawError, whipsawResults, analyzeWhipsawRootCause } = useRetrospectiveAnalysis()
const earlyCount = ref(0), earlyLoss = ref(0), lateCount = ref(0), lateLoss = ref(0)

async function analyze() {
  await analyzeWhipsawRootCause(earlyCount.value, earlyLoss.value, lateCount.value, lateLoss.value)
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { display: flex; gap: 10px; margin-bottom: 15px; flex-wrap: wrap; }
.input { padding: 8px; background: #161b22; border: 1px solid #30363d; color: #e2e8f0; border-radius: 4px; width: 120px; }
.btn { padding: 8px 16px; background: #238636; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
.btn:hover { background: #2ea043; }
.spinner, .empty { text-align: center; color: #8b949e; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
.result { display: flex; flex-direction: column; gap: 10px; }
.row { display: flex; justify-content: space-between; padding: 10px; background: #161b22; border-radius: 4px; }
strong { color: #1f6feb; font-weight: 700; }
</style>
