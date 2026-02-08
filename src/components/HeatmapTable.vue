<template>
  <div class="heatmap-wrapper">
    <table class="heatmap-table">
      <thead>
        <tr>
          <th class="header-corner">Type d'événement</th>
          <th 
            v-for="pair in pairs" 
            :key="pair"
            class="sortable-header"
            @click="toggleSort(pair)"
            :title="`Trier par ${pair}`"
          >
            <div class="header-content">
              {{ pair }}
              <span class="sort-icon" v-if="currentSortPair === pair">
                {{ currentSortDir === 'desc' ? '↓' : '↑' }}
              </span>
              <span class="sort-icon placeholder" v-else>↕</span>
            </div>
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="eventType in processedEventTypes" :key="eventType.name">
          <td class="event-type-cell" :class="{ 'no-data': eventType.has_data === false }">
            <div class="event-type-name">
              {{ getFormattedEventName(eventType.name) }}
              <span class="event-count" title="Nombre d'occurrences">({{ eventType.count }} occurrences)</span>
            </div>
          </td>
          <td v-for="pair in pairs" :key="`${eventType.name}-${pair}`" 
              :class="['heatmap-cell', getCellClass(eventType.name, pair)]"
              :title="getCellTitle(eventType.name, pair)"
              @click="shouldShowValue(eventType.name, pair) ? emit('analyze-cell', eventType.name, pair) : null"
              :style="{ cursor: shouldShowValue(eventType.name, pair) ? 'pointer' : 'default' }"
          >
            <span v-if="shouldShowValue(eventType.name, pair)" class="cell-value">
              {{ formaterPointsAvecPips(pair, getHeatmapValue(eventType.name, pair)) }}
              <span v-if="getCellCount(eventType.name, pair) > 0" class="cell-count" :class="{ 'low-sample': getCellCount(eventType.name, pair) < 5 }">
                N={{ getCellCount(eventType.name, pair) }}
              </span>
            </span>
            <span v-else-if="getHeatmapValue(eventType.name, pair) === -1" class="no-data-indicator">N/A</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { formaterPointsAvecPips } from '../utils/pipConverter'

interface EventTypeEntry {
  name: string
  count: number
  has_data?: boolean
}

const props = defineProps<{
  pairs: string[]
  sortedEventTypes: EventTypeEntry[]
  minVolatility: number
  getHeatmapValue: (e: string, p: string) => number
  getHeatmapCount?: (e: string, p: string) => number
  getHeatmapClass: (v: number) => string
  getFormattedEventName: (e: string) => string
}>()

const emit = defineEmits<{
  (e: 'analyze-cell', eventName: string, pair: string): void
}>()

const currentSortPair = ref<string | null>(null)
const currentSortDir = ref<'asc' | 'desc'>('desc')

const processedEventTypes = computed(() => {
  if (!currentSortPair.value) {
    return props.sortedEventTypes
  }

  const sorted = [...props.sortedEventTypes]
  sorted.sort((a, b) => {
    const valA = props.getHeatmapValue(a.name, currentSortPair.value!)
    const valB = props.getHeatmapValue(b.name, currentSortPair.value!)
    
    // Gérer les cas N/A (-1) pour qu'ils soient toujours à la fin
    if (valA === -1 && valB === -1) return 0
    if (valA === -1) return 1
    if (valB === -1) return -1

    if (currentSortDir.value === 'asc') {
      return valA - valB
    } else {
      return valB - valA
    }
  })
  return sorted
})

function toggleSort(pair: string) {
  if (currentSortPair.value === pair) {
    if (currentSortDir.value === 'desc') {
      currentSortDir.value = 'asc'
    } else {
      currentSortPair.value = null
      currentSortDir.value = 'desc'
    }
  } else {
    currentSortPair.value = pair
    currentSortDir.value = 'desc'
  }
}

function shouldShowValue(eventName: string, pair: string): boolean {
  const val = props.getHeatmapValue(eventName, pair)
  return val !== -1 && val >= props.minVolatility
}

function getCellCount(eventName: string, pair: string): number {
  return props.getHeatmapCount ? props.getHeatmapCount(eventName, pair) : 0
}

function getCellTitle(eventName: string, pair: string): string {
  if (!shouldShowValue(eventName, pair)) return ''
  const count = getCellCount(eventName, pair)
  if (count > 0 && count < 5) return `⚠️ N=${count} — échantillon trop petit (< 5), peu fiable`
  if (count > 0) return `Analyser ce setup (N=${count})`
  return 'Analyser ce setup'
}

function getCellClass(eventName: string, pair: string): string {
  const val = props.getHeatmapValue(eventName, pair)
  if (val === -1) return 'no-data-cell'
  if (val < props.minVolatility) return 'filtered-cell'
  const count = getCellCount(eventName, pair)
  const colorClass = props.getHeatmapClass(val)
  return count > 0 && count < 5 ? `${colorClass} low-sample-cell` : colorClass
}
</script>

<style scoped>
.heatmap-wrapper { overflow-x: auto; margin-bottom: 20px; width: 100%; }
.heatmap-table { min-width: 100%; width: auto; border-collapse: collapse; background: #0d1117; border: 1px solid #30363d; }
.heatmap-table thead { background: #161b22; }
.heatmap-table th { padding: 12px; text-align: center; color: #8b949e; font-weight: 600; border-bottom: 1px solid #30363d; }
.sortable-header { cursor: pointer; user-select: none; transition: background 0.2s; white-space: nowrap; }
.sortable-header:hover { background: #1c2128; color: #58a6ff; }
.header-content { display: flex; align-items: center; justify-content: center; gap: 6px; }
.sort-icon { font-size: 0.8em; opacity: 0.7; }
.sort-icon.placeholder { opacity: 0.2; }
.header-corner { text-align: left; }
.event-type-cell { background: #0d1117; padding: 12px; font-weight: 600; color: #c9d1d9; border-bottom: 1px solid #30363d; }
.event-type-cell.no-data { opacity: 0.6; }
.event-type-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 8px; }
.event-count { font-size: 0.85em; color: #8b949e; font-weight: 400; }
.heatmap-cell { padding: 12px; text-align: center; border: 1px solid #30363d; white-space: nowrap; }
.heatmap-cell.empty-cell { background: #0d1117; color: #6e7681; }
.heatmap-cell.no-data-cell { background: #161b22; color: #484f58; font-style: italic; font-size: 0.8em; }
.heatmap-cell.filtered-cell { background: #0d1117; color: #484f58; }
.cell-value { font-weight: 600; font-size: 0.9em; display: flex; flex-direction: column; align-items: center; gap: 2px; }
.cell-count { font-size: 0.7em; font-weight: 400; opacity: 0.7; }
.cell-count.low-sample { color: #f0883e; font-weight: 600; opacity: 1; }
.low-sample-cell { opacity: 0.5; background-image: repeating-linear-gradient(45deg, transparent, transparent 3px, rgba(0,0,0,0.1) 3px, rgba(0,0,0,0.1) 6px) !important; }
.no-data-indicator { opacity: 0.5; }
.heat-extreme { background: #1a7f37; color: white; font-weight: 700; }   /* Exceptionnel (Vert vif) */
.heat-very-high { background: #238636; color: white; }                    /* Excellent (Vert) */
.heat-high { background: #3fb950; color: #0d1117; }                       /* Bon (Vert clair) */
.heat-medium { background: #d29922; color: black; }                       /* Moyen (Orange) */
.heat-low { background: #e16f24; color: white; }                          /* Faible (Orange foncé) */
.heat-very-low { background: #da3633; color: white; }                     /* Très faible (Rouge) */
</style>
