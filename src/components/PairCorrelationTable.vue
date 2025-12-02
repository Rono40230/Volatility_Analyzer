<template>
  <div class="correlation-table-container">
    <table class="correlation-table">
      <thead>
        <tr>
          <th>Rang</th>
          <th>Événement</th>
          <th colspan="3" style="text-align: center;">Volatilité observée (pips)</th>
          <th>Score</th>
        </tr>
        <tr>
          <th />
          <th />
          <th>-30mn</th>
          <th>+30mn</th>
          <th>1h total</th>
          <th />
        </tr>
      </thead>
      <tbody>
        <tr v-for="(event, index) in topEvents" :key="event.name" :class="{ 'top-event': index < 3 }">
          <td>#{{ index + 1 }}</td>
          <td class="event-name">{{ getTranslatedName(event.name) }}</td>
          <td class="volatility">{{ event.volatility_before_fmt }}</td>
          <td class="volatility">{{ event.volatility_after_fmt }}</td>
          <td class="volatility-total">{{ event.volatility_total_fmt }}</td>
          <td class="correlation-score">
            <span class="score-value" :class="getScoreClass(event.correlation_score)">
              {{ event.correlation_score.toFixed(1) }}%
            </span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { defineProps, computed } from 'vue'
import type { EventCorrelation } from '../composables/useEventCorrelationByPair'
import { getEventTranslation } from '../stores/eventTranslations'

const props = defineProps<{
  topEvents: EventCorrelation[]
  getScoreClass: (score: number) => string
}>()

function getTranslatedName(eventName: string): string {
  const translation = getEventTranslation(eventName)
  return `${eventName} (${translation.fr}) ${translation.flag}`
}
</script>

<style scoped>
.correlation-table-container { margin: 30px 0; }
.correlation-table { width: 100%; border-collapse: collapse; background: #1a202c; border-radius: 8px; }
.correlation-table thead { background: #2d3748; }
.correlation-table th { padding: 12px; text-align: left; font-weight: 600; color: #e2e8f0; border-bottom: 2px solid #4a5568; }
.correlation-table td { padding: 12px; border-bottom: 1px solid #2d3748; color: #e2e8f0; }
.correlation-table tbody tr:hover { background: #2d3748; }
.correlation-table tbody tr.top-event { background: rgba(34, 197, 94, 0.1); }
.event-name { font-weight: 600; }
.volatility, .volatility-total { font-family: monospace; color: #58a6ff; }
.correlation-score { text-align: center; }
.score-value { font-weight: bold; padding: 2px 6px; border-radius: 4px; }
.score-green { background: rgba(34, 197, 94, 0.2); color: #22c55e; }
.score-orange { background: rgba(245, 158, 11, 0.2); color: #f59e0b; }
.score-red { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
</style>
