<template>
  <div
    v-if="stats.length > 0"
    class="hourly-table"
  >
    <!-- Header simple avec titre -->
    <div class="table-header">
      <div class="header-left">
        <h3>📅 Statistiques par Heure (UTC)</h3>
      </div>
    </div>
    
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th
              v-if="props.stats15min"
              style="width: 30px;"
            />
            <th>Heure</th>
            <th>ATR Moyen</th>
            <th>Volatilite %</th>
            <th>Body Range %</th>
            <th>Direction Strength</th>
            <th>Noise Ratio</th>
            <th>Breakouts %</th>
          </tr>
        </thead>
        <tbody>
          <template
            v-for="stat in stats"
            :key="stat.hour"
          >
            <!-- Ligne horaire normale -->
            <tr>
              <td
                v-if="props.stats15min"
                class="expand-cell"
              >
                <button
                  v-if="getQuartersForHour(stat.hour).length > 0"
                  class="expand-btn"
                  :class="{ expanded: expandedHours.includes(stat.hour) }"
                  :title="expandedHours.includes(stat.hour) ? 'Replier' : 'Voir 15min'"
                  @click="toggleExpand(stat.hour)"
                >
                  ▶
                </button>
              </td>
              <td class="hour-cell">
                {{ formatHour(stat.hour) }}
                <span
                  v-if="stat.hour === props.bestQuarter[0]"
                  class="star"
                >⭐</span>
              </td>
              <td :class="metricClass('atr', stat.atr_mean)">{{ formatVal(stat.atr_mean) }} {{ props.unit || 'pips' }}</td>
              <td :class="metricClass('volatility', stat.volatility_mean)">{{ (stat.volatility_mean * 100).toFixed(2) }}%</td>
              <td :class="metricClass('bodyrange', stat.body_range_mean)">
                {{ Math.abs(stat.body_range_mean).toFixed(2) }}%
                <span style="font-size: 0.8em; opacity: 0.7;">{{ stat.body_range_mean >= 0 ? '↗' : '↘' }}</span>
              </td>
              <td :class="metricClass('volumeimbalance', Math.abs(stat.volume_imbalance_mean))">{{ (stat.volume_imbalance_mean * 100).toFixed(2) }}%</td>
              <td :class="metricClass('noiseratio', stat.noise_ratio_mean)">{{ stat.noise_ratio_mean.toFixed(2) }}%</td>
              <td :class="metricClass('breakout', stat.breakout_percentage)">{{ stat.breakout_percentage.toFixed(2) }}%</td>
            </tr>

            <!-- Accordion 15-minutes -->
            <tr
              v-if="expandedHours.includes(stat.hour) && props.stats15min"
              class="accordion-row"
            >
              <td
                :colspan="props.stats15min ? 8 : 7"
                class="accordion-cell"
              >
                <QuarterDetails
                  :hour="stat.hour"
                  :quarters="getQuartersForHour(stat.hour)"
                  :best-quarter="props.bestQuarter"
                  :unit="props.unit || 'pts'"
                  :symbol="props.symbol"
                  @entry-point-analyze="(h, q) => emit('entry-point-analyze', h, q)"
                />
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { HourlyStats, Stats15Min } from '../stores/volatility'
import { calculateStraddleScore } from '../utils/volatilityScore'
import QuarterDetails from './QuarterDetails.vue'
import { pipsToDisplayValue } from '../utils/assetUnit'

interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  mean_range: number
  total_candles: number
}

interface ScoredSlice {
  hour: number
  quarter: number
  score: number
}

const props = defineProps<{
  stats: HourlyStats[]
  bestQuarter: [number, number]  // [hour, quarter] - meilleur quarter de la journée
  stats15min?: Stats15Min[]  // Stats 15-minutes optionnels
  globalMetrics?: GlobalMetrics // Pour normalisation (ATR, Tick Quality)
  pointValue?: number // Valeur d'un point pour normalisation (ex: 0.001 pour JPY)
  unit?: string // Unité d'affichage (pips, points, $)
  symbol?: string // Symbole pour conversion pips/points
}>()

const emit = defineEmits<{
  'entry-point-analyze': [hour: number, quarter: number]
}>()

/** Dénormalise les pips en valeur réelle pour les assets en $ ou pts */
function formatVal(pipsValue: number): string {
  const display = props.symbol ? pipsToDisplayValue(pipsValue, props.symbol) : pipsValue
  return display.toFixed(1)
}

const stats = computed(() => props.stats ?? [])

// État du drawer
const expandedHours = ref<number[]>([])

// TOP 3 réactif : se recalcule quand stats15min ou symbol changent
const top3Slices = computed<Array<{ hour: number; quarter: number }>>(() => {
  if (!props.stats15min || props.stats15min.length === 0) return []
  try {
    const scoredSlices = props.stats15min.map((slice: Stats15Min): ScoredSlice => ({
      hour: slice.hour,
      quarter: slice.quarter,
      score: calculateStraddleScore(slice)
    }))
    return scoredSlices
      .sort((a: ScoredSlice, b: ScoredSlice) => b.score - a.score)
      .slice(0, 3)
      .map((item: ScoredSlice) => ({ hour: item.hour, quarter: item.quarter }))
  } catch {
    return []
  }
})

function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}

// Fonctions pour accordion 15-minutes
function toggleExpand(hour: number) {
  const idx = expandedHours.value.indexOf(hour)
  if (idx > -1) {
    expandedHours.value.splice(idx, 1)
  } else {
    expandedHours.value.push(hour)
  }
}

function getQuartersForHour(hour: number) {
  if (!props.stats15min) return []
  
  // Créer les 4 quarters (0-3) pour cette heure, en cherchant les stats s'ils existent
  const quarters = []
  for (let q = 0; q < 4; q++) {
    const stat = props.stats15min.find(s => s.hour === hour && s.quarter === q)
    if (stat) {
      quarters.push(stat)
    } else {
      // Créer un quarter vide s'il n'existe pas dans stats_15min
      quarters.push({
        hour,
        quarter: q,
        candle_count: 0,
        atr_mean: 0,
        atr_max: 0,
        max_true_range: 0,
        volatility_mean: 0,
        range_mean: 0,
        body_range_mean: 0,
        shadow_ratio_mean: 0,
        volume_imbalance_mean: 0,
        noise_ratio_mean: 0,
        breakout_percentage: 0,
        events: [],
        peak_duration_minutes: undefined,
        volatility_half_life_minutes: undefined,
        recommended_trade_expiration_minutes: undefined,
      } as Stats15Min)
    }
  }
  return quarters
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
/* Expand button */
.expand-cell {
  text-align: center;
  width: 30px;
  padding: 0 !important;
}

.expand-btn {
  background: none;
  border: none;
  color: #58a6ff;
  cursor: pointer;
  font-size: 1em;
  padding: 5px 8px;
  transition: transform 0.3s ease;
}

.expand-btn:hover {
  color: #79c0ff;
}

.expand-btn.expanded {
  transform: rotate(90deg);
}

/* Accordion row */
.accordion-row {
  background-color: #0d1117;
}

.metric-excellent { color: #10b981; font-weight: 600; }
.metric-good { color: #3b82f6; }
.metric-acceptable { color: #f59e0b; }
.metric-poor { color: #ef4444; font-weight: 600; }

.accordion-cell {
  padding: 0 !important;
}

.hourly-table {
  background: #161b22;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  border: 1px solid #30363d;
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.table-header {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  margin-bottom: 2rem;
  gap: 2rem;
  flex-shrink: 0;
}

.header-left h3 {
  margin: 0;
  color: #e6edf3;
}

.table-container {
  overflow-x: auto;
  overflow-y: auto;
  flex: 1;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

thead {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: white;
  position: sticky;
  top: 0;
  z-index: 10;
}

th {
  padding: 1rem;
  text-align: left;
  font-weight: bold;
  white-space: nowrap;
}

td {
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #30363d;
  color: #e6edf3;
}

tbody tr:hover {
  background: #0d1117;
}

.best-hour {
  background: #2d2715 !important;
  font-weight: bold;
}

.best-hour:hover {
  background: #3d3715 !important;
}

.hour-cell {
  font-weight: bold;
  color: #58a6ff;
}

.star {
  margin-left: 0.5rem;
}

.star-15min {
  margin-left: 0.3rem;
  color: #fbbf24;
  font-size: 0.9em;
}

.quality-score {
  display: inline-block;
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-weight: bold;
  color: white;
  min-width: 40px;
  text-align: center;
}

.quality-score.straddle {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
}

.quality-score.excellent {
  background: #22c55e;
}

.range-threshold {
  background: rgba(34, 197, 94, 0.1);
  border-radius: 4px;
  padding: 0.25rem 0.5rem;
}

.badge-threshold {
  display: inline-block;
  background: #22c55e;
  color: white;
  padding: 0.15rem 0.5rem;
  border-radius: 3px;
  font-size: 0.75rem;
  font-weight: bold;
  margin-left: 0.5rem;
}

.quality-score.good {
  background: #3b82f6;
}

.quality-score.fair {
  background: #f59e0b;
}

.quality-score.poor {
  background: #ef4444;
}

.events-cell {
  font-size: 0.85rem;
  text-align: center;
}

.event-badge-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid;
  font-size: 0.85em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  background: transparent;
  white-space: nowrap;
}

.event-badge-btn.high {
  color: #58a6ff;
  border-color: #58a6ff;
}

.event-badge-btn.high:hover {
  background: rgba(88, 166, 255, 0.1);
  border-color: #79c0ff;
  transform: translateY(-1px);
}

.event-badge-btn.medium {
  color: #58a6ff;
  border-color: #58a6ff;
}

.event-badge-btn.medium:hover {
  background: rgba(88, 166, 255, 0.1);
  border-color: #79c0ff;
  transform: translateY(-1px);
}

.no-event {
  color: #6e7681;
  font-size: 1.2rem;
}
</style>

