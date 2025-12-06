<script setup lang="ts">
import { computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'
import { useEventPairCorrelation } from '../../composables/useEventPairCorrelation'
import { useEventDetail } from '../../composables/useEventDetail'
import { useEventTranslation } from '../../composables/useEventTranslation'
import EventDetailModal from './EventDetailModal.vue'

const { eventStatistics } = useArchiveStatistics()
const { getPairsByEvent, hasHeatmapData } = useEventPairCorrelation()
const { isOpen, selectedEvent, openDetail, closeDetail } = useEventDetail()
const { translateEventName } = useEventTranslation()

interface EventDisplay {
  eventType: string
  stats: ReturnType<typeof useArchiveStatistics>['eventStatistics']['value'][string]
  tradability: 'OPTIMAL' | 'BON' | 'RISQUÉ'
  colorClass: string
  icon: string
}

const sortedEvents = computed<EventDisplay[]>(() => {
  if (!eventStatistics.value) return []

  return Object.entries(eventStatistics.value)
    .map(([eventType, stats]) => {
      const score = stats.tradabilityScore || 0
      let tradability: 'OPTIMAL' | 'BON' | 'RISQUÉ' = 'RISQUÉ'
      let colorClass = 'text-red-500'
      let icon = '🔴'

      if (score >= 80) {
        tradability = 'OPTIMAL'
        colorClass = 'text-green-500'
        icon = '🟢'
      } else if (score >= 60) {
        tradability = 'BON'
        colorClass = 'text-yellow-500'
        icon = '🟡'
      }

      return {
        eventType,
        stats,
        tradability,
        colorClass,
        icon,
      }
    })
    .sort((a, b) => (b.stats.tradabilityScore || 0) - (a.stats.tradabilityScore || 0))
})

const totalEvents = computed(() => sortedEvents.value.length)
const optimalCount = computed(() => sortedEvents.value.filter(e => e.tradability === 'OPTIMAL').length)
const avgConfidence = computed(() => {
  if (sortedEvents.value.length === 0) return 0
  const sum = sortedEvents.value.reduce((acc, e) => acc + e.stats.avgConfidence, 0)
  return Math.round((sum / sortedEvents.value.length) * 100) / 100
})

/**
 * Récupère les paires pour un événement avec couleur d'impact
 */
function getImpactColor(impact: number): string {
  if (impact >= 80) return '#10b981' // Vert - Excellent
  if (impact >= 60) return '#fbbf24' // Ambre - Bon
  if (impact >= 40) return '#f97316' // Orange - Moyen
  return '#ef4444' // Rouge - Faible
}

function getImpactIcon(impact: number): string {
  if (impact >= 80) return '🟢'
  if (impact >= 60) return '🟡'
  if (impact >= 40) return '🟠'
  return '🔴'
}

function openEventDetail(event: any) {
  openDetail({
    eventType: event.eventType,
    score: event.stats.tradabilityScore,
    avgATR: event.stats.avgATR,
    avgPeakDelay: event.stats.avgPeakDelay,
    avgConfidence: event.stats.avgConfidence,
    tradability: event.tradability,
  })
}
</script>

<template>
  <div class="event-analysis-block">
    <!-- Header -->
    <div class="header-section">
      <div class="header-content">
      </div>
    </div>

    <!-- Events List -->
    <div v-if="sortedEvents.length > 0" class="events-list">
      <div v-for="event in sortedEvents" :key="event.eventType" class="event-card">
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

        <!-- Straddle Setup -->
        <div class="straddle-setup">
          <div class="setup-label">Straddle Setup</div>
          <div class="setup-params">
            <span>SL: <strong>{{ Math.round(event.stats.avgATR * 1.5) }}p</strong></span>
            <span class="separator">•</span>
            <span>TP: <strong>{{ Math.round(event.stats.avgATR * 3) }}p</strong></span>
            <span class="separator">•</span>
            <span>Ratio: <strong>1:2</strong></span>
            <span class="separator">•</span>
            <span>Placement: <strong>{{ Math.round(event.stats.avgPeakDelay * 60) }}sec avant</strong></span>
          </div>
        </div>

        <!-- Best Pairs (if Heatmap data available) -->
        <div v-if="hasHeatmapData" class="best-pairs-section">
          <div class="pairs-label">Meilleures Paires</div>
          <div class="pairs-list">
            <div v-for="(pair, idx) in getPairsByEvent(event.eventType).slice(0, 3)" :key="pair.pair" class="pair-badge" :style="{ borderLeftColor: getImpactColor(pair.impact) }">
              <span class="pair-icon">{{ getImpactIcon(pair.impact) }}</span>
              <span class="pair-name">{{ pair.pair }}</span>
              <span class="pair-impact">{{ Math.round(pair.impact) }}%</span>
            </div>
            <div v-if="getPairsByEvent(event.eventType).length === 0" class="no-pairs">
              Aucune paire détectée
            </div>
          </div>
          <button class="detail-button" @click="openEventDetail(event)">
            → Voir tous les détails
          </button>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <p>Aucun événement analysé</p>
    </div>

    <!-- Event Detail Modal -->
    <EventDetailModal :is-open="isOpen" :event="selectedEvent" @close="closeDetail" />
  </div>
</template>

<style scoped>
.event-analysis-block {
  animation: slideIn 0.3s ease-out;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  border-bottom: 1px solid rgba(59, 130, 246, 0.3);
  padding-bottom: 16px;
  margin-bottom: 20px;
}

.header-content h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
}

.header-subtitle {
  margin: 6px 0 0 0;
  font-size: 12px;
  color: #a0aec0;
}

.header-icon {
  font-size: 32px;
  opacity: 0.7;
}

.events-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(480px, 1fr));
  gap: 12px;
}

.event-card {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 10px;
  padding: 12px;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.event-card:hover {
  background: rgba(0, 0, 0, 0.5);
  border-color: rgba(255, 255, 255, 0.1);
}

.event-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.event-title-group {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.event-icon {
  font-size: 20px;
  margin-top: 2px;
}

.event-name {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
}

.tradability-badge {
  display: inline-block;
  margin-top: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 600;
}

.tradability-badge[data-level="optimal"] {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.tradability-badge[data-level="bon"] {
  background: rgba(251, 146, 60, 0.2);
  color: #fb923c;
}

.tradability-badge[data-level="risqué"] {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.event-score {
  text-align: right;
}

.score-value {
  font-size: 20px;
  font-weight: 700;
  color: #ffffff;
}

.score-label {
  font-size: 11px;
  color: #a0aec0;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 8px;
  margin-bottom: 12px;
}

.metric-box {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 6px;
  padding: 8px;
}

.metric-label {
  font-size: 11px;
  color: #a0aec0;
  margin-bottom: 4px;
}

.metric-value {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.straddle-setup {
  background: linear-gradient(135deg, rgba(147, 51, 234, 0.15), rgba(79, 70, 229, 0.15));
  border: 1px solid rgba(147, 51, 234, 0.3);
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 12px;
}

.setup-label {
  font-size: 11px;
  font-weight: 600;
  color: #a855f7;
  text-transform: uppercase;
  margin-bottom: 8px;
}

.setup-params {
  font-size: 12px;
  color: #e2e8f0;
  line-height: 1.6;
}

.separator {
  color: #64748b;
  margin: 0 4px;
}

.best-pairs-section {
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.15), rgba(37, 99, 235, 0.15));
  border: 1px solid rgba(59, 130, 246, 0.3);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.pairs-label {
  font-size: 11px;
  font-weight: 600;
  color: #3b82f6;
  text-transform: uppercase;
  margin-bottom: 8px;
}

.pairs-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pair-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(0, 0, 0, 0.3);
  border-left: 3px solid #3b82f6;
  border-radius: 4px;
  padding: 6px 10px;
  font-size: 12px;
}

.pair-icon {
  font-size: 14px;
}

.pair-name {
  font-weight: 600;
  color: #ffffff;
  flex: 1;
}

.pair-impact {
  font-size: 11px;
  color: #a0aec0;
  font-weight: 600;
}

.no-pairs {
  font-size: 11px;
  color: #a0aec0;
  font-style: italic;
  padding: 6px 10px;
}

.detail-button {
  align-self: flex-start;
  background: rgba(59, 130, 246, 0.2);
  border: 1px solid rgba(59, 130, 246, 0.4);
  color: #3b82f6;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.detail-button:hover {
  background: rgba(59, 130, 246, 0.3);
  border-color: rgba(59, 130, 246, 0.6);
  transform: translateX(2px);
}

.separator {
  color: #64748b;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
