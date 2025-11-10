<template>
  <div v-if="loadingHeatmap" class="loading">
    <div class="spinner"></div>
    <p>Génération de la heatmap...</p>
  </div>

  <div v-if="heatmapData && !loadingHeatmap" class="heatmap-container">
    <table class="heatmap-table">
      <thead>
        <tr>
          <th class="header-corner">Type d'événement</th>
          <th v-for="pair in heatmapData.pairs" :key="pair">{{ pair }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="eventType in heatmapData.event_types" :key="eventType.name">
          <td class="event-type-cell">
            <div class="event-type-name">{{ eventType.name }}</div>
            <div class="event-count">({{ eventType.count }} evt)</div>
          </td>
          <td v-for="pair in heatmapData.pairs" :key="`${eventType.name}-${pair}`" class="heatmap-cell" :class="getHeatmapClass(getHeatmapValue(eventType.name, pair))">
            <span class="cell-value">{{ getHeatmapValue(eventType.name, pair) }}</span>
          </td>
        </tr>
      </tbody>
    </table>

    <div class="heatmap-legend">
      <div class="legend-title">Légende :</div>
      <div class="legend-items">
        <div class="legend-item"><div class="legend-color heat-very-high"></div><span>>500 pips</span></div>
        <div class="legend-item"><div class="legend-color heat-high"></div><span>200-500 pips</span></div>
        <div class="legend-item"><div class="legend-color heat-medium"></div><span>100-200 pips</span></div>
        <div class="legend-item"><div class="legend-color heat-low"></div><span>50-100 pips</span></div>
        <div class="legend-item"><div class="legend-color heat-very-low"></div><span><50 pips</span></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface HeatmapData {
  period: string
  pairs: string[]
  event_types: { name: string; count: number }[]
  data: { [key: string]: { [key: string]: number } }
}

const loadingHeatmap = ref(false)
const heatmapData = ref<HeatmapData | null>(null)

onMounted(async () => {
  loadingHeatmap.value = true
  try {
    heatmapData.value = await invoke<HeatmapData>('get_correlation_heatmap', { monthsBack: 6 })
  } catch (error) {
    console.error('Erreur heatmap:', error)
    heatmapData.value = null
  } finally {
    loadingHeatmap.value = false
  }
})

function getHeatmapValue(eventType: string, pair: string): number {
  return heatmapData.value?.data[eventType]?.[pair] || 0
}

function getHeatmapClass(value: number): string {
  if (value >= 500) return 'heat-very-high'
  if (value >= 200) return 'heat-high'
  if (value >= 100) return 'heat-medium'
  if (value >= 50) return 'heat-low'
  return 'heat-very-low'
}
</script>

<style scoped>
.loading {
  text-align: center;
  padding: 60px 20px;
  color: #e2e8f0;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #2d3748;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.heatmap-container {
  background: #1a202c;
  padding: 25px;
  border-radius: 12px;
  border: 1px solid #2d3748;
}

.heatmap-table {
  width: 100%;
  border-collapse: collapse;
  margin-bottom: 30px;
  font-size: 0.75em;
}

.header-corner {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 700;
  padding: 8px;
  border: 1px solid #4a5568;
}

.heatmap-table th {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 600;
  padding: 8px;
  border: 1px solid #4a5568;
  text-align: center;
  font-size: 0.8em;
}

.event-type-cell {
  background: #2d3748;
  padding: 8px;
  border: 1px solid #4a5568;
  text-align: left;
}

.event-type-name {
  font-weight: 700;
  color: #e2e8f0;
  margin-bottom: 2px;
  font-size: 0.8em;
}

.event-count {
  font-size: 0.7em;
  color: #a0aec0;
}

.heatmap-cell {
  padding: 8px;
  text-align: center;
  border: 1px solid #4a5568;
  transition: all 0.2s;
}

.heatmap-cell:hover {
  transform: scale(1.05);
  box-shadow: 0 0 10px rgba(102, 126, 234, 0.5);
}

.cell-value {
  font-weight: 700;
  font-size: 0.9em;
}

.heat-very-high {
  background: #7f1d1d;
  color: #fca5a5;
}

.heat-high {
  background: #dc2626;
  color: white;
}

.heat-medium {
  background: #f59e0b;
  color: white;
}

.heat-low {
  background: #3b82f6;
  color: white;
}

.heat-very-low {
  background: #4a5568;
  color: #e2e8f0;
}

.heatmap-legend {
  display: flex;
  gap: 20px;
  align-items: center;
  padding: 20px;
  background: #2d3748;
  border-radius: 8px;
}

.legend-title {
  font-weight: 700;
  color: #e2e8f0;
}

.legend-items {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  color: #e2e8f0;
}

.legend-color {
  width: 30px;
  height: 20px;
  border-radius: 4px;
  border: 1px solid #4a5568;
}
</style>
