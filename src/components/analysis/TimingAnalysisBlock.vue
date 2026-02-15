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
  unit: string
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
      unit: stats.unit || 'pts'
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
        <div class="col-gain">
          +{{ event.estimatedGain.toFixed(1) }} {{ event.unit }}
        </div>
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
@import './TimingAnalysisBlock.css';
</style>