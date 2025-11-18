<template>
  <div v-if="loadingHeatmap" class="loading">
    <div class="spinner"></div>
    <p>Génération de la heatmap...</p>
  </div>

  <div v-if="heatmapData && !loadingHeatmap" class="heatmap-container">
    <div class="heatmap-header">
      <div class="heatmap-scale">
        <span class="scale-item"><span class="scale-color heat-very-low"></span>≥12 pips</span>
        <span class="scale-item"><span class="scale-color heat-low"></span>9-11 pips</span>
        <span class="scale-item"><span class="scale-color heat-medium"></span>6-8 pips</span>
        <span class="scale-item"><span class="scale-color heat-high"></span>3-5 pips</span>
        <span class="scale-item"><span class="scale-color heat-very-high"></span>&lt;3 pips</span>
      </div>
    </div>

    <!-- Filtres -->
    <div class="filters-container">
      <div class="filter-group">
        <label for="volatility-threshold">Volatilité minimale :</label>
        <select id="volatility-threshold" v-model.number="minVolatilityThreshold" class="filter-select">
          <option value="3">≥3 pips</option>
          <option value="6">≥6 pips</option>
          <option value="9">≥9 pips</option>
          <option value="12">≥12 pips</option>
        </select>
      </div>
      <div class="filter-group">
        <label for="max-events">Nombre d'événements max :</label>
        <select id="max-events" v-model.number="maxEventsToDisplay" class="filter-select">
          <option value="5">5 événements</option>
          <option value="10">10 événements</option>
          <option value="15">15 événements</option>
          <option value="20">20 événements</option>
        </select>
      </div>
    </div>
    <div class="heatmap-wrapper">
      <table class="heatmap-table">
      <thead>
        <tr>
          <th class="header-corner">Type d'événement</th>
          <th v-for="pair in heatmapData.pairs" :key="pair">{{ pair }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="eventType in getSortedEventTypes()" :key="eventType.name">
          <td class="event-type-cell" :class="{ 'no-data': eventType.has_data === false }">
            <div class="event-type-name">{{ getFormattedEventName(eventType.name) }}</div>
          </td>
          <td v-for="pair in heatmapData.pairs" :key="`${eventType.name}-${pair}`" :class="['heatmap-cell', getHeatmapValue(eventType.name, pair) >= minVolatilityThreshold ? getHeatmapClass(getHeatmapValue(eventType.name, pair)) : 'empty-cell']">
            <span v-if="getHeatmapValue(eventType.name, pair) >= minVolatilityThreshold" class="cell-value">{{ getHeatmapValue(eventType.name, pair) }}</span>
          </td>
        </tr>
      </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getEventTranslation } from '../stores/eventTranslations'

interface HeatmapData {
  period: string
  pairs: string[]
  event_types: { name: string; count: number; has_data?: boolean }[]
  data: { [key: string]: { [key: string]: number } }
}

interface Props {
  calendarId: number | null
  availablePairs: string[]
}

const props = defineProps<Props>()

const loadingHeatmap = ref(false)
const heatmapData = ref<HeatmapData | null>(null)
const minVolatilityThreshold = ref(3)  // Seuil minimum en pips: 3, 6, 9, 12
const maxEventsToDisplay = ref(10)     // Nombre max: 5, 10, 15, 20

async function loadHeatmap() {
  if (!props.calendarId || props.availablePairs.length === 0) {
    heatmapData.value = null
    return
  }

  loadingHeatmap.value = true
  try {
    heatmapData.value = await invoke<HeatmapData>('get_correlation_heatmap', { 
      calendarId: props.calendarId,
      pairs: props.availablePairs
    })
    console.log('📊 Heatmap data loaded:', heatmapData.value)
    
    // Debug: inspecter la structure data
    if (heatmapData.value?.data) {
      console.log('🔍 Data structure keys:', Object.keys(heatmapData.value.data))
      for (const [eventType, pairData] of Object.entries(heatmapData.value.data)) {
        console.log(`  📌 ${eventType}:`, pairData)
        for (const [pair, value] of Object.entries(pairData as any)) {
          console.log(`    ✓ ${pair}: ${value}`)
        }
      }
    }
  } catch (error) {
    console.error('Erreur heatmap:', error)
    heatmapData.value = null
  } finally {
    loadingHeatmap.value = false
  }
}

onMounted(() => {
  loadHeatmap()
})

// Recharger si le calendrier change
watch(() => props.calendarId, () => {
  loadHeatmap()
})

function getHeatmapValue(eventType: string, pair: string): number {
  return heatmapData.value?.data[eventType]?.[pair] || 0
}

function getEventAverage(eventType: string): number {
  if (!heatmapData.value) return 0
  const pairs = heatmapData.value.pairs
  const values = pairs.map(pair => getHeatmapValue(eventType, pair))
  const sum = values.reduce((acc, val) => acc + val, 0)
  return values.length > 0 ? sum / values.length : 0
}

function getSortedEventTypes() {
  if (!heatmapData.value) return []
  
  // Trier les événements par moyenne décroissante
  let sorted = [...heatmapData.value.event_types].sort((a, b) => {
    const avgA = getEventAverage(a.name)
    const avgB = getEventAverage(b.name)
    return avgB - avgA // Ordre décroissant
  })

  // Limiter au nombre maximum d'événements (filtre par volatilité appliqué au niveau cellule)
  sorted = sorted.slice(0, maxEventsToDisplay.value)

  return sorted
}

function getHeatmapClass(value: number): string {
  if (value >= 12) return 'heat-very-low'
  if (value >= 9) return 'heat-low'
  if (value >= 6) return 'heat-medium'
  if (value >= 3) return 'heat-high'
  return 'heat-very-high'
}

function getFormattedEventName(eventName: string): string {
  const translation = getEventTranslation(eventName)
  return `${eventName} (${translation.fr}) ${translation.flag}`
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

.heatmap-header {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 2px solid #2d3748;
}

.heatmap-title {
  font-size: 1.1em;
  font-weight: 600;
  color: #e2e8f0;
  flex: 1;
}

.heatmap-scale {
  display: flex;
  gap: 20px;
  align-items: center;
  font-size: 0.85em;
  color: #cbd5e0;
}

.scale-item {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.scale-color {
  display: inline-block;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  border: 1px solid #1a202c;
}

.filters-container {
  display: flex;
  gap: 30px;
  align-items: center;
  margin-bottom: 25px;
  padding: 15px;
  background: #2d3748;
  border-radius: 8px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter-group label {
  font-size: 0.9em;
  font-weight: 600;
  color: #e2e8f0;
  white-space: nowrap;
}

.filter-select {
  padding: 8px 12px;
  background: #ffffff;
  color: #000000;
  border: 1px solid #4a5568;
  border-radius: 6px;
  font-size: 0.9em;
  cursor: pointer;
  transition: all 0.2s ease;
}

.filter-select:hover {
  border-color: #667eea;
  box-shadow: 0 0 8px rgba(102, 126, 234, 0.3);
}

.filter-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 12px rgba(102, 126, 234, 0.5);
}

.heatmap-wrapper {
  overflow-x: auto;
  margin-bottom: 30px;
  border-radius: 8px;
}

.heatmap-table {
  width: 100%;
  border-collapse: separate;
  border-spacing: 4px;
  margin-bottom: 30px;
  font-size: 0.95em;
  background: #0f1419;
  padding: 15px;
  border-radius: 8px;
}

.header-corner {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 700;
  padding: 12px 8px;
  border: 1px solid #4a5568;
  min-width: 200px;
  text-align: left;
}

.heatmap-table th {
  background: #2d3748;
  color: #e2e8f0;
  font-weight: 600;
  padding: 12px 8px;
  border: 1px solid #4a5568;
  text-align: center;
  font-size: 0.85em;
  min-width: 60px;
}

.event-type-cell {
  background: #2d3748;
  padding: 12px;
  border: 1px solid #4a5568;
  text-align: left;
  min-width: 200px;
  font-weight: 500;
}

.event-type-cell.no-data {
  opacity: 0.5;
  background: #1a1f2e;
}

.no-data-badge {
  display: inline-block;
  font-size: 0.75em;
  margin-left: 8px;
  padding: 3px 6px;
  background: #7f1d1d;
  color: #fca5a5;
  border-radius: 4px;
  font-weight: bold;
}

.has-data-badge {
  display: inline-block;
  font-size: 0.75em;
  margin-left: 8px;
  padding: 3px 6px;
  background: #15803d;
  color: #bbf7d0;
  border-radius: 4px;
  font-weight: bold;
}

.event-type-name {
  font-weight: 700;
  color: #e2e8f0;
  margin-bottom: 4px;
  font-size: 0.9em;
}

.event-count {
  font-size: 0.75em;
  color: #a0aec0;
}

.heatmap-cell {
  padding: 16px 12px;
  text-align: center;
  border: 1px solid #4a5568;
  transition: all 0.3s ease;
  min-width: 70px;
  min-height: 60px;
  display: table-cell;
  vertical-align: middle;
  cursor: pointer;
}

.heatmap-cell:hover {
  transform: scale(1.08);
  box-shadow: 0 0 15px rgba(102, 126, 234, 0.7);
  border-color: #667eea;
  z-index: 10;
}

.cell-value {
  font-weight: 700;
  font-size: 0.9em;
  display: block;
}

.empty-cell {
  background: #1a1f2e;
  color: transparent;
}

.empty-cell:hover {
  transform: none;
  box-shadow: none;
  border-color: #4a5568;
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
  background: #16a34a;
  color: white;
}

.heat-very-low {
  background: #22c55e;
  color: #1a202c;
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
