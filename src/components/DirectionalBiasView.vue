<template>
  <div class="container">
    <div class="controls">
      <div><label>UP Wins: <input v-model.number="upWins" type="number" class="input"/></label></div>
      <div><label>DOWN Wins: <input v-model.number="downWins" type="number" class="input"/></label></div>
      <div><label>Whipsaws: <input v-model.number="whipsaws" type="number" class="input"/></label></div>
      <button @click="analyze" class="btn">Analyser</button>
    </div>
    <div v-if="biasLoading" class="spinner">⏳</div>
    <div v-else-if="biasError" class="error">{{ biasError }}</div>
    <div v-else-if="!biasResults" class="empty">📭 Entrez les données</div>
    <div v-else class="result">
      <div class="row"><span>UP Wins:</span><strong>{{ biasResults.up_wins_count }}</strong></div>
      <div class="row"><span>DOWN Wins:</span><strong>{{ biasResults.down_wins_count }}</strong></div>
      <div class="row"><span>Classification:</span><strong>{{ biasResults.classification }}</strong></div>
      <div class="row"><span>Asymétrie:</span><strong>{{ biasResults.asymmetry_percent.toFixed(1) }}%</strong></div>
      <div class="row"><span>Confiance:</span><strong>{{ biasResults.confidence_level }}</strong></div>
    </div>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

const { biasLoading, biasError, biasResults, analyzeDirectionalBias } = useRetrospectiveAnalysis()
const upWins = ref(0), downWins = ref(0), whipsaws = ref(0)

async function analyze() {
  await analyzeDirectionalBias(upWins.value, downWins.value, whipsaws.value)
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { display: flex; gap: 10px; margin-bottom: 15px; flex-wrap: wrap; }
.input { padding: 8px; background: #161b22; border: 1px solid #30363d; color: #e2e8f0; border-radius: 4px; width: 100px; }
.btn { padding: 8px 16px; background: #238636; color: #fff; border: none; border-radius: 4px; cursor: pointer; }
.btn:hover { background: #2ea043; }
.spinner, .empty { text-align: center; color: #8b949e; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
.result { display: flex; flex-direction: column; gap: 10px; }
.row { display: flex; justify-content: space-between; padding: 10px; background: #161b22; border-radius: 4px; }
strong { color: #1f6feb; font-weight: 700; }
</style>
