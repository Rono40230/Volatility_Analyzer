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
}

interface ColorGroup {
  level: 'OPTIMAL' | 'BON' | 'RISQUÉ'
  label: string
  emoji: string
  bgColor: string
  borderColor: string
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
const groupStates = ref<Record<string, boolean>>({
  OPTIMAL: true,
  BON: true,
  RISQUÉ: true
})

// Grouper les événements par couleur
const groupedEvents = computed<ColorGroup[]>(() => {
  const optimal: EventDisplay[] = []
  const bon: EventDisplay[] = []
  const risque: EventDisplay[] = []

  for (const event of props.sortedEvents) {
    if (event.tradability === 'OPTIMAL') optimal.push(event)
    else if (event.tradability === 'BON') bon.push(event)
    else risque.push(event)
  }

  return [
    {
      level: 'OPTIMAL',
      label: 'Excellents (Vert)',
      emoji: '🟢',
      bgColor: 'rgba(16, 185, 129, 0.1)',
      borderColor: '#10b981',
      events: optimal,
      isOpen: groupStates.value.OPTIMAL ?? true
    },
    {
      level: 'BON',
      label: 'Bons (Orange)',
      emoji: '🟡',
      bgColor: 'rgba(251, 191, 36, 0.1)',
      borderColor: '#fbbf24',
      events: bon,
      isOpen: groupStates.value.BON ?? true
    },
    {
      level: 'RISQUÉ',
      label: 'Risqués (Rouge)',
      emoji: '🔴',
      bgColor: 'rgba(239, 68, 68, 0.1)',
      borderColor: '#ef4444',
      events: risque,
      isOpen: groupStates.value.RISQUÉ ?? true
    }
  ].filter(g => g.events.length > 0)
})

function toggleGroup(level: string) {
  groupStates.value[level] = !groupStates.value[level]
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
    <!-- Color Groups (Accordion) -->
    <div v-for="group in groupedEvents" :key="group.level" class="color-group">
      <!-- Group Header (Accordion Toggle) -->
      <div 
        class="group-header" 
        :style="{ backgroundColor: group.bgColor, borderLeftColor: group.borderColor }"
        @click="toggleGroup(group.level)"
      >
        <div class="group-title">
          <span class="group-emoji">{{ group.emoji }}</span>
          <span class="group-label">{{ group.label }}</span>
          <span class="group-count">{{ group.events.length }}</span>
        </div>
        <div class="group-toggle">
          <span class="toggle-icon" :class="{ open: groupStates[group.level] }">▶</span>
        </div>
      </div>

      <!-- Group Content (Cards) -->
      <div v-if="groupStates[group.level]" class="group-content">
        <div v-for="event in group.events" :key="event.eventType" class="event-card">
          <!-- Event Header -->
          <div class="event-header">
            <div class="event-title-group">
              <span class="event-icon">{{ event.icon }}</span>
              <div>
                <h4 class="event-name">{{ translateEventName(event.eventType) }}</h4>
                <span class="tradability-badge" :data-level="event.tradability.toLowerCase()">
                  {{ event.tradability }}
                </span>
              </div>
            </div>
            <div class="event-score">
              <div class="score-value">{{ Math.round(event.stats.tradabilityScore || 0) }}/100</div>
              <div class="score-label">Score</div>
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
