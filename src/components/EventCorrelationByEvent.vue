<template>
  <div v-if="loadingEvent" class="loading"><div class="spinner" /><p>Analyse de l'impact de l'événement...</p></div>

  <EventSelectorPanel
    v-model="selectedEventId"
    :past-events="pastEvents"
    :event-impact="eventImpact"
    :loading="loadingEvent"
    :get-event-label="getEventLabel"
    @load="loadEventImpact"
    @archive="openArchiveModal"
  />

  <div v-if="eventImpact && !loadingEvent" class="event-impact-results">
    <EventCorrelationTable
      :event-impact="eventImpact"
      :format-date-range="formatDateRange"
      :get-multiplier-class="getMultiplierClass"
      @sort="sortEventVolatility"
    />

    <div class="observations-card">
      <h3>💡 Observations</h3>
      <ul>
        <li v-for="(obs, index) in eventImpact.observations" :key="index">{{ obs }}</li>
      </ul>
    </div>
  </div>

  <ArchiveModal
    :show="showArchiveModal"
    archive-type="Corrélation événement/paire"
    :period-start="archivePeriodStart"
    :period-end="archivePeriodEnd"
    :event-name="eventImpact?.event_name"
    :event-name-fr="eventImpact?.event_name ? eventTranslations[eventImpact.event_name]?.fr : ''"
    :event-flag="eventImpact?.event_name ? getEventFlag(eventImpact.event_name) : ''"
    :data-json="archiveDataJson"
    @close="showArchiveModal = false"
    @saved="handleArchiveSaved"
  />
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { eventTranslations } from '../stores/eventTranslations'
import { useEventCorrelationByEvent } from '../composables/useEventCorrelationByEvent'
import ArchiveModal from './ArchiveModal.vue'
import EventSelectorPanel from './EventSelectorPanel.vue'
import EventCorrelationTable from './EventCorrelationTable.vue'

interface PastEvent {
  name: string
  count: number
}

const props = defineProps<{ pastEvents: PastEvent[]; calendarId: number | null }>()
const emit = defineEmits<{ 'event-loaded': [any] }>()

const {
  selectedEventId,
  eventImpact,
  loadingEvent,
  loadEventImpact,
  sortEventVolatility,
  getEventLabel,
  getMultiplierClass,
  formatDateRange
} = useEventCorrelationByEvent(props.pastEvents, props.calendarId)

const showArchiveModal = ref(false)
const archivePeriodStart = ref('')
const archivePeriodEnd = ref('')
const archiveDataJson = ref('')

function getEventFlag(eventName: string): string {
  return eventTranslations[eventName]?.flag || ''
}

function openArchiveModal() {
  if (!eventImpact.value) return
  archivePeriodStart.value = eventImpact.value.datetime
  archivePeriodEnd.value = eventImpact.value.last_datetime
  archiveDataJson.value = JSON.stringify({ eventImpact: eventImpact.value, selectedEvent: selectedEventId.value })
  showArchiveModal.value = true
}

function handleArchiveSaved() {
  showArchiveModal.value = false
}
</script>

<style scoped>
.loading { text-align: center; padding: 60px 20px; color: #e2e8f0; }
.spinner { width: 50px; height: 50px; border: 4px solid #2d3748; border-top: 4px solid #667eea; border-radius: 50%; animation: spin 1s linear infinite; margin: 0 auto 20px; }
@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }
.event-impact-results { display: flex; flex-direction: column; gap: 30px; }
.observations-card { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.observations-card h3 { color: #e2e8f0; margin-bottom: 15px; }
.observations-card ul { list-style: none; padding: 0; }
.observations-card li { padding: 10px; margin-bottom: 8px; background: #2d3748; border-left: 3px solid #667eea; border-radius: 4px; color: #e2e8f0; }
</style>
