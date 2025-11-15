<template>
  <div class="hourly-scalping-container">
    <div class="table-header">
      <h3>📊 Volatilité par Heure (Paris) - Cliquez pour voir les 15-minutes</h3>
      <p class="subtitle">Vue horaire + expansion 15-minute en accordion</p>
    </div>

    <table class="hourly-table">
      <thead>
        <tr>
          <th class="expand-col"></th>
          <th>Heure (Paris)</th>
          <th>ATR Moyen</th>
          <th>Range</th>
          <th>Volatilité %</th>
          <th>Body Range %</th>
          <th>Tick Quality</th>
          <th>Noise Ratio</th>
          <th>Breakout %</th>
          <th>Potentiel Straddle</th>
          <th>Événements</th>
        </tr>
      </thead>
      <tbody>
        <!-- Boucle sur chaque heure -->
        <template v-for="(hour, hourIdx) in hourlyStats" :key="`hour-${hour.hour}`">
          <!-- ROW: Heure principale (clickable) -->
          <tr class="hour-row" :class="{ expanded: expandedHours.includes(hour.hour) }">
            <td class="expand-col">
              <button
                class="expand-btn"
                :class="{ expanded: expandedHours.includes(hour.hour) }"
                @click="toggleExpand(hour.hour)"
                :title="expandedHours.includes(hour.hour) ? 'Replier' : 'Voir 15 minutes'"
              >
                ▶
              </button>
            </td>
            <td class="hour-cell">
              <strong>{{ String(hour.hour).padStart(2, '0') }}:00-{{ String((hour.hour + 1) % 24).padStart(2, '0') }}:00</strong>
            </td>
            <td class="number">{{ formatNumber(hour.atr_mean, 2) }}</td>
            <td class="number">{{ formatNumber(hour.range_mean, 4) }}</td>
            <td class="number percentage">{{ formatNumber(hour.volatility_mean, 1) }}%</td>
            <td class="number percentage">{{ formatNumber(hour.body_range_mean, 1) }}%</td>
            <td class="number">{{ formatNumber(hour.tick_quality_mean, 2) }}</td>
            <td class="number">{{ formatNumber(hour.noise_ratio_mean, 2) }}</td>
            <td class="number percentage">{{ formatNumber(hour.breakout_percentage, 1) }}%</td>
            <td class="number potentiel" :class="getStraddleClass(hour.volatility_mean)">
              {{ calculateStraddlePotential(hour) }}
            </td>
            <td class="events-cell">
              <div v-if="hour.events && hour.events.length > 0" class="event-badges">
                <span
                  v-for="event in hour.events"
                  :key="`${event.event_name}-${event.datetime}`"
                  class="event-badge"
                  :title="`${event.event_name} (${event.impact})`"
                >
                  {{ getEventFlag(event.event_name) }}
                </span>
              </div>
              <span v-else class="no-events">-</span>
            </td>
          </tr>

          <!-- ACCORDION: Les 4 tranches de 15min pour cette heure -->
          <tr
            v-if="expandedHours.includes(hour.hour)"
            class="accordion-row"
          >
            <td colspan="11" class="accordion-content">
              <div class="scalping-details">
                <div class="scalping-subtitle">
                  Détail 15-minutes pour {{ String(hour.hour).padStart(2, '0') }}:00
                </div>
                
                <table class="scalping-table">
                  <thead>
                    <tr>
                      <th>Tranche</th>
                      <th>ATR</th>
                      <th>Vol %</th>
                      <th>Body %</th>
                      <th>Quality</th>
                      <th>Noise</th>
                      <th>Breakout %</th>
                      <th>Score</th>
                      <th>Événements</th>
                    </tr>
                  </thead>
                  <tbody>
                    <!-- Les 4 tranches de 15min pour cette heure -->
                    <tr
                      v-for="quarter in getQuartersForHour(hour.hour)"
                      :key="`${hour.hour}-${quarter.quarter}`"
                      class="scalping-row"
                      :class="getQualityClass(calculateQualityScore(quarter))"
                    >
                      <td class="time-cell">
                        {{ String(hour.hour).padStart(2, '0') }}:{{ String(quarter.quarter * 15).padStart(2, '0') }}-{{ String(hour.hour).padStart(2, '0') }}:{{ String(Math.min(quarter.quarter * 15 + 15, 60)).padStart(2, '0') }}
                      </td>
                      <td class="number">{{ formatNumber(quarter.atr_mean, 2) }}</td>
                      <td class="number">{{ formatNumber(quarter.volatility_mean, 1) }}%</td>
                      <td class="number">{{ formatNumber(quarter.body_range_mean, 1) }}%</td>
                      <td class="number">{{ formatNumber(quarter.tick_quality_mean, 2) }}</td>
                      <td class="number">{{ formatNumber(quarter.noise_ratio_mean, 2) }}</td>
                      <td class="number">{{ formatNumber(quarter.breakout_percentage, 1) }}%</td>
                      <td class="quality-score" :class="getQualityClass(calculateQualityScore(quarter))">
                        {{ calculateQualityScore(quarter).toFixed(0) }}
                      </td>
                      <td class="events-cell">
                        <div v-if="quarter.events && quarter.events.length > 0" class="event-badges-small">
                          <span
                            v-for="event in quarter.events"
                            :key="`${event.event_name}-${event.datetime}`"
                            class="event-badge-small"
                            :title="`${event.event_name} (${event.impact})`"
                          >
                            {{ getEventFlag(event.event_name) }}
                          </span>
                        </div>
                        <span v-else class="no-events">-</span>
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </td>
          </tr>
        </template>
      </tbody>
    </table>

    <div class="legend">
      <div class="legend-item">
        <span class="quality high">██</span> Excellente tranche (≥70)
      </div>
      <div class="legend-item">
        <span class="quality medium">██</span> Bonne tranche (40-70)
      </div>
      <div class="legend-item">
        <span class="quality low">██</span> Faible tranche (<40)
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { HourlyStats, Stats15Min } from '../stores/volatility'
import { getEventTranslation } from '../stores/eventTranslations'

const props = defineProps<{
  hourlyStats: HourlyStats[]
  stats15min: Stats15Min[]
  bestHours?: number[]
}>()

const expandedHours = ref<number[]>([])

// Toggle expand/collapse pour une heure
function toggleExpand(hour: number) {
  const idx = expandedHours.value.indexOf(hour)
  if (idx > -1) {
    expandedHours.value.splice(idx, 1)
  } else {
    expandedHours.value.push(hour)
  }
}

// Récupère les 4 tranches de 15min pour une heure donnée
function getQuartersForHour(hour: number): Stats15Min[] {
  return props.stats15min
    .filter(stat => stat.hour === hour)
    .sort((a, b) => a.quarter - b.quarter)
}

// Calcule le score de qualité (0-100) pour une tranche
function calculateQualityScore(stat: Stats15Min): number {
  if (stat.candle_count === 0) return 0
  
  const volatility_score = Math.min(stat.volatility_mean * 5, 40)
  const breakout_score = Math.min(stat.breakout_percentage / 2.5, 30)
  const quality_score = Math.min(stat.tick_quality_mean * 10, 30)
  
  return Math.round(volatility_score + breakout_score + quality_score)
}

// Classe CSS pour qualité
function getQualityClass(score: number): string {
  if (score >= 70) return 'high-quality'
  if (score >= 40) return 'medium-quality'
  return 'low-quality'
}

// Calcule le potentiel Straddle pour une heure
function calculateStraddlePotential(hour: HourlyStats): string {
  if (hour.volatility_mean < 10) return '❌'
  if (hour.volatility_mean < 20) return '⚠️'
  if (hour.volatility_mean < 30) return '✅'
  return '🔥'
}

// Classe CSS pour le potentiel Straddle
function getStraddleClass(volatility: number): string {
  if (volatility < 10) return 'bad'
  if (volatility < 20) return 'medium'
  if (volatility < 30) return 'good'
  return 'excellent'
}

// Format les nombres
function formatNumber(value: number, decimals: number): string {
  if (isNaN(value) || value === 0) return '0'
  return value.toFixed(decimals)
}

// Récupère le drapeau du pays
function getEventFlag(eventName: string): string {
  const translation = getEventTranslation(eventName)
  return translation?.flag || '🌍'
}
</script>

<style scoped>
.hourly-scalping-container {
  padding: 20px;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 8px;
  margin: 20px 0;
  color: #e0e0e0;
}

.table-header {
  margin-bottom: 20px;
  border-bottom: 2px solid #00d4ff;
  padding-bottom: 10px;
}

.table-header h3 {
  margin: 0 0 5px 0;
  color: #00d4ff;
  font-size: 1.3em;
}

.subtitle {
  margin: 0;
  color: #b0b0b0;
  font-size: 0.9em;
  font-style: italic;
}

.hourly-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9em;
  font-family: 'Courier New', monospace;
}

.hourly-table thead {
  background-color: #0f3460;
  position: sticky;
  top: 0;
  z-index: 10;
}

.hourly-table th {
  padding: 10px 5px;
  text-align: right;
  border: 1px solid #1a5276;
  color: #00d4ff;
  font-weight: bold;
  white-space: nowrap;
}

.hourly-table th.expand-col {
  width: 30px;
  text-align: center;
}

.expand-col {
  width: 30px;
  text-align: center;
  padding: 0 !important;
}

.expand-btn {
  background: none;
  border: none;
  color: #00d4ff;
  cursor: pointer;
  font-size: 1.2em;
  padding: 5px 10px;
  transition: transform 0.3s ease;
}

.expand-btn.expanded {
  transform: rotate(90deg);
}

.expand-btn:hover {
  color: #fff;
}

.hourly-table tbody tr.hour-row {
  border-bottom: 1px solid #2a2a3e;
  transition: background-color 0.2s ease;
}

.hourly-table tbody tr.hour-row:hover {
  background-color: rgba(0, 212, 255, 0.05);
}

.hourly-table tbody tr.hour-row.expanded {
  background-color: rgba(0, 212, 255, 0.1);
  border-bottom: 3px solid #00d4ff;
}

.hour-cell {
  text-align: left;
  font-weight: bold;
  color: #00d4ff;
  min-width: 100px;
}

.number {
  text-align: right;
  color: #e0e0e0;
  padding-right: 10px;
}

.percentage {
  color: #ffd700;
}

.potentiel {
  font-size: 1.2em;
  text-align: center;
  font-weight: bold;
}

.potentiel.bad {
  color: #ff3c3c;
}

.potentiel.medium {
  color: #ffa500;
}

.potentiel.good {
  color: #00c864;
}

.potentiel.excellent {
  color: #ff0000;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.events-cell {
  text-align: center;
}

.event-badges {
  display: flex;
  gap: 4px;
  justify-content: center;
  flex-wrap: wrap;
}

.event-badge {
  display: inline-block;
  font-size: 1em;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.event-badge:hover {
  transform: scale(1.2);
}

.no-events {
  color: #666;
  font-style: italic;
}

/* ACCORDION CONTENT */
.accordion-row {
  background-color: rgba(0, 212, 255, 0.05);
  border: 2px solid #00d4ff;
}

.accordion-row td {
  padding: 20px !important;
}

.accordion-content {
  background: #0f2a3e;
  border-radius: 8px;
}

.scalping-subtitle {
  color: #00d4ff;
  font-weight: bold;
  font-size: 1.1em;
  margin-bottom: 15px;
}

.scalping-table {
  width: 100%;
  border-collapse: collapse;
  background: #1a1a2e;
  border-radius: 6px;
  overflow: hidden;
}

.scalping-table thead {
  background-color: #0f3460;
}

.scalping-table th {
  padding: 8px 5px;
  text-align: right;
  border: 1px solid #1a5276;
  color: #00d4ff;
  font-weight: bold;
  font-size: 0.85em;
}

.scalping-table tbody tr {
  border-bottom: 1px solid #2a2a3e;
}

.scalping-table tbody tr:hover {
  background-color: rgba(0, 212, 255, 0.05);
}

.scalping-row {
  padding: 8px;
}

.scalping-row.high-quality {
  background-color: rgba(0, 200, 100, 0.08);
}

.scalping-row.medium-quality {
  background-color: rgba(255, 165, 0, 0.08);
}

.scalping-row.low-quality {
  background-color: rgba(255, 60, 60, 0.08);
}

.scalping-table td {
  padding: 8px 5px;
  border: 1px solid #2a2a3e;
  text-align: right;
  font-size: 0.85em;
}

.time-cell {
  text-align: left;
  color: #00d4ff;
  font-weight: bold;
  min-width: 80px;
}

.quality-score {
  font-weight: bold;
  border-radius: 4px;
  padding: 4px 8px !important;
}

.quality-score.high-quality {
  background-color: rgba(0, 200, 100, 0.3);
  color: #00c864;
}

.quality-score.medium-quality {
  background-color: rgba(255, 165, 0, 0.3);
  color: #ffa500;
}

.quality-score.low-quality {
  background-color: rgba(255, 60, 60, 0.3);
  color: #ff3c3c;
}

.event-badges-small {
  display: flex;
  gap: 2px;
  justify-content: center;
  flex-wrap: wrap;
}

.event-badge-small {
  display: inline-block;
  font-size: 0.9em;
  cursor: pointer;
}

.legend {
  display: flex;
  gap: 30px;
  margin-top: 20px;
  padding-top: 15px;
  border-top: 1px solid #2a2a3e;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85em;
  color: #b0b0b0;
}

.quality {
  font-size: 0.8em;
}

.quality.high {
  color: #00c864;
}

.quality.medium {
  color: #ffa500;
}

.quality.low {
  color: #ff3c3c;
}
</style>
