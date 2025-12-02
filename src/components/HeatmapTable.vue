<template>
  <div class="heatmap-wrapper">
    <table class="heatmap-table">
      <thead>
        <tr>
          <th class="header-corner">Type d'événement</th>
          <th v-for="pair in pairs" :key="pair">{{ pair }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="eventType in sortedEventTypes" :key="eventType.name">
          <td class="event-type-cell" :class="{ 'no-data': eventType.has_data === false }">
            <div class="event-type-name">{{ getFormattedEventName(eventType.name) }}</div>
          </td>
          <td v-for="pair in pairs" :key="`${eventType.name}-${pair}`" :class="['heatmap-cell', getHeatmapValue(eventType.name, pair) >= minVolatility ? getHeatmapClass(getHeatmapValue(eventType.name, pair)) : 'empty-cell']">
            <span v-if="getHeatmapValue(eventType.name, pair) >= minVolatility" class="cell-value">{{ getHeatmapValue(eventType.name, pair).toFixed(1) }}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
interface EventTypeEntry {
  name: string
  has_data?: boolean
}

defineProps<{
  pairs: string[]
  sortedEventTypes: EventTypeEntry[]
  minVolatility: number
  getHeatmapValue: (e: string, p: string) => number
  getHeatmapClass: (v: number) => string
  getFormattedEventName: (e: string) => string
}>()
</script>

<style scoped>
.heatmap-wrapper { overflow-x: auto; margin-bottom: 20px; }
.heatmap-table { width: 100%; border-collapse: collapse; background: #0d1117; border: 1px solid #30363d; }
.heatmap-table thead { background: #161b22; }
.heatmap-table th { padding: 12px; text-align: center; color: #8b949e; font-weight: 600; border-bottom: 1px solid #30363d; }
.header-corner { text-align: left; }
.event-type-cell { background: #0d1117; padding: 12px; font-weight: 600; color: #c9d1d9; border-bottom: 1px solid #30363d; }
.event-type-cell.no-data { opacity: 0.6; }
.event-type-name { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.heatmap-cell { padding: 12px; text-align: center; border: 1px solid #30363d; }
.heatmap-cell.empty-cell { background: #0d1117; color: #6e7681; }
.cell-value { font-weight: 600; font-size: 0.9em; }
.heat-very-high { background: #f5222d; color: white; }
.heat-high { background: #ff7a45; color: white; }
.heat-medium { background: #fadb14; color: black; }
.heat-low { background: #95de64; color: black; }
.heat-very-low { background: #52c41a; color: white; }
</style>
