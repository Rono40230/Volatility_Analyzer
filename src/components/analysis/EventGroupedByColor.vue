<script setup lang="ts">
import { ref, computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'
import { useEventPairCorrelation } from '../../composables/useEventPairCorrelation'
import { useEventDetail } from '../../composables/useEventDetail'
import { useEventTranslation } from '../../composables/useEventTranslation'

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
}

const props = defineProps<{
  sortedEvents: EventDisplay[]
}>()

const { eventPairStatistics } = useArchiveStatistics()
const { getPairsByEvent, hasHeatmapData } = useEventPairCorrelation()
const { openDetail } = useEventDetail()
const { translateEventName } = useEventTranslation()

// État des accordéons (tous ouverts par défaut)
const groupStates = ref<Record<number, boolean>>({})

// Compter les paires vertes par événement
function countGreenPairs(eventType: string): number {
  const pairs = getPairsByEvent(eventType)
  return pairs.filter(p => p.impact >= 80).length
}

// Enrichir les événements avec le compte de paires vertes, puis trier
const eventsWithGreenCount = computed(() => {
  return props.sortedEvents.map(event => ({
    ...event,
    greenPairCount: countGreenPairs(event.eventType)
  })).sort((a, b) => b.greenPairCount - a.greenPairCount)
})

// Grouper par nombre de paires vertes (en ordre descendant)
const groupedByGreen = computed<GreenGroup[]>(() => {
  const groupMap = new Map<number, EventDisplay[]>()
  
  for (const event of eventsWithGreenCount.value) {
    const key = event.greenPairCount
    if (!groupMap.has(key)) {
      groupMap.set(key, [])
    }
    groupMap.get(key)!.push(event)
  }

  // Tri par greenCount descendant
  const sortedKeys = Array.from(groupMap.keys()).sort((a, b) => b - a)
  
  return sortedKeys.map(greenCount => ({
    greenCount,
    label: greenCount === 0 ? '❌ Aucune paire verte' : `✅ ${greenCount} paire${greenCount > 1 ? 's' : ''} verte${greenCount > 1 ? 's' : ''}`,
    events: groupMap.get(greenCount)!,
    isOpen: groupStates.value[greenCount] ?? true
  }))
})

function toggleGroup(greenCount: number) {
  groupStates.value[greenCount] = !groupStates.value[greenCount]
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
  
  openDetail({
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
    <div v-for="group in groupedByGreen" :key="group.greenCount" class="color-group">
      <!-- Group Header (Accordion Toggle) -->
      <div 
        class="group-header"
        :style="{ backgroundColor: getGroupBgColor(group.greenCount), borderLeftColor: getGroupBorderColor(group.greenCount) }"
        @click="toggleGroup(group.greenCount)"
      >
        <div class="group-title">
          <span class="group-label">{{ group.label }}</span>
          <span class="group-count">{{ group.events.length }} carte{{ group.events.length > 1 ? 's' : '' }}</span>
        </div>
        <div class="group-toggle">
          <span class="toggle-icon" :class="{ open: groupStates[group.greenCount] }">▶</span>
        </div>
      </div>

      <!-- Group Content (Cards) -->
      <div v-if="groupStates[group.greenCount]" class="group-content">
        <div v-for="event in group.events" :key="event.eventType" class="event-card">
          <!-- Event Header -->
          <div class="event-header">
            <div class="event-title-group">
              <span class="event-icon">{{ event.icon }}</span>
              <div>
                <h4 class="event-name">{{ translateEventName(event.eventType) }}</h4>
                <span class="green-pairs-badge">✅ {{ event.greenPairCount }} paires vertes</span>
              </div>
            </div>
          </div>

          <!-- Metrics Grid -->
          <div class="metrics-grid">
            <div class="metric-box">
              <div class="metric-label">Volatilité ATR</div>
              <div class="metric-value">{{ Math.round(event.stats.avgATR * 10) / 10 }}p</div>
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
              <div class="metric-label">Analyses</div>
              <div class="metric-value">{{ event.stats.count }}</div>
            </div>
          </div>

          <!-- Best Pairs -->
          <div v-if="hasHeatmapData" class="best-pairs-section">
            <div class="pairs-label">Meilleures Paires</div>
            <div class="pairs-list">
              <div v-for="pair in getPairsByEvent(event.eventType).slice(0, 3)" :key="pair.pair" class="pair-badge" :style="{ borderLeftColor: getImpactColor(pair.impact) }">
                <span class="pair-icon">{{ getImpactIcon(pair.impact) }}</span>
                <span class="pair-name">{{ pair.pair }}</span>
                <span class="pair-impact">{{ Math.round(pair.impact) }}%</span>
              </div>
              <div v-if="getPairsByEvent(event.eventType).length === 0" class="no-pairs">
                Aucune paire détectée
              </div>
            </div>
            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
              <button v-for="pair in getPairsByEvent(event.eventType).slice(0, 3)" :key="pair.pair" class="detail-button" @click="openEventDetail(event.eventType, pair.pair)">
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
