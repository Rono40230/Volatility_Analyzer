<template>
  <div class="container">
    <div class="controls">
      <label>CSV Backtest: <input type="file" @change="handleFileUpload" accept=".csv" class="input"/></label>
    </div>
    <div v-if="entryTimingLoading" class="spinner">⏳</div>
    <div v-else-if="entryTimingError" class="error">{{ entryTimingError }}</div>
    <div v-else-if="entryTimingResults.length === 0" class="empty">📭 Uploadez un CSV</div>
    <table v-else>
      <thead><tr><th>Offset</th><th>Win %</th><th>Avg Profit</th><th>Whipsaw %</th><th>Score</th><th>Best</th></tr></thead>
      <tbody><tr v-for="r in entryTimingResults" :key="r.entry_offset_minutes"><td>{{ r.entry_offset_minutes }}</td><td>{{ r.win_rate.toFixed(1) }}</td><td>{{ r.avg_profit_pips.toFixed(2) }}</td><td>{{ r.whipsaw_rate.toFixed(1) }}</td><td>{{ r.quality_score.toFixed(0) }}</td><td>{{ r.is_best ? '✅' : '' }}</td></tr></tbody>
    </table>
  </div>
</template>
<script setup lang="ts">
import { ref } from 'vue'
import { useRetrospectiveAnalysis } from '../composables/useRetrospectiveAnalysis'

const { entryTimingLoading, entryTimingError, entryTimingResults, analyzeEntryTiming } = useRetrospectiveAnalysis()

async function handleFileUpload(e: any) {
  const file = e.target.files?.[0]
  if (!file) return
  const text = await file.text()
  const rows = text.split('\n').slice(1).filter(r => r.trim())
  const data = rows.map(r => {
    const [offset, wins, losses, profit] = r.split(',')
    return [parseInt(offset), parseFloat(profit), parseInt(wins) > 0]
  })
  await analyzeEntryTiming(data)
}
</script>
<style scoped>
.container { padding: 20px; background: #0d1117; border-radius: 8px; color: #e2e8f0; }
.controls { margin-bottom: 15px; }
.input { padding: 8px; background: #161b22; border: 1px solid #30363d; color: #e2e8f0; border-radius: 4px; }
.spinner, .empty { text-align: center; color: #8b949e; padding: 30px; }
.error { background: #3d2626; color: #f85149; padding: 10px; border-radius: 4px; }
table { width: 100%; border-collapse: collapse; background: #161b22; }
th, td { padding: 10px; text-align: left; border-bottom: 1px solid #30363d; }
th { color: #1f6feb; font-weight: 700; }
</style>
