<script setup lang="ts">
import { ref, computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'
import { useEventPairCorrelation } from '../../composables/useEventPairCorrelation'
import { useEventTranslation } from '../../composables/useEventTranslation'
import type { EventDetailState } from '../../composables/useEventDetail'

interface EventDisplay {
  eventType: string
  stats: ReturnType<typeof useArchiveStatistics>['eventStatistics']['value'][string]
  tradability: 'OPTIMAL' | 'BON' | 'RISQUÉ'
  colorClass: string
  icon: string
  greenPairCount: number
}

interface GreenGroup {
  greenCount: number
  label: string
  events: EventDisplay[]
  isOpen: boolean
  category: 'vert' | 'orange' | 'rouge'
}

const props = defineProps<{
  sortedEvents: EventDisplay[]
  openDetail: (event: EventDetailState) => void
}>()

const { eventPairStatistics } = useArchiveStatistics()
const { getPairsByEvent, hasHeatmapData } = useEventPairCorrelation()
const { translateEventName } = useEventTranslation()

// État des accordéons (tous ouverts par défaut)
const groupStates = ref<Record<number, boolean>>({})

// Wrapper fonction pour getPairsByEvent (c'est un computed retournant une fonction)
function getPairsByEventFunc(eventType: string) {
  return getPairsByEvent.value(eventType)
}

// Compter les paires vertes par événement
function countGreenPairs(eventType: string): number {
  const pairs = getPairsByEventFunc(eventType)
  return pairs.filter(p => p.impact >= 80).length
}

// Déterminer la catégorie (meilleur impact trouvé)
function getCategory(eventType: string): 'vert' | 'orange' | 'rouge' {
  const pairs = getPairsByEventFunc(eventType)
  if (pairs.some(p => p.impact >= 80)) return 'vert'
  if (pairs.some(p => p.impact >= 60)) return 'orange'
  return 'rouge'
}

// Enrichir les événements avec le compte de paires vertes, puis trier
const eventsWithGreenCount = computed(() => {
  return props.sortedEvents.map(event => ({
    ...event,
    greenPairCount: countGreenPairs(event.eventType)
  })).sort((a, b) => b.greenPairCount - a.greenPairCount)
})

// Grouper par catégorie (vert > orange > rouge)
const groupedByGreen = computed<GreenGroup[]>(() => {
  const groups: GreenGroup[] = [
    { greenCount: 0, label: '🟢 Paires Vertes', category: 'vert', events: [], isOpen: groupStates.value['vert'] ?? true },
    { greenCount: 1, label: '🟡 Paires Oranges', category: 'orange', events: [], isOpen: groupStates.value['orange'] ?? true },
    { greenCount: 2, label: '🔴 Paires Rouges', category: 'rouge', events: [], isOpen: groupStates.value['rouge'] ?? true },
  ]
  
  for (const event of eventsWithGreenCount.value) {
    const category = getCategory(event.eventType)
    const group = groups.find(g => g.category === category)
    if (group) {
      group.events.push(event)
    }
  }
  
  // Retourner seulement les groupes avec des événements
  return groups.filter(g => g.events.length > 0)
})

function toggleGroup(category: string) {
  groupStates.value[category] = !groupStates.value[category]
}

function getGroupBgColor(greenCount: number): string {
  if (greenCount >= 2) return 'rgba(16, 185, 129, 0.1)' // Vert
  if (greenCount === 1) return 'rgba(251, 191, 36, 0.1)' // Orange
  return 'rgba(239, 68, 68, 0.1)' // Rouge
}

function getGroupBorderColor(greenCount: number): string {
  if (greenCount >= 2) return '#10b981' // Vert
  if (greenCount === 1) return '#fbbf24' // Orange
  return '#ef4444' // Rouge
}

function getImpactColor(impact: number): string {
  if (impact >= 80) return '#10b981'
  if (impact >= 60) return '#fbbf24'
  if (impact >= 40) return '#f97316'
  return '#ef4444'
}

function getImpactIcon(impact: number): string {
  if (impact >= 80) return '🟢'
  if (impact >= 60) return '🟡'
  if (impact >= 40) return '🟠'
  return '🔴'
}

function openEventDetail(eventType: string, pair: string) {
  const eventStats = useArchiveStatistics().eventStatistics.value?.[eventType]
  const key = `${eventType}|${pair}`
  const pairStats = eventPairStatistics.value?.[key]
  
  if (!eventStats || !pairStats) return
  
  props.openDetail({
    eventType,
    score: eventStats.tradabilityScore,
    avgATR: eventStats.avgATR,
    avgPeakDelay: eventStats.avgPeakDelay,
    avgConfidence: eventStats.avgConfidence,
    tradability: eventStats.tradabilityScore ? 
      (eventStats.tradabilityScore >= 80 ? 'OPTIMAL' : eventStats.tradabilityScore >= 60 ? 'BON' : 'RISQUÉ') : 'RISQUÉ',
    pair,
    slAdjusted: pairStats.slAdjusted,
    trailingStopCoefficient: pairStats.trailingStopCoefficient,
  })
}
</script>

<template>
  <div class="event-grouped-container">
    <!-- Green Count Groups (Accordion) -->
    <div v-for="group in groupedByGreen" :key="group.category" class="color-group">
      <!-- Group Header (Accordion Toggle) -->
      <div 
        class="group-header"
        :style="{ backgroundColor: getGroupBgColor(group.category === 'vert' ? 2 : group.category === 'orange' ? 1 : 0), borderLeftColor: getGroupBorderColor(group.category === 'vert' ? 2 : group.category === 'orange' ? 1 : 0) }"
        @click="toggleGroup(group.category)"
      >
        <div class="group-title">
          <span class="group-label">{{ group.label }}</span>
          <span class="group-count">{{ group.events.length }} carte{{ group.events.length > 1 ? 's' : '' }}</span>
        </div>
        <div class="group-toggle">
          <span class="toggle-icon" :class="{ open: groupStates[group.category] }">▶</span>
        </div>
      </div>

      <!-- Group Content (Cards) -->
      <div v-if="groupStates[group.category]" class="group-content">
        <div v-for="event in group.events" :key="event.eventType" class="event-card">
          <!-- Event Header -->
          <div class="event-header">
            <div class="event-title-group">
              <div>
                <h4 class="event-name">{{ translateEventName(event.eventType) }}</h4>
              </div>
            </div>
          </div>

          <!-- Metrics Grid -->
          <div class="metrics-grid">
            <div class="metric-box">
              <div class="metric-label">Volatilité ATR</div>
              <div class="metric-value">{{ event.stats.avgATR.toFixed(1) }} {{ event.stats.unit || 'pips' }}</div>
            </div>
            <div class="metric-box">
              <div class="metric-label">Pic (+/-)</div>
              <div class="metric-value">+{{ Math.round(event.stats.avgPeakDelay * 10) / 10 }}min</div>
            </div>
            <div class="metric-box">
              <div class="metric-label">Confiance</div>
              <div class="metric-value">{{ Math.round(event.stats.avgConfidence) }}%</div>
            </div>
            <div class="metric-box">
              <div class="metric-label">Occurrences</div>
              <div class="metric-value">{{ event.stats.count }}</div>
            </div>
          </div>

          <!-- Best Pairs -->
          <div v-if="hasHeatmapData" class="best-pairs-section">
            <div class="pairs-label">Meilleures Paires</div>
            <div class="pairs-list">
              <div v-for="pair in getPairsByEventFunc(event.eventType).slice(0, 3)" :key="pair.pair" class="pair-badge" :style="{ borderLeftColor: getImpactColor(pair.impact) }">
                <span class="pair-icon">{{ getImpactIcon(pair.impact) }}</span>
                <span class="pair-name">{{ pair.pair }}</span>
                <span class="pair-impact">{{ Math.round(pair.impact) }}%</span>
              </div>
              <div v-if="getPairsByEventFunc(event.eventType).length === 0" class="no-pairs">
                Aucune paire détectée
              </div>
            </div>
            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
              <button v-for="pair in getPairsByEventFunc(event.eventType).slice(0, 3)" :key="pair.pair" class="detail-button" @click="openEventDetail(event.eventType, pair.pair)">
                {{ pair.pair }} →
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@import './EventGroupedByColor.css';
</style>
