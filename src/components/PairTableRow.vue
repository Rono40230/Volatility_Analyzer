<template>
  <tr>
    <td><span class="badge badge-pair">{{ pair.symbol }}</span></td>
    <td><span class="badge badge-timeframe">{{ pair.timeframe }}</span></td>
    <td class="text-right">{{ pair.row_count.toLocaleString() }}</td>
    <td>{{ formatDate(pair.last_updated) }}</td>
    <td class="filename-small">{{ pair.last_imported_file }}</td>
    <td class="text-center">
      <span class="quality-score" :class="`quality-${qualityLevel(pair.quality_score)}`">
        ★ {{ (pair.quality_score * 100).toFixed(0) }}%
      </span>
    </td>
  </tr>
</template>

<script setup lang="ts">
import { defineProps } from 'vue'

interface PairFileInfo {
  symbol: string
  timeframe: string
  row_count: number
  last_updated: string
  last_imported_file: string
  quality_score: number
}

defineProps<{
  pair: PairFileInfo
}>()

function formatDate(date: string): string {
  return new Date(date).toLocaleDateString('fr-FR')
}

function qualityLevel(score: number): string {
  if (score >= 0.9) return 'excellent'
  if (score >= 0.7) return 'good'
  if (score >= 0.5) return 'fair'
  return 'poor'
}
</script>

<style scoped>
.badge { padding: 4px 8px; border-radius: 4px; font-size: 0.9em; font-weight: 500; white-space: nowrap; }
.badge-pair { background: #238636; color: #fff; }
.badge-timeframe { background: #1f6feb; color: #fff; }
.text-right { text-align: right; }
.text-center { text-align: center; }
.filename-small { font-size: 0.9em; color: #8b949e; max-width: 150px; overflow: hidden; text-overflow: ellipsis; }
.quality-score { font-weight: 600; padding: 2px 6px; border-radius: 3px; }
.quality-excellent { background: #1f6feb; color: #fff; }
.quality-good { background: #238636; color: #fff; }
.quality-fair { background: #9e6a03; color: #fff; }
.quality-poor { background: #da3633; color: #fff; }
</style>
