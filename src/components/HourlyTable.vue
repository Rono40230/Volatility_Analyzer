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
            <th>True Range</th>
            <th>Volatilite %</th>
            <th>Body Range %</th>
            <th>Direction Strength</th>
            <th>Tick Quality</th>
            <th>Noise Ratio</th>
            <th>Breakouts %</th>
            <th>Evenements</th>
          </tr>
        </thead>
        <tbody>
          <template v-for="stat in stats" :key="stat.hour">
            <!-- Ligne horaire normale -->
            <tr>
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
                <span v-if="stat.hour == bestSliceHour" class="star">⭐</span>
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
              <td>{{ formatTickQuality(stat.tick_quality_mean) }}</td>
              <td>{{ stat.noise_ratio_mean.toFixed(2) }}</td>
              <td>{{ stat.breakout_percentage.toFixed(2) }}%</td>
              <td class="events-cell">
                <button
                  v-if="getDistinctEventCount(stat.events) > 0"
                  class="event-badge-btn"
                  :class="getEventBadgeClass(stat.events)"
                  @click="selectHour(stat.hour, stat.events)"
                  :title="`${getDistinctEventCount(stat.events)} événement(s) HIGH`"
                >
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
                        <th>Evenements</th>
                      </tr>
                    </thead>
                    <tbody>
                      <tr v-for="quarter in getQuartersForHour(stat.hour)" :key="`${stat.hour}-${quarter.quarter}`" class="scalping-row" :class="{ 'top3-slice': isInTop3Slice(stat.hour, quarter.quarter) }">
                        <td class="time-cell">
                          <span v-if="isInTop3Slice(stat.hour, quarter.quarter)" class="top3-star">⭐ {{ getTop3SliceRank(stat.hour, quarter.quarter) }}</span>
                          {{ String(stat.hour).padStart(2, '0') }}:{{ String(quarter.quarter * 15).padStart(2, '0') }}-{{ String(stat.hour).padStart(2, '0') }}:{{ String(Math.min(quarter.quarter * 15 + 15, 60)).padStart(2, '0') }}
                        </td>
                        <td>{{ formatATR(quarter.atr_mean) }}</td>
                        <td>{{ formatATR(quarter.range_mean) }}</td>
                        <td>{{ (quarter.volatility_mean * 100).toFixed(2) }}%</td>
                        <td>
                          {{ Math.abs(quarter.body_range_mean).toFixed(2) }}%
                          <span style="font-size: 0.8em; opacity: 0.7;">{{ quarter.body_range_mean >= 0 ? '↗' : '↘' }}</span>
                        </td>
                        <td>{{ (quarter.volume_imbalance_mean * 100).toFixed(2) }}%</td>
                        <td>{{ formatTickQuality(quarter.tick_quality_mean) }}</td>
                        <td>{{ quarter.noise_ratio_mean.toFixed(2) }}</td>
                        <td>{{ quarter.breakout_percentage.toFixed(2) }}%</td>
                        <td class="events-cell">
                          <button
                            v-if="getDistinctEventCount(quarter.events) > 0"
                            class="event-badge-btn"
                            :class="getEventBadgeClass(quarter.events)"
                            @click="selectHour(stat.hour, quarter.events)"
                            style="font-size: 0.8em; padding: 2px 6px;"
                          >
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
import { ref, onMounted, computed } from 'vue'
import type { HourlyStats, EventInHour } from '../stores/volatility'
import EventDetailsDrawer from './EventDetailsDrawer.vue'

const props = defineProps<{
  stats: HourlyStats[]
  bestHours: number[]
  stats15min?: any[]  // Stats 15-minutes optionnels
  globalMetrics?: any // Pour normalisation (ATR, Tick Quality)
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

// Formatte l'ATR en % du prix
function formatATR(atr: number): string {
  const price = getEstimatedPrice()
  const atrPercent = (atr / price) * 100
  return `${atrPercent.toFixed(2)}%`
}

// Formatte le Tick Quality en % du prix
function formatTickQuality(tick: number): string {
  const price = getEstimatedPrice()
  const tickPercent = (tick / price) * 100
  return `${tickPercent.toFixed(2)}%`
}

// État du drawer
const drawerOpen = ref(false)
const selectedHour = ref<number | null>(null)
const selectedEvents = ref<EventInHour[] | null>(null)
const expandedHours = ref<number[]>([])
const top3Slices = ref<any[]>([])

// Calculer le TOP 3 au montage/changement des stats
onMounted(() => {
  console.log('📊 HourlyTable mounted. Stats received:', props.stats.length)
  
  if (props.stats.length > 0) {
    // Inspecter la première heure et quelques autres pour voir la structure events
    const sampleHours = [0, 8, 12, 16]
    sampleHours.forEach(h => {
      const stat = props.stats.find(s => s.hour === h)
      if (stat) {
        console.log(`🔍 Hour ${h} events:`, stat.events)
        if (stat.events) {
           console.log(`   - Count: ${stat.events.length}`)
           if (stat.events.length > 0) {
             console.log(`   - First event:`, stat.events[0])
           }
        } else {
           console.log(`   - Events field is UNDEFINED or NULL`)
        }
      }
    })
    
    // Compter le nombre total d'heures avec événements
    const hoursWithEvents = props.stats.filter(s => s.events && s.events.length > 0).length
    console.log(`📈 Total hours with events: ${hoursWithEvents} / ${props.stats.length}`)
  }
  if (props.stats15min && props.stats15min.length > 0) {
    try {
      // Besoin de globalMetrics pour analyzeTop3Slices
      // On va créer une fonction locale pour identifier les TOP 3 slices
      const scoredSlices = props.stats15min.map((slice: any) => ({
        hour: slice.hour,
        quarter: slice.quarter,
        score: calculateSliceScore(slice)
      }))
      
      // Trier par score décroissant et prendre les 3 meilleurs
      top3Slices.value = scoredSlices
        .sort((a: any, b: any) => b.score - a.score)
        .slice(0, 3)
        .map((item: any) => ({ hour: item.hour, quarter: item.quarter }))
    } catch (err) {
      console.error('Erreur calcul TOP 3:', err)
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
  if (top3Slices.value && top3Slices.value.length > 0) {
    return top3Slices.value[0].hour
  }
  return null
})

function isBestHour(hour: number): boolean {
  return props.bestHours.includes(hour)
}

// ============================================
// Gestion des événements (drawer)
// ============================================

function selectHour(hour: number, events: EventInHour[]) {
  selectedHour.value = hour
  selectedEvents.value = events
  drawerOpen.value = true
}



function normalizeImpact(impact: string): string {
  const i = impact.toUpperCase().trim()
  if (i === 'HIGH' || i === 'H') return 'HIGH'
  if (i === 'MEDIUM' || i === 'M' || i === 'MED') return 'MEDIUM'
  if (i === 'LOW' || i === 'L') return 'LOW'
  return 'UNKNOWN'
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
  // Retourner TOUS les quarters (0, 1, 2, 3) triés par ordre horaire
  return props.stats15min
    .filter(stat => stat.hour === hour)
    .sort((a, b) => a.quarter - b.quarter)
}

function calculateSliceScore(slice: any): number {
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

function getTop3SliceRank(hour: number, quarter: number): number {
  const found = top3Slices.value.findIndex(item => item.hour === hour && item.quarter === quarter)
  return found >= 0 ? found + 1 : 0
}

function hasTop3SlicesInHour(hour: number): boolean {
  return top3Slices.value.some(item => item.hour === hour)
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
</style>

