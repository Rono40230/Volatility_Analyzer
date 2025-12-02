<template>
  <tr
    v-if="expandedHours.includes(stat.hour) && stats15min"
    class="accordion-row"
  >
    <td :colspan="colspan" class="accordion-cell">
      <div class="scalping-details">
        <table class="details-table">
          <thead>
            <tr>
              <th>Créneau</th>
              <th>ATR</th>
              <th>Range</th>
              <th>Volatilité %</th>
              <th>Body Range %</th>
              <th>Vol. Imbalance %</th>
              <th>Noise Ratio</th>
              <th>Breakouts %</th>
              <th>Peak Dur</th>
              <th>Half-life</th>
              <th>Exp Dur</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="quarter in quarters"
              :key="quarter.quarter"
              :class="{ 'best-slice': isBestSliceInHour(stat.hour, quarter.quarter) }"
            >
              <td class="time-cell">
                <span
                  v-if="isBestSliceInHour(stat.hour, quarter.quarter)"
                  class="top3-star"
                >⭐</span>
                {{ formatTime(stat.hour, quarter.quarter) }}
              </td>
              <td>{{ formatATR(quarter.atr_mean, estimatedPrice) }}</td>
              <td>{{ formatATR(quarter.range_mean, estimatedPrice) }}</td>
              <td>{{ (quarter.volatility_mean * 100).toFixed(2) }}%</td>
              <td>
                {{ Math.abs(quarter.body_range_mean).toFixed(2) }}%
                <span style="font-size: 0.8em; opacity: 0.7;">{{ quarter.body_range_mean >= 0 ? '↗' : '↘' }}</span>
              </td>
              <td>{{ (quarter.volume_imbalance_mean * 100).toFixed(2) }}%</td>
              <td>{{ quarter.noise_ratio_mean.toFixed(2) }}</td>
              <td>{{ quarter.breakout_percentage.toFixed(2) }}%</td>
              <td class="duration-cell" :title="`${quarter.peak_duration_minutes ?? 'N/A'} min`">
                {{ quarter.peak_duration_minutes !== undefined ? quarter.peak_duration_minutes + ' min' : '—' }}
              </td>
              <td class="duration-cell" :title="`${quarter.volatility_half_life_minutes ?? 'N/A'} min`">
                {{ quarter.volatility_half_life_minutes !== undefined ? quarter.volatility_half_life_minutes + ' min' : '—' }}
              </td>
              <td :class="{ 'warning-exp': isTradeExpTooLong(quarter) }">
                {{ quarter.recommended_trade_expiration_minutes !== undefined ? quarter.recommended_trade_expiration_minutes + ' min' : '—' }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </td>
  </tr>
</template>

<script setup lang="ts">
import type { HourlyStats, Stats15Min } from '../stores/volatility'
import { formatATR, isTradeExpTooLong } from '../utils/hourlyTableUtils'

defineProps<{
  stat: HourlyStats
  expandedHours: number[]
  stats15min: Stats15Min[] | undefined
  quarters: Stats15Min[]
  estimatedPrice: number
  colspan: number
}>()

defineEmits<{
  updateTop3: []
}>()

function formatTime(hour: number, quarter: number): string {
  const h = String(hour).padStart(2, '0')
  const start = String(quarter * 15).padStart(2, '0')
  const end = String(Math.min(quarter * 15 + 15, 60)).padStart(2, '0')
  return `${h}:${start}-${h}:${end}`
}

function isBestSliceInHour(hour: number, quarter: number): boolean {
  if (!stats15min) return false
  const quartersInHour = stats15min.filter(s => s.hour === hour)
  if (quartersInHour.length === 0) return false
  
  const { calculateSliceScore } = require('../utils/hourlyTableUtils')
  const scoredQuarters = quartersInHour.map(q => ({
    quarter: q.quarter,
    score: calculateSliceScore(q)
  }))
  
  interface ScoredQuarter {
    quarter: number
    score: number
  }
  
  const bestQuarter = scoredQuarters.reduce((p: ScoredQuarter, c: ScoredQuarter) => 
    c.score > p.score ? c : p
  )
  
  return bestQuarter.quarter === quarter
}
</script>

<style scoped>
.accordion-row { background: #0d1117; }
.accordion-cell { padding: 15px 0 !important; }
.scalping-details { padding: 0 15px; }
.details-table { width: 100%; border-collapse: collapse; font-size: 0.9em; }
.details-table th { text-align: left; padding: 10px; border-bottom: 1px solid #30363d; font-weight: 600; }
.details-table td { padding: 8px; border-bottom: 1px solid #1a202c; }
.details-table tr:hover { background: #161b22; }
.best-slice { background: rgba(16, 185, 129, 0.1) !important; }
.best-slice .top3-star { color: #fbbf24; font-size: 1.2em; }
.time-cell { min-width: 130px; }
.duration-cell { min-width: 80px; }
.warning-exp { color: #fbbf24; font-weight: 600; }
</style>
