<template>
  <div v-if="stats.length > 0" class="hourly-table">
    <!-- Header simple avec titre -->
    <div class="table-header">
      <div class="header-left">
        <h3>📅 Statistiques par Heure (Heure de Paris)</h3>
      </div>
    </div>
    
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th v-if="props.stats15min" style="width: 30px;"></th>
            <th>Heure</th>
            <th>ATR Moyen</th>
            <th>Range (H-L)</th>
            <th>Volatilité %</th>
            <th>Body Range %</th>
            <th>Tick Quality</th>
            <th>Noise Ratio</th>
            <th>Breakouts %</th>
            <th>Potentiel Straddle</th>
            <th>Événements</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="stat in stats" :key="stat.hour">
            <!-- Ligne horaire normale -->
            <tr 
              :class="{ 'best-hour': isBestHour(stat.hour) }"
            >
              <td v-if="props.stats15min" class="expand-cell">
                <button
                  v-if="getQuartersForHour(stat.hour).length > 0"
                  class="expand-btn"
                  :class="{ expanded: expandedHours.includes(stat.hour) }"
                  @click="toggleExpand(stat.hour)"
                  :title="expandedHours.includes(stat.hour) ? 'Replier' : 'Voir 15min'"
                >
                  ▶
                </button>
              </td>
              <td class="hour-cell">
                {{ formatHour(stat.hour) }}
                <span v-if="isBestHour(stat.hour)" class="star">⭐</span>
              </td>
              <td>{{ formatNumber(stat.atr_mean, 5) }}</td>
              <td :class="{ 'range-threshold': stat.range_mean > 0.0025 }">
                {{ formatNumber(stat.range_mean, 5) }}
                <span v-if="stat.range_mean > 0.0025" class="badge-threshold">✅ >25pips</span>
              </td>
              <td>{{ (stat.volatility_mean * 100).toFixed(2) }}</td>
              <td>{{ stat.body_range_mean.toFixed(1) }}</td>
              <td>{{ formatNumber(stat.tick_quality_mean, 5) }}</td>
              <td>{{ stat.noise_ratio_mean.toFixed(2) }}</td>
              <td>{{ stat.breakout_percentage.toFixed(1) }}</td>
              <td>
                <div class="quality-score straddle" :class="getQualityClass(stat)">
                  {{ getQualityScore(stat).toFixed(0) }}
                </div>
              </td>
              <td class="events-cell">
                <button
                  v-if="stat.events && stat.events.length > 0"
                  class="event-badge-btn"
                  :class="getEventBadgeClass(stat.events)"
                  @click="selectHour(stat.hour, stat.events)"
                  :title="`${getDistinctEventCount(stat.events)} événement(s) distinct(s) - ${getDistinctHighCount(stat.events)} HIGH, ${getDistinctMediumCount(stat.events)} MEDIUM`"
                >
                  <span v-html="getEventIcon(stat.events)" style="display: inline-flex;"></span>
                  {{ getDistinctEventCount(stat.events) }}
                </button>
                <span v-else class="no-event">—</span>
              </td>
            </tr>

            <!-- Accordion 15-minutes -->
            <tr v-if="expandedHours.includes(stat.hour) && props.stats15min" class="accordion-row">
              <td :colspan="props.stats15min ? 11 : 10" class="accordion-cell">
                <div class="scalping-details">
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
                        <th>Événements</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="quarter in getQuartersForHour(stat.hour)" :key="`${stat.hour}-${quarter.quarter}`" class="scalping-row">
                        <td class="time-cell">{{ String(stat.hour).padStart(2, '0') }}:{{ String(quarter.quarter * 15).padStart(2, '0') }}-{{ String(stat.hour).padStart(2, '0') }}:{{ String(Math.min(quarter.quarter * 15 + 15, 60)).padStart(2, '0') }}</td>
                        <td>{{ formatNumber(quarter.atr_mean, 5) }}</td>
                        <td>{{ (quarter.volatility_mean * 100).toFixed(2) }}</td>
                        <td>{{ quarter.body_range_mean.toFixed(1) }}</td>
                        <td>{{ formatNumber(quarter.tick_quality_mean, 5) }}</td>
                        <td>{{ quarter.noise_ratio_mean.toFixed(2) }}</td>
                        <td>{{ quarter.breakout_percentage.toFixed(1) }}</td>
                        <td class="events-cell">
                          <button
                            v-if="quarter.events && quarter.events.length > 0"
                            class="event-badge-btn"
                            :class="getEventBadgeClass(quarter.events)"
                            @click="selectHour(stat.hour, quarter.events)"
                            style="font-size: 0.8em; padding: 2px 6px;"
                          >
                            <span v-html="getEventIcon(quarter.events)" style="display: inline-flex;"></span>
                            {{ getDistinctEventCount(quarter.events) }}
                          </button>
                          <span v-else class="no-event">—</span>
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
    </div>

    <!-- Drawer des événements -->
    <EventDetailsDrawer
      :isOpen="drawerOpen"
      :selectedHour="selectedHour"
      :allEvents="selectedEvents"
      @close="drawerOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { HourlyStats, EventInHour } from '../stores/volatility'
import EventDetailsDrawer from './EventDetailsDrawer.vue'

const props = defineProps<{
  stats: HourlyStats[]
  bestHours: number[]
  stats15min?: any[]  // Stats 15-minutes optionnels
}>()

// État du drawer
const drawerOpen = ref(false)
const selectedHour = ref<number | null>(null)
const selectedEvents = ref<EventInHour[] | null>(null)
const expandedHours = ref<number[]>([])

function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}

function formatNumber(num: number, decimals: number): string {
  return num.toFixed(decimals)
}

function isBestHour(hour: number): boolean {
  return props.bestHours.includes(hour)
}

function getQualityScore(stat: HourlyStats): number {
  // Calcule le score de potentiel STRADDLE (même logique que backend)
  // Voir src-tauri/src/models/hourly_stats.rs::movement_potential_score_straddle()
  if (stat.candle_count === 0) return 0
  
  let score = 0
  
  // 1. RANGE (60 pts max) - Dominante pour straddle
  if (stat.range_mean > 0.0025) {
    score += 60 // >25 pips = excellent
  } else if (stat.range_mean > 0.0020) {
    score += 50 // 20-25 pips = très bon
  } else if (stat.range_mean > 0.0015) {
    score += 40 // 15-20 pips = bon
  } else if (stat.range_mean > 0.0010) {
    score += 20 // 10-15 pips = acceptable
  }
  
  // 2. ATR (25 pts max) - Volatilité soutenue
  if (stat.atr_mean > 0.0020) {
    score += 25 // >20 pips = excellent
  } else if (stat.atr_mean > 0.0015) {
    score += 20 // 15-20 pips = très bon
  } else if (stat.atr_mean > 0.0010) {
    score += 15 // 10-15 pips = bon
  } else if (stat.atr_mean > 0.0005) {
    score += 8 // 5-10 pips = acceptable
  }
  
  // 3. BodyRange (15 pts max) - Directionnalité
  if (stat.body_range_mean > 45.0) {
    score += 15 // >45% = excellent
  } else if (stat.body_range_mean > 35.0) {
    score += 12 // 35-45% = bon
  } else if (stat.body_range_mean > 25.0) {
    score += 8 // 25-35% = acceptable
  } else if (stat.body_range_mean > 15.0) {
    score += 3 // 15-25% = limite
  }
  
  return Math.min(score, 100)
}

function getQualityClass(stat: HourlyStats): string {
  const score = getQualityScore(stat)
  if (score >= 80) return 'excellent'
  if (score >= 60) return 'good'
  if (score >= 40) return 'fair'
  return 'poor'
}

// ============================================
// Gestion des événements (drawer)
// ============================================

function selectHour(hour: number, events: EventInHour[]) {
  selectedHour.value = hour
  selectedEvents.value = events
  drawerOpen.value = true
}

function getDistinctHighCount(events: EventInHour[]): number {
  if (!events) return 0
  // Compter les PAIRES (nom + HIGH) distinctes
  const distinctHigh = new Set(
    events.filter(e => e.impact === 'HIGH').map(e => `${e.event_name}|HIGH`)
  )
  return distinctHigh.size
}

function getDistinctMediumCount(events: EventInHour[]): number {
  if (!events) return 0
  // Compter les PAIRES (nom + MEDIUM) distinctes
  const distinctMedium = new Set(
    events.filter(e => e.impact === 'MEDIUM').map(e => `${e.event_name}|MEDIUM`)
  )
  return distinctMedium.size
}

function getEventIcon(events: EventInHour[]): string {
  const highCount = getDistinctHighCount(events)
  const mediumCount = getDistinctMediumCount(events)
  const total = highCount + mediumCount
  
  if (total === 0) return '○'
  
  // Calculer la hauteur des barres en pourcentage
  const maxCount = Math.max(highCount, mediumCount) || 1
  const highHeight = (highCount / maxCount) * 10
  const mediumHeight = (mediumCount / maxCount) * 10
  
  // Créer un histogramme SVG simple
  const svg = `<svg width="20" height="12" viewBox="0 0 20 12" style="display: inline-block; vertical-align: middle; margin-right: 4px;">
    <!-- Barre HIGH (rouge) -->
    <rect x="2" y="${12 - highHeight}" width="6" height="${highHeight}" fill="#ff6b6b" rx="1"/>
    <!-- Barre MEDIUM (orange) -->
    <rect x="12" y="${12 - mediumHeight}" width="6" height="${mediumHeight}" fill="#ffa94d" rx="1"/>
  </svg>`
  
  return svg
}


function getEventBadgeClass(events: EventInHour[]): string {
  const hasHigh = events.some(e => e.impact === 'HIGH')
  return hasHigh ? 'high' : 'medium'
}

function getDistinctEventCount(events: EventInHour[] | undefined): number {
  if (!events || events.length === 0) return 0
  // Compter les PAIRES (nom + impact) distinctes
  const distinctPairs = new Set(events.map(e => `${e.event_name}|${e.impact}`))
  return distinctPairs.size
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
  return props.stats15min
    .filter(stat => stat.hour === hour)
    .sort((a, b) => a.quarter - b.quarter)
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

.accordion-cell {
  padding: 0 !important;
}

.scalping-details {
  padding: 15px;
  background: #0d1117;
}

.scalping-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.85em;
  margin: 0;
}

.scalping-table thead {
  background: #1f6feb;
}

.scalping-table th {
  padding: 0.5rem;
  text-align: left;
  font-weight: bold;
  border: 1px solid #30363d;
  white-space: nowrap;
}

.scalping-table td {
  padding: 0.5rem;
  border: 1px solid #30363d;
}

.scalping-row:hover {
  background: #161b22;
}

.time-cell {
  font-weight: bold;
  color: #58a6ff;
}

.hourly-table {
  background: #161b22;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  border: 1px solid #30363d;
}

.table-header {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  margin-bottom: 2rem;
  gap: 2rem;
}

.header-left h3 {
  margin: 0;
  color: #e6edf3;
}

.table-container {
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
}

thead {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: white;
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

