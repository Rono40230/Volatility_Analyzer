<template>
  <div class="scalping-table-container">
    <div class="table-header">
      <h3>📊 Statistiques par Tranche de 15 Minutes (Heure de Paris)</h3>
      <p class="subtitle">Vue détaillée pour analyse de scalping robot - 96 tranches / 24h</p>
    </div>

    <table class="scalping-table">
      <thead>
        <tr>
          <th>Heure</th>
          <th>ATR Moyen</th>
          <th>Range</th>
          <th>Volatilité %</th>
          <th>Body Range %</th>
          <th>Shadow Ratio</th>
          <th>Tick Quality</th>
          <th>Volume Imbalance</th>
          <th>Noise Ratio</th>
          <th>Breakout %</th>
          <th>Score Qualité</th>
          <th>Événements</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="(stat, idx) in stats"
          :key="`${stat.hour}-${stat.quarter}`"
          :class="[
            'data-row',
            { 'hour-separator': stat.quarter === 0 && stat.hour !== 0 },
            { 'low-quality': stat.quality_score < 40 },
            { 'medium-quality': stat.quality_score >= 40 && stat.quality_score < 70 },
            { 'high-quality': stat.quality_score >= 70 }
          ]"
        >
          <td class="time-cell">{{ stat.time_label }}</td>
          <td class="number">{{ formatNumber(stat.atr_mean, 2) }}</td>
          <td class="number">{{ formatNumber(stat.range_mean, 4) }}</td>
          <td class="number percentage">{{ formatNumber(stat.volatility_mean, 1) }}%</td>
          <td class="number percentage">{{ formatNumber(stat.body_range_mean, 1) }}%</td>
          <td class="number">{{ formatNumber(stat.shadow_ratio_mean, 2) }}</td>
          <td class="number">{{ formatNumber(stat.tick_quality_mean, 2) }}</td>
          <td class="number">{{ formatNumber(stat.volume_imbalance_mean, 2) }}</td>
          <td class="number">{{ formatNumber(stat.noise_ratio_mean, 2) }}</td>
          <td class="number percentage">{{ formatNumber(stat.breakout_percentage, 1) }}%</td>
          <td class="quality-badge" :class="getQualityClass(stat.quality_score)">
            {{ stat.quality_score.toFixed(0) }}<br/>
            <small>{{ getQualityRating(stat.quality_score) }}</small>
          </td>
          <td class="events-cell">
            <div v-if="stat.events && stat.events.length > 0" class="event-badges">
              <span
                v-for="event in stat.events"
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
      </tbody>
    </table>

    <div class="legend">
      <div class="legend-item">
        <span class="quality high">██</span> Haute qualité (≥70)
      </div>
      <div class="legend-item">
        <span class="quality medium">██</span> Qualité moyenne (40-70)
      </div>
      <div class="legend-item">
        <span class="quality low">██</span> Faible qualité (<40)
      </div>
      <div class="legend-item">
        <span class="event-demo">🇺🇸</span> Événements économiques (drapeau pays)
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Stats15Min } from '../stores/volatility'
import { getEventTranslation } from '../stores/eventTranslations'

const props = defineProps<{
  stats15min: Stats15Min[]
}>()

// Format et affichage
const stats = computed(() => {
  return props.stats15min.map((stat) => ({
    ...stat,
    events: stat.events || [],  // Safeguard: ensure events is always an array
    time_label: `${String(stat.hour).padStart(2, '0')}:${String(stat.quarter * 15).padStart(2, '0')}-${String(stat.hour).padStart(2, '0')}:${String(Math.min(stat.quarter * 15 + 15, 60)).padStart(2, '0')}`,
    quality_score: calculateQualityScore(stat),
  }))
})

// Calcule le score de qualité (0-100)
function calculateQualityScore(stat: Stats15Min): number {
  if (stat.candle_count === 0) return 0

  const volatility_score = Math.min(stat.volatility_mean * 5, 40) // 0-40
  const breakout_score = Math.min(stat.breakout_percentage / 2.5, 30) // 0-30
  const quality_score = Math.min(stat.tick_quality_mean * 10, 30) // 0-30

  return Math.round(volatility_score + breakout_score + quality_score)
}

// Classe de qualité pour couleur
function getQualityClass(score: number): string {
  if (score >= 70) return 'high-quality'
  if (score >= 40) return 'medium-quality'
  return 'low-quality'
}

// Rating textuel
function getQualityRating(score: number): string {
  if (score >= 70) return 'Excellent'
  if (score >= 50) return 'Bon'
  if (score >= 35) return 'Moyen'
  return 'Faible'
}

// Format les nombres avec décimales
function formatNumber(value: number, decimals: number): string {
  if (isNaN(value) || value === 0) return '0'
  return value.toFixed(decimals)
}

// Récupère le drapeau du pays pour un événement
function getEventFlag(eventName: string): string {
  const translation = getEventTranslation(eventName)
  return translation?.flag || '🌍'
}
</script>

<style scoped>
.scalping-table-container {
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

.scalping-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85em;
  font-family: 'Courier New', monospace;
}

.scalping-table thead {
  background-color: #0f3460;
  position: sticky;
  top: 0;
  z-index: 10;
}

.scalping-table th {
  padding: 10px 5px;
  text-align: right;
  border: 1px solid #1a5276;
  color: #00d4ff;
  font-weight: bold;
  white-space: nowrap;
}

.scalping-table th:first-child,
.scalping-table td:first-child {
  text-align: left;
}

.scalping-table tbody tr {
  border-bottom: 1px solid #2a2a3e;
  transition: background-color 0.2s ease;
}

.scalping-table tbody tr:hover {
  background-color: rgba(0, 212, 255, 0.05);
}

.scalping-table tbody tr.hour-separator {
  border-top: 3px solid #00d4ff;
}

.scalping-table td {
  padding: 8px 5px;
  border: 1px solid #2a2a3e;
  text-align: right;
}

.time-cell {
  text-align: left;
  font-weight: bold;
  color: #00d4ff;
  min-width: 90px;
}

.number {
  color: #e0e0e0;
}

.percentage {
  color: #ffd700;
}

.quality-badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-weight: bold;
  min-width: 60px;
}

.quality-badge.high-quality {
  background-color: rgba(0, 200, 100, 0.3);
  color: #00c864;
}

.quality-badge.medium-quality {
  background-color: rgba(255, 165, 0, 0.3);
  color: #ffa500;
}

.quality-badge.low-quality {
  background-color: rgba(255, 60, 60, 0.3);
  color: #ff3c3c;
}

.quality-badge small {
  font-size: 0.7em;
  margin-top: 2px;
  opacity: 0.9;
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
  font-size: 1.2em;
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

.data-row.low-quality {
  opacity: 0.7;
}

.data-row.high-quality td:not(.time-cell) {
  background-color: rgba(0, 200, 100, 0.05);
}

.data-row.medium-quality td:not(.time-cell) {
  background-color: rgba(255, 165, 0, 0.05);
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

.event-demo {
  font-size: 1.1em;
}
</style>
