<script setup lang="ts">
import { computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'

const { eventStatistics } = useArchiveStatistics()

interface TimingEvent {
  eventType: string
  peakDelay: number
  placementSeconds: number
  exitMinutes: number
  estimatedGain: number
  confidence: number
  tradabilityScore: number
}

const sortedByTiming = computed<TimingEvent[]>(() => {
  if (!eventStatistics.value) return []

  return Object.entries(eventStatistics.value)
    .map(([eventType, stats]) => ({
      eventType,
      peakDelay: stats.avgPeakDelay,
      placementSeconds: Math.round(stats.avgPeakDelay * 60),
      exitMinutes: Math.round(stats.avgDecayTimeout * 1.5),
      estimatedGain: Math.round(stats.avgATR * 2.5),
      confidence: stats.confidence,
      tradabilityScore: stats.tradabilityScore,
    }))
    .sort((a, b) => a.peakDelay - b.peakDelay)
})

const fastestEvent = computed(() => sortedByTiming.value[0] || null)
const slowestEvent = computed(() => {
  if (sortedByTiming.value.length === 0) return null
  return sortedByTiming.value[sortedByTiming.value.length - 1]
})
const avgPlacement = computed(() => {
  if (sortedByTiming.value.length === 0) return 0
  const sum = sortedByTiming.value.reduce((acc, e) => acc + e.placementSeconds, 0)
  return Math.round(sum / sortedByTiming.value.length)
})
</script>

<template>
  <div class="timing-analysis-block">
    <!-- Header -->
    <div class="header-section">
      <div class="header-content">
      </div>
    </div>

    <!-- Fastest & Slowest Events Row -->
    <div class="fast-slow-row">
      <!-- Fastest Event -->
      <div v-if="fastestEvent" class="fastest-event">
        <div class="fastest-label">⚡ Réaction la plus rapide</div>
        <div class="fastest-content">
          <h4 class="fastest-title">{{ fastestEvent.eventType }}</h4>
          <div class="fastest-value">
            <div class="fastest-sublabel">Pic après placement</div>
            <div class="fastest-number">+{{ fastestEvent.peakDelay.toFixed(1) }}min</div>
          </div>
        </div>
      </div>

      <!-- Slowest Event -->
      <div v-if="slowestEvent" class="slowest-event">
        <div class="slowest-label">🐢 Réaction la plus lente</div>
        <div class="slowest-content">
          <h4 class="slowest-title">{{ slowestEvent.eventType }}</h4>
          <div class="slowest-value">
            <div class="slowest-sublabel">Pic après placement</div>
            <div class="slowest-number">+{{ slowestEvent.peakDelay.toFixed(1) }}min</div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="sortedByTiming.length > 0" class="timeline-table">
      <div class="table-header">
        <div class="col-event">Événement</div>
        <div class="col-placement">Placement</div>
        <div class="col-duration">Durée exit</div>
        <div class="col-gain">Gain estim.</div>
        <div class="col-score">Score</div>
      </div>
      <div v-for="event in sortedByTiming" :key="event.eventType" class="table-row">
        <div class="col-event">{{ event.eventType }}</div>
        <div class="col-placement">{{ event.placementSeconds }}sec</div>
        <div class="col-duration">{{ event.exitMinutes }}min</div>
        <div class="col-gain">+{{ event.estimatedGain }}p</div>
        <div class="col-score">
          <span v-if="event.tradabilityScore >= 80" class="score-good">{{ Math.round(event.tradabilityScore) }}</span>
          <span v-else-if="event.tradabilityScore >= 60" class="score-medium">{{ Math.round(event.tradabilityScore) }}</span>
          <span v-else class="score-bad">{{ Math.round(event.tradabilityScore) }}</span>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-if="sortedByTiming.length === 0" class="empty-state">
      <p>Aucune donnée de timing disponible</p>
    </div>
  </div>
</template>

<style scoped>
.timing-analysis-block {
  animation: slideIn 0.3s ease-out 0.2s both;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  border-bottom: 1px solid rgba(251, 146, 60, 0.3);
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

/* Fast & Slow Row */
.fast-slow-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 16px;
}

.fastest-event {
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.15), rgba(52, 211, 153, 0.15));
  border: 1px solid rgba(16, 185, 129, 0.3);
  border-radius: 10px;
  padding: 12px;
}

.fastest-label {
  font-size: 11px;
  font-weight: 600;
  color: #10b981;
  margin-bottom: 8px;
}

.fastest-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.fastest-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
}

.fastest-value {
  text-align: right;
}

.fastest-sublabel {
  font-size: 11px;
  color: #a0aec0;
}

.fastest-number {
  font-size: 18px;
  font-weight: 700;
  color: #10b981;
}

.timeline-table {
  background: rgba(0, 0, 0, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 16px;
}

.table-header {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
  gap: 8px;
  background: rgba(0, 0, 0, 0.3);
  padding: 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  font-size: 10px;
  font-weight: 600;
  color: #a0aec0;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.table-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
  gap: 8px;
  padding: 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  font-size: 12px;
  align-items: center;
  transition: background 0.2s ease;
}

.table-row:hover {
  background: rgba(255, 255, 255, 0.02);
}

.table-row:last-child {
  border-bottom: none;
}

.col-event {
  color: #ffffff;
  font-weight: 500;
}

.col-placement {
  color: #fb923c;
  text-align: center;
  font-weight: 600;
}

.col-duration {
  color: #60a5fa;
  text-align: center;
  font-weight: 600;
}

.col-gain {
  color: #10b981;
  text-align: center;
  font-weight: 600;
}

.col-score {
  text-align: center;
}

.score-good {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 11px;
}

.score-medium {
  background: rgba(251, 146, 60, 0.2);
  color: #fb923c;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 11px;
}

.score-bad {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  font-size: 11px;
}

.slowest-event {
  background: linear-gradient(135deg, rgba(239, 68, 68, 0.15), rgba(251, 113, 113, 0.15));
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 10px;
  padding: 12px;
}

.slowest-label {
  font-size: 11px;
  font-weight: 600;
  color: #ef4444;
  margin-bottom: 8px;
}

.slowest-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.slowest-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #ffffff;
}

.slowest-value {
  text-align: right;
}

.slowest-sublabel {
  font-size: 11px;
  color: #a0aec0;
}

.slowest-number {
  font-size: 18px;
  font-weight: 700;
  color: #ef4444;
}

.empty-state {
  border: 1px dashed rgba(255, 255, 255, 0.2);
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  color: #a0aec0;
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
