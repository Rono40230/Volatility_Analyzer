import { ref, computed } from 'vue'

export interface EventDetailState {
  eventType: string | null
  score: number | null
  avgATR: number | null
  avgPeakDelay: number | null
  avgConfidence: number | null
  tradability: 'OPTIMAL' | 'BON' | 'RISQUÉ' | null
  pair?: string
  slAdjusted?: number
  trailingStopCoefficient?: number
}

export function useEventDetail() {
  const isOpen = ref(false)
  const selectedEvent = ref<EventDetailState>({
    eventType: null,
    score: null,
    avgATR: null,
    avgPeakDelay: null,
    avgConfidence: null,
    tradability: null,
    pair: undefined,
    slAdjusted: undefined,
    trailingStopCoefficient: undefined,
  })

  function openDetail(event: EventDetailState) {
    selectedEvent.value = event
    isOpen.value = true
  }

  function closeDetail() {
    isOpen.value = false
    setTimeout(() => {
      selectedEvent.value = {
        eventType: null,
        score: null,
        avgATR: null,
        avgPeakDelay: null,
        avgConfidence: null,
        tradability: null,
        pair: undefined,
        slAdjusted: undefined,
        trailingStopCoefficient: undefined,
      }
    }, 300)
  }

  return {
    isOpen,
    selectedEvent,
    openDetail,
    closeDetail,
  }
}
