<template>
  <div class="scalping-details">
    <table class="scalping-table">
      <thead>
        <tr>
          <th>Tranche</th>
          <th>ATR Moyen</th>
          <th>Volatilité %</th>
          <th>Body Range %</th>
          <th>Direction Strength</th>
          <th>Noise Ratio</th>
          <th>Breakouts %</th>
          <th title="Minutes volatilité > 70% pic">
            Peak (min)
          </th>
          <th title="Minutes pour -50% volatilité">
            Half-Life (min)
          </th>
          <th title="Durée optimale fermeture trade">
            Trade Exp (min)
          </th>
          <th>Action</th>
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
            <span
              v-if="isBestSlice(quarter.quarter)"
              class="top3-star"
            >⭐</span>
            {{ formatQuarterLabel(hour, quarter.quarter) }}
          </td>
          <td :class="metricClass('atr', quarter.atr_mean)">{{ formatVal(quarter.atr_mean) }} {{ unit || 'pips' }}</td>
          <td :class="metricClass('volatility', quarter.volatility_mean)">{{ (quarter.volatility_mean * 100).toFixed(2) }}%</td>
          <td :class="metricClass('bodyrange', quarter.body_range_mean)">
            {{ Math.abs(quarter.body_range_mean).toFixed(2) }}%
            <span style="font-size: 0.8em; opacity: 0.7;">{{ quarter.body_range_mean >= 0 ? '↗' : '↘' }}</span>
          </td>
          <td :class="metricClass('volumeimbalance', Math.abs(quarter.volume_imbalance_mean))">{{ (quarter.volume_imbalance_mean * 100).toFixed(2) }}%</td>
          <td :class="metricClass('noiseratio', quarter.noise_ratio_mean)">{{ quarter.noise_ratio_mean.toFixed(2) }}x</td>
          <td :class="metricClass('breakout', quarter.breakout_percentage)">{{ quarter.breakout_percentage.toFixed(2) }}%</td>
          <td
            class="duration-cell"
            :title="`Peak duration moyen: ${quarter.peak_duration_mean ?? 'N/A'} min`"
          >
            {{ quarter.peak_duration_mean !== undefined ? quarter.peak_duration_mean + ' min' : '—' }}
          </td>
          <td
            class="duration-cell"
            :title="`Half-life moyen: ${quarter.volatility_half_life_mean ?? 'N/A'} min`"
          >
            {{ quarter.volatility_half_life_mean !== undefined ? quarter.volatility_half_life_mean + ' min' : '—' }}
          </td>
          <td
            class="trade-exp-cell"
            :class="{ 'warning': isTradeExpTooLong(quarter) }"
            :title="`Fermer trade après ${quarter.recommended_trade_expiration_mean ?? 'N/A'} min`"
          >
            {{ quarter.recommended_trade_expiration_mean !== undefined ? quarter.recommended_trade_expiration_mean + ' min' : '—' }}
            <span
              v-if="isTradeExpTooLong(quarter)"
              class="warning-icon"
            >⚠️</span>
          </td>
          <td class="analyze-cell">
            <button
              class="analyze-btn"
              :disabled="quarter.candle_count === 0"
              @click="emit('entry-point-analyze', hour, quarter.quarter)"
            >
              📊 Analyser
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import type { Stats15Min } from '../stores/volatility'
import { pipsToDisplayValue } from '../utils/assetUnit'

const props = defineProps<{
  hour: number
  quarters: Stats15Min[]
  bestQuarter: [number, number]
  unit: string
  symbol?: string
}>()

const emit = defineEmits<{
  'entry-point-analyze': [hour: number, quarter: number]
}>()

function formatVal(pipsValue: number): string {
  const display = props.symbol ? pipsToDisplayValue(pipsValue, props.symbol) : pipsValue
  return display.toFixed(1)
}

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

function metricClass(metric: string, value: number): string {
  let quality: string
  switch (metric) {
    case 'atr':
      quality = value > 50 ? 'excellent' : value > 20 ? 'good' : value > 10 ? 'acceptable' : 'poor'
      break
    case 'volatility':
      quality = value >= 0.30 ? 'excellent' : value >= 0.15 ? 'good' : value >= 0.05 ? 'acceptable' : 'poor'
      break
    case 'bodyrange':
      quality = Math.abs(value) > 45 ? 'excellent' : Math.abs(value) > 35 ? 'good' : Math.abs(value) > 15 ? 'acceptable' : 'poor'
      break
    case 'noiseratio':
      quality = value < 2.0 ? 'excellent' : value < 3.0 ? 'good' : value < 4.0 ? 'acceptable' : 'poor'
      break
    case 'volumeimbalance':
      quality = value > 0.5 ? 'excellent' : value > 0.3 ? 'good' : value > 0.1 ? 'acceptable' : 'poor'
      break
    case 'breakout':
      quality = value >= 20 ? 'excellent' : value >= 10 ? 'good' : value >= 5 ? 'acceptable' : 'poor'
      break
    default:
      return ''
  }
  return `metric-${quality}`
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
.metric-excellent { color: #10b981; font-weight: 600; }
.metric-good { color: #3b82f6; }
.metric-acceptable { color: #f59e0b; }
.metric-poor { color: #ef4444; font-weight: 600; }
</style>
