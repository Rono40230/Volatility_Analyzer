<script setup lang="ts">
import { computed } from 'vue'
import { useArchiveStatistics } from '../../composables/useArchiveStatistics'
import { useEventTranslation } from '../../composables/useEventTranslation'
import { useEventDetail } from '../../composables/useEventDetail'
import EventDetailModal from './EventDetailModal.vue'
import EventGroupedByColor from './EventGroupedByColor.vue'

const { eventStatistics } = useArchiveStatistics()
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
    .filter(([eventType]) => eventType !== 'Non spécifié')
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
</script>

<template>
  <div class="event-analysis-block">
    <!-- Header -->
    <div class="header-section">
      <div class="header-content">
      </div>
    </div>

    <!-- Events List (Grouped by Color) -->
    <EventGroupedByColor v-if="sortedEvents.length > 0" :sorted-events="sortedEvents" />

    <!-- Empty State -->
    <div v-else class="empty-state">
      <p>Aucun événement analysé</p>
    </div>

    <!-- Event Detail Modal -->
    <EventDetailModal :is-open="isOpen" :event="selectedEvent" @close="closeDetail" />
  </div>
</template>

<style scoped>
@import './EventAnalysisBlock.css';
</style>