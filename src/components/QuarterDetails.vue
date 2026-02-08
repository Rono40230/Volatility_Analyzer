<template>
  <div class="scalping-details">
    <table class="scalping-table">
      <thead>
        <tr>
          <th>Tranche</th>
          <th>ATR Moyen</th>
          <th>Max Spike</th>
          <th>Volatilité %</th>
          <th>Body Range %</th>
          <th>Direction Strength</th>
          <th>Noise Ratio</th>
          <th>Breakouts %</th>
          <th title="Minutes volatilité > 80% pic">Peak (min)</th>
          <th title="Minutes pour -50% volatilité">Half-Life (min)</th>
          <th title="Durée optimale fermeture trade">Trade Exp (min)</th>
          <th>Analyser</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="quarter in quarters"
          :key="`${hour}-${quarter.quarter}`"
          class="scalping-row"
          :class="{ 'top3-slice': isBestSlice(quarter.quarter) }"
        >
          <td class="time-cell">
            <span v-if="isBestSlice(quarter.quarter)" class="top3-star">⭐</span>
            {{ formatQuarterLabel(hour, quarter.quarter) }}
          </td>
          <td><UnitDisplay :value="quarter.atr_mean" :unit="unit" :symbol="symbol" /></td>
          <td><UnitDisplay :value="quarter.max_true_range ?? 0" :unit="unit" :symbol="symbol" /></td>
          <td>{{ (quarter.volatility_mean * 100).toFixed(2) }}%</td>
          <td>
            {{ Math.abs(quarter.body_range_mean).toFixed(2) }}%
            <span style="font-size: 0.8em; opacity: 0.7;">{{ quarter.body_range_mean >= 0 ? '↗' : '↘' }}</span>
          </td>
          <td>{{ (quarter.volume_imbalance_mean * 100).toFixed(2) }}%</td>
          <td>{{ quarter.noise_ratio_mean.toFixed(2) }}%</td>
          <td>{{ quarter.breakout_percentage.toFixed(2) }}%</td>
          <td class="duration-cell" :title="`Peak duration moyen: ${quarter.peak_duration_mean ?? 'N/A'} min`">
            {{ quarter.peak_duration_mean !== undefined ? quarter.peak_duration_mean + ' min' : '—' }}
          </td>
          <td class="duration-cell" :title="`Half-life moyen: ${quarter.volatility_half_life_mean ?? 'N/A'} min`">
            {{ quarter.volatility_half_life_mean !== undefined ? quarter.volatility_half_life_mean + ' min' : '—' }}
          </td>
          <td
            class="trade-exp-cell"
            :class="{ 'warning': isTradeExpTooLong(quarter) }"
            :title="`Fermer trade après ${quarter.recommended_trade_expiration_mean ?? 'N/A'} min`"
          >
            {{ quarter.recommended_trade_expiration_mean !== undefined ? quarter.recommended_trade_expiration_mean + ' min' : '—' }}
            <span v-if="isTradeExpTooLong(quarter)" class="warning-icon">⚠️</span>
          </td>
          <td class="analyze-cell">
            <button
              class="analyze-btn"
              :disabled="quarter.candle_count === 0"
              @click="emit('analyze-quarter', hour, quarter.quarter)"
            >
              Analyser
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { Stats15Min } from '../stores/volatility'
import UnitDisplay from './UnitDisplay.vue'

const props = defineProps<{
  hour: number
  quarters: Stats15Min[]
  bestQuarter: [number, number]
  unit: string
  symbol?: string
}>()

const emit = defineEmits<{
  'analyze-quarter': [hour: number, quarter: number]
}>()

function formatQuarterLabel(hour: number, quarter: number): string {
  const startMin = quarter * 15
  const endMin = startMin + 15
  if (endMin >= 60) {
    const endHour = (hour + 1) % 24
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(endHour).padStart(2, '0')}:00`
  }
  return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(hour).padStart(2, '0')}:${String(endMin).padStart(2, '0')}`
}

function isBestSlice(quarter: number): boolean {
  return props.bestQuarter[0] === props.hour && props.bestQuarter[1] === quarter
}

function isTradeExpTooLong(slice: Stats15Min): boolean {
  return (slice.recommended_trade_expiration_minutes ?? 0) > 150
}
</script>

<style scoped>
.scalping-details { padding: 15px; background: #161b22; border-top: 2px solid #21262d; }
.scalping-table { width: 100%; border-collapse: collapse; font-size: 0.85em; margin: 0; }
.scalping-table thead { background: #1f6feb; }
.scalping-table th { padding: 0.5rem; text-align: left; font-weight: bold; border: 1px solid #30363d; white-space: nowrap; }
.scalping-table td { padding: 0.5rem; border: 1px solid #30363d; }
.scalping-row { background: #2d333b; }
.scalping-row:hover { background: #353d48; }
.scalping-row.top3-slice { background: rgba(251, 191, 36, 0.25); border-left: 4px solid #fbbf24; }
.scalping-row.top3-slice:hover { background: rgba(251, 191, 36, 0.35); }
.top3-star { color: #fbbf24; font-weight: bold; margin-right: 4px; }
.time-cell { font-weight: bold; color: #58a6ff; }
.duration-cell { text-align: center; color: #e6edf3; font-size: 0.85em; }
.analyze-cell { text-align: center; }
.analyze-btn { background: #1f6feb; border: none; color: #ffffff; padding: 6px 10px; border-radius: 6px; font-size: 0.85em; cursor: pointer; transition: background 0.2s; }
.analyze-btn:hover:not(:disabled) { background: #388bfd; }
.analyze-btn:disabled { background: #4b5563; cursor: not-allowed; opacity: 0.7; }
</style>
