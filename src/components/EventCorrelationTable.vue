<template>
  <div class="impact-ranking">
    <h3>📊 Impact du {{ formatDateRange(eventImpact.datetime) }} au {{ formatDateRange(eventImpact.last_datetime) }} ({{ eventImpact.event_count }} occurrences)</h3>
    <table class="impact-table">
      <thead>
        <tr>
          <th>Rang</th>
          <th>Paire</th>
          <th style="cursor: pointer" @click="$emit('sort')">Volatilité Event</th>
          <th title="Variation moyenne 7j avant l'événement, même heure">Vol. Baseline</th>
          <th>Multiplicateur</th>
          <th>Observations</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(impact, idx) in eventImpact.pair_impacts" :key="impact.symbol" :class="{ 'highlighted-row': idx === 0 }">
          <td class="rank">{{ idx + 1 }}</td>
          <td><strong>{{ impact.symbol }}</strong></td>
          <td class="volatility-value">{{ impact.event_volatility.toFixed(2) }} pips</td>
          <td class="baseline-value">{{ impact.baseline_volatility.toFixed(2) }} pips</td>
          <td :class="getMultiplierClass(impact.multiplier)">
            <span class="multiplier-value">{{ impact.multiplier.toFixed(1) }}x</span>
          </td>
          <td class="observations-cell">✓ Analysée</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'
import type { EventImpactResult } from '../composables/useEventCorrelationByEvent'

defineProps<{
  eventImpact: EventImpactResult
  formatDateRange: (date: string) => string
  getMultiplierClass: (multiplier: number) => string
}>()

defineEmits<{ sort: [] }>()
</script>

<style scoped>
.impact-ranking { margin-top: 30px; }
.impact-ranking h3 { color: #e6edf3; margin-bottom: 15px; }
.impact-table { width: 100%; border-collapse: collapse; background: #161b22; border-radius: 8px; }
.impact-table thead { background: #1c2128; }
.impact-table th { padding: 12px; text-align: left; color: #e6edf3; font-weight: 600; border-bottom: 2px solid #30363d; }
.impact-table td { padding: 12px; color: #8b949e; border-bottom: 1px solid #30363d; }
.impact-table tbody tr:hover { background: #1c2128; }
.impact-table tbody tr.highlighted-row { background: rgba(248, 81, 73, 0.1); }
.rank { font-weight: 600; color: #58a6ff; width: 50px; }
.volatility-value, .baseline-value { font-family: monospace; color: #58a6ff; }
.multiplier-value { font-weight: bold; }
.mult-extreme { color: #f85149; background: rgba(248, 81, 73, 0.1); padding: 2px 6px; border-radius: 4px; }
.mult-very-high { color: #d29922; background: rgba(210, 153, 34, 0.1); padding: 2px 6px; border-radius: 4px; }
.mult-high { color: #3fb950; background: rgba(63, 185, 80, 0.1); padding: 2px 6px; border-radius: 4px; }
.mult-medium { color: #58a6ff; background: rgba(88, 166, 255, 0.1); padding: 2px 6px; border-radius: 4px; }
.mult-low { color: #8b949e; background: rgba(139, 148, 158, 0.1); padding: 2px 6px; border-radius: 4px; }
.observations-cell { text-align: center; }
</style>
