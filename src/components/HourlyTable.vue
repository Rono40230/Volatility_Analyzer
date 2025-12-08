<template>
  <div
    v-if="stats.length > 0"
    class="hourly-table"
  >
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
            <th
              v-if="props.stats15min"
              style="width: 30px;"
            />
            <th>Heure</th>
            <th>ATR Moyen</th>
            <th>True Range</th>
            <th>Volatilite %</th>
            <th>Body Range %</th>
            <th>Direction Strength</th>
            <th>Noise Ratio</th>
            <th>Breakouts %</th>
            <th>Evenements</th>
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
                  v-if="stat.hour == bestSliceHour"
                  class="star"
                >⭐</span>
              </td>
              <td>{{ formatATR(stat.atr_mean) }}</td>
              <td>
                {{ formatATR(stat.range_mean) }}
              </td>
              <td>{{ (stat.volatility_mean * 100).toFixed(2) }}%</td>
              <td>
                {{ Math.abs(stat.body_range_mean).toFixed(2) }}%
                <span style="font-size: 0.8em; opacity: 0.7;">{{ stat.body_range_mean >= 0 ? '↗' : '↘' }}</span>
              </td>
              <td>{{ (stat.volume_imbalance_mean * 100).toFixed(2) }}%</td>
              <td>{{ stat.noise_ratio_mean.toFixed(2) }}%</td>
              <td>{{ stat.breakout_percentage.toFixed(2) }}%</td>
              <td class="events-cell">
                <button
                  v-if="getDistinctEventCount(stat.events) > 0"
                  class="event-badge-btn"
                  :class="getEventBadgeClass(stat.events)"
                  :title="`${getDistinctEventCount(stat.events)} événement(s) HIGH`"
                  @click="selectHour(stat.hour, stat.events)"
                >
                  {{ logHourEventCount(stat.hour, stat.events) }}
                </button>
                <span
                  v-else
                  class="no-event"
                >—</span>
              </td>
            </tr>

            <!-- Accordion 15-minutes -->
            <tr
              v-if="expandedHours.includes(stat.hour) && props.stats15min"
              class="accordion-row"
            >
              <td
                :colspan="props.stats15min ? 14 : 10"
                class="accordion-cell"
              >
                <div class="scalping-details">
                  <table class="scalping-table">
                    <thead>
                      <tr>
                        <th>Tranche</th>
                        <th>ATR Moyen</th>
                        <th>True Range</th>
                        <th>Volatilité %</th>
                        <th>Body Range %</th>
                        <th>Direction Strength</th>
                        <th>Noise Ratio</th>
                        <th>Breakouts %</th>
                        <th title="TÂCHE 4: Minutes volatilité > 80% pic">
                          Peak (min)
                        </th>
                        <th title="TÂCHE 4: Minutes pour -50% volatilité">
                          Half-Life (min)
                        </th>
                        <th title="TÂCHE 4: Durée optimale fermeture trade">
                          Trade Exp (min)
                        </th>
                        <th>Événements</th>
                        <th style="width: 140px;">Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr
                        v-for="quarter in getQuartersForHour(stat.hour)"
                        :key="`${stat.hour}-${quarter.quarter}`"
                        class="scalping-row"
                        :class="{ 'top3-slice': isBestSliceInHour(stat.hour, quarter.quarter) }"
                      >
                        <td class="time-cell">
                          <span
                            v-if="isBestSliceInHour(stat.hour, quarter.quarter)"
                            class="top3-star"
                          >⭐</span>
                          {{ formatQuarterLabel(stat.hour, quarter.quarter) }}
                        </td>
                        <td>{{ formatATR(quarter.atr_mean) }}</td>
                        <td>{{ formatATR(quarter.range_mean) }}</td>
                        <td>{{ (quarter.volatility_mean * 100).toFixed(2) }}%</td>
                        <td>
                          {{ Math.abs(quarter.body_range_mean).toFixed(2) }}%
                          <span style="font-size: 0.8em; opacity: 0.7;">{{ quarter.body_range_mean >= 0 ? '↗' : '↘' }}</span>
                        </td>
                        <td>{{ (quarter.volume_imbalance_mean * 100).toFixed(2) }}%</td>
                        <td>{{ quarter.noise_ratio_mean.toFixed(2) }}%</td>
                        <td>{{ quarter.breakout_percentage.toFixed(2) }}%</td>
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
                        <td class="events-cell">
                          <button
                            v-if="getEventsForQuarter(stat.events, stat.hour, quarter.quarter).length > 0"
                            class="event-badge-btn high"
                            style="font-size: 0.8em; padding: 2px 6px;"
                            @click="selectHour(stat.hour, getEventsForQuarter(stat.events, stat.hour, quarter.quarter))"
                          >
                            {{ getEventsForQuarter(stat.events, stat.hour, quarter.quarter).length }}
                          </button>
                          <span
                            v-else
                            class="no-event"
                          >—</span>
                        </td>
                        <td class="actions-cell">
                          <button
                            class="btn-bidi-params"
                            :title="`Ouvrir l'analyse pour ${formatQuarterLabel(stat.hour, quarter.quarter)}`"
                            @click="openBidiParams(stat.hour, quarter.quarter)"
                          >
                            ⚙️ Paramètres Bidi
                          </button>
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
      :is-open="drawerOpen"
      :selected-hour="selectedHour"
      :all-events="selectedEvents"
      @close="drawerOpen = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import type { HourlyStats, EventInHour, Stats15Min } from '../stores/volatility'
import EventDetailsDrawer from './EventDetailsDrawer.vue'

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
}>()

const emit = defineEmits<{
  'open-bidi-params': [{ hour: number; quarter: number }]
}>()

// Fonction helper pour estimer le prix moyen (pour normalisation)
function getEstimatedPrice(): number {
  if (!props.globalMetrics) {
    return 100000 // Valeur par défaut
  }
  // Utilise l'ATR moyen pour estimer l'ordre de grandeur du prix
  const atr = props.globalMetrics.mean_atr
  if (atr > 1000) return 100000 // Crypto (BTCUSD ~100k)
  if (atr > 10) return 10000    // Indices (SPX ~10k)
  return 1.0                     // Forex (EURUSD ~1.0)
}

// Formatte l'ATR en points (arrondir à l'unité supérieure) au lieu du pourcentage
function formatATR(atr: number): string {
  return `${Math.ceil(atr)} pts`
}

// Formate le label de quarter avec gestion du changement d'heure
function formatQuarterLabel(hour: number, quarter: number): string {
  const startMin = quarter * 15
  const endMin = startMin + 15
  
  if (endMin >= 60) {
    const endHour = (hour + 1) % 24
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(endHour).padStart(2, '0')}:00`
  } else {
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(hour).padStart(2, '0')}:${String(endMin).padStart(2, '0')}`
  }
}

// État du drawer
const drawerOpen = ref(false)
const selectedHour = ref<number | null>(null)
const selectedEvents = ref<EventInHour[] | null>(null)
const expandedHours = ref<number[]>([])
const top3Slices = ref<Array<{ hour: number; quarter: number }>>([])

// Calculer le TOP 3 au montage/changement des stats
onMounted(() => {
  if (props.stats.length > 0 && props.stats15min && props.stats15min.length > 0) {
    try {
      // Besoin de globalMetrics pour analyzeTop3Slices
      // On va créer une fonction locale pour identifier les TOP 3 slices
      const scoredSlices = props.stats15min.map((slice: Stats15Min): ScoredSlice => ({
        hour: slice.hour,
        quarter: slice.quarter,
        score: calculateSliceScore(slice)
      }))
      
      // Trier par score décroissant et prendre les 3 meilleurs
      top3Slices.value = scoredSlices
        .sort((a: ScoredSlice, b: ScoredSlice) => b.score - a.score)
        .slice(0, 3)
        .map((item: ScoredSlice) => ({ hour: item.hour, quarter: item.quarter }))
    } catch (err) {
      // Erreur calcul TOP 3 - ignorer silencieusement
    }
  }
})

function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}

function formatNumber(num: number, decimals: number): string {
  return num.toFixed(decimals)
}



const bestSliceHour = computed(() => {
  return props.bestQuarter[0] // L'heure qui contient le meilleur quarter
})

function isBestHour(hour: number): boolean {
  return props.bestQuarter[0] === hour
}

// ============================================
// Gestion des événements (drawer)
// ============================================

function selectHour(hour: number, events: EventInHour[]) {
  selectedHour.value = hour
  selectedEvents.value = events
  drawerOpen.value = true
}

function openBidiParams(hour: number, quarter: number) {
  emit('open-bidi-params', { hour, quarter })
}


function normalizeImpact(impact: string): string {
  const i = impact.toUpperCase().trim()
  if (i === 'HIGH' || i === 'H') return 'HIGH'
  if (i === 'MEDIUM' || i === 'M' || i === 'MED') return 'MEDIUM'
  if (i === 'LOW' || i === 'L') return 'LOW'
  return 'UNKNOWN'
}

// Log le compte d'événements pour l'heure 0
function logHourEventCount(hour: number, events: EventInHour[] | undefined): number {
  return getDistinctEventCount(events)
}




function getEventBadgeClass(events: EventInHour[]): string {
  const hasHigh = events.some(e => normalizeImpact(e.impact) === 'HIGH')
  return hasHigh ? 'high' : 'hidden-badge' // hidden-badge si pas de HIGH
}

function getDistinctEventCount(events: EventInHour[] | undefined): number {
  if (!events || events.length === 0) return 0
  
  // Filtrer pour ne garder que HIGH
  const highEvents = events.filter(e => normalizeImpact(e.impact) === 'HIGH')
  
  // Compter les PAIRES (nom + impact) distinctes
  const distinctPairs = new Set(highEvents.map(e => `${e.event_name}|HIGH`))
  return distinctPairs.size
}

// Compter le nombre total d'événements (pour afficher par quarter)
function getTotalEventCount(events: EventInHour[] | undefined): number {
  if (!events || events.length === 0) return 0
  // Retourner simplement le nombre total d'événements
  return events.length
}

// Extraire les événements HIGH DISTINCTS pour un quarter spécifique
// Même logique que getDistinctEventCount mais filtrée par quarter
function getEventsForQuarter(hourEvents: EventInHour[] | undefined, hour: number, quarter: number): EventInHour[] {
  if (!hourEvents || hourEvents.length === 0) return []
  
  // D'abord : récupérer TOUTES les paires HIGH distinctes de l'heure
  const allHighEvents = hourEvents.filter(e => normalizeImpact(e.impact) === 'HIGH')
  const allDistinctPairs = new Set(allHighEvents.map(e => `${e.event_name}|HIGH`))
  
  // Répartir par quarter basé sur le datetime (format "HH:MM:SS")
  const quarterStart = quarter * 15
  const quarterEnd = quarterStart + 15
  
  // Pour chaque paire DISTINCTE, vérifier s'il existe au moins un événement dans ce quarter
  const pairsInQuarter = new Set<string>()
  
  for (const pair of allDistinctPairs) {
    // Chercher un événement avec cette paire qui tombe dans ce quarter
    const eventInQuarter = allHighEvents.find(e => {
      if (`${e.event_name}|HIGH` !== pair) return false
      
      const timeParts = e.datetime.split(':')
      if (timeParts.length < 2) return false
      const minute = parseInt(timeParts[1], 10)
      return minute >= quarterStart && minute < quarterEnd
    })
    
    if (eventInQuarter) {
      pairsInQuarter.add(pair)
    }
  }
  
  // Retourner un tableau avec une seule instance de chaque paire qui est dans ce quarter
  const result: EventInHour[] = []
  for (const pair of pairsInQuarter) {
    const event = allHighEvents.find(e => `${e.event_name}|HIGH` === pair)
    if (event) {
      result.push(event)
    }
  }
  
  return result
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

function calculateSliceScore(slice: Stats15Min): number {
  // Même logique que straddleAnalysis.ts::calculateStraddleScore
  if (slice.candle_count === 0) return 0
  let score = 0

  // TRUE RANGE (60 pts max) - Includes gap detection
  if (slice.range_mean > 0.0025) {
    score += 60
  } else if (slice.range_mean > 0.0020) {
    score += 50
  } else if (slice.range_mean > 0.0015) {
    score += 40
  } else if (slice.range_mean > 0.0010) {
    score += 20
  }

  // ATR (25 pts max)
  if (slice.atr_mean > 0.0020) {
    score += 25
  } else if (slice.atr_mean > 0.0015) {
    score += 20
  } else if (slice.atr_mean > 0.0010) {
    score += 15
  } else if (slice.atr_mean > 0.0005) {
    score += 8
  }

  // BodyRange (15 pts max)
  if (slice.body_range_mean > 45.0) {
    score += 15
  } else if (slice.body_range_mean > 35.0) {
    score += 12
  } else if (slice.body_range_mean > 25.0) {
    score += 8
  } else if (slice.body_range_mean > 15.0) {
    score += 3
  }

  return Math.min(score, 100)
}

function isInTop3Slice(hour: number, quarter: number): boolean {
  return top3Slices.value.some(item => item.hour === hour && item.quarter === quarter)
}

// NEW: Retourner le MEILLEUR créneau 15min de chaque heure (pas les 3 globaux)
function isBestSliceInHour(hour: number, quarter: number): boolean {
  // Comparer directement avec le meilleur quarter global
  // props.bestQuarter est [hour, quarter]
  return props.bestQuarter[0] === hour && props.bestQuarter[1] === quarter
}

function getTop3SliceRank(hour: number, quarter: number): number {
  const found = top3Slices.value.findIndex(item => item.hour === hour && item.quarter === quarter)
  return found >= 0 ? found + 1 : 0
}

function hasTop3SlicesInHour(hour: number): boolean {
  return top3Slices.value.some(item => item.hour === hour)
}

// TÂCHE 4: Alerter si durée recommandée trop longue (>150 min = 2.5h)
function isTradeExpTooLong(slice: Stats15Min): boolean {
  return (slice.recommended_trade_expiration_minutes ?? 0) > 150
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
  background: #161b22;
  border-top: 2px solid #21262d;
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

/* Fond gris clair pour TOUTES les lignes 15min */
.scalping-row {
  background: #2d333b;
}

.scalping-row:hover {
  background: #353d48;
}

/* Fond plus foncé (doré) pour les lignes TOP 3 */
.scalping-row.top3-slice {
  background: rgba(251, 191, 36, 0.25);
  border-left: 4px solid #fbbf24;
}

.scalping-row.top3-slice:hover {
  background: rgba(251, 191, 36, 0.35);
}

.top3-star {
  color: #fbbf24;
  font-weight: bold;
  margin-right: 4px;
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

/* TÂCHE 4: Styles pour durées de volatilité */
.trade-exp-header {
  background: linear-gradient(135deg, rgba(249, 158, 11, 0.1), rgba(239, 68, 68, 0.1)) !important;
  font-weight: 600;
}

.duration-cell {
  text-align: center;
  color: #e6edf3;
  font-size: 0.85em;
}

.trade-exp-cell {
  text-align: center;
  color: #e6edf3;
  font-size: 0.85em;
}

.trade-exp-cell.warning {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.trade-exp-cell.warning .warning-icon {
  margin-left: 4px;
  font-size: 0.9em;
}

.scalping-row:hover .duration-cell {
  color: #e6edf3;
}

.scalping-row:hover .trade-exp-cell {
  color: #e6edf3;
}

.actions-cell {
  text-align: center;
  padding: 8px !important;
}

.btn-bidi-params {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid #58a6ff;
  background: rgba(88, 166, 255, 0.1);
  color: #58a6ff;
  font-size: 0.85em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-bidi-params:hover {
  background: rgba(88, 166, 255, 0.2);
  border-color: #79c0ff;
  color: #79c0ff;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(88, 166, 255, 0.2);
}

.btn-bidi-params:active {
  transform: translateY(0);
}
</style>

