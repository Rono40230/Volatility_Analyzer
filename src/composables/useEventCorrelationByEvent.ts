import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'
import { eventTranslations } from '../stores/eventTranslations'

export interface PairImpact {
  symbol: string
  event_volatility: number
  baseline_volatility: number
  multiplier: number
}

export interface EventImpactResult {
  event_id: number
  event_name: string
  datetime: string
  last_datetime: string
  country: string
  currency: string
  event_count: number
  window_start: string
  window_end: string
  pair_impacts: PairImpact[]
  observations: string[]
}

interface PastEvent {
  name: string
  count: number
}

export function useEventCorrelationByEvent(pastEvents: PastEvent[], calendarId: number | null) {
  const store = useAnalysisStore()
  const selectedEventId = ref<string>('')
  const eventImpact = ref<EventImpactResult | null>(null)
  const loadingEvent = ref(false)
  const eventVolatilitySortOrder = ref<'asc' | 'desc'>('desc')

  onMounted(() => {
    if (store.eventCorrelationData?.eventImpact) {
      eventImpact.value = store.eventCorrelationData.eventImpact
      selectedEventId.value = store.selectedEvent
    }
  })

  async function loadEventImpact() {
    if (!selectedEventId.value) return
    loadingEvent.value = true
    try {
      const selectedEvent = pastEvents.find(e => e.name === selectedEventId.value)
      const eventCount = selectedEvent?.count || 0
      eventImpact.value = await invoke<EventImpactResult>('get_event_impact_by_pair', { eventType: selectedEventId.value, eventCount, calendarId })
      store.setEventSelection(selectedEventId.value, calendarId)
      store.setEventCorrelationData({ event: selectedEventId.value, eventImpact: eventImpact.value })
    } catch (error) {
      console.error('Erreur analyse événement:', error)
      eventImpact.value = null
    } finally {
      loadingEvent.value = false
    }
  }

  function sortEventVolatility() {
    eventVolatilitySortOrder.value = eventVolatilitySortOrder.value === 'asc' ? 'desc' : 'asc'
    if (!eventImpact.value?.pair_impacts) return
    const sorted = [...eventImpact.value.pair_impacts]
    sorted.sort((a, b) => eventVolatilitySortOrder.value === 'asc' ? a.event_volatility - b.event_volatility : b.event_volatility - a.event_volatility)
    eventImpact.value.pair_impacts = sorted
  }

  function getEventLabel(eventName: string): string {
    const translation = eventTranslations[eventName]
    return translation ? `${eventName} (${translation.fr}) ${translation.flag}` : eventName
  }

  function getMultiplierClass(multiplier: number): string {
    if (multiplier >= 50) return 'mult-extreme'
    if (multiplier >= 20) return 'mult-very-high'
    if (multiplier >= 10) return 'mult-high'
    if (multiplier >= 5) return 'mult-medium'
    return 'mult-low'
  }

  function formatDateRange(datetime: string): string {
    const date = new Date(datetime)
    return date.toLocaleString('fr-FR', { day: 'numeric', month: 'long', year: 'numeric' })
  }

  return {
    selectedEventId,
    eventImpact,
    loadingEvent,
    eventVolatilitySortOrder,
    loadEventImpact,
    sortEventVolatility,
    getEventLabel,
    getMultiplierClass,
    formatDateRange
  }
}
