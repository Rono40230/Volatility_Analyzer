import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'

export interface EventCorrelation {
  name: string
  count: number
  volatility_before: number
  volatility_after: number
  volatility_total: number
  volatility_before_fmt: string
  volatility_after_fmt: string
  volatility_total_fmt: string
  correlation_score: number
  has_data?: boolean
}

export interface PairCorrelationData {
  pair: string
  period_start?: string
  period_end?: string
  events: EventCorrelation[]
}

export function useEventCorrelationByPair(availablePairs: string[], isArchiveMode: boolean, archiveData?: PairCorrelationData) {
  const store = useAnalysisStore()
  const loading = ref(false)

  const selectedPair = computed({
    get: () => store.selectedPair,
    set: (value) => store.setPairSelection(value, store.selectedCalendarId)
  })

  const pairCorrelation = computed({
    get: () => {
      if (isArchiveMode && archiveData) return archiveData
      return store.pairCorrelationData as PairCorrelationData | null
    },
    set: (value) => {
      if (!isArchiveMode) store.setPairCorrelationData(value)
    }
  })

  const topEvents = computed(() => {
    if (!pairCorrelation.value) return []
    return pairCorrelation.value.events.slice(0, 10).sort((a, b) => b.correlation_score - a.correlation_score)
  })

  const observations = computed(() => {
    if (!topEvents.value.length) return []
    const obs: string[] = []
    const topEvent = topEvents.value[0]
    if (topEvent) {
      obs.push(`L'événement "${topEvent.name}" est le plus corrélé avec ${selectedPair.value} (score: ${topEvent.correlation_score.toFixed(1)}%).`)
      const avgScore = topEvents.value.reduce((sum, e) => sum + e.correlation_score, 0) / topEvents.value.length
      if (avgScore > 60) obs.push(`Corrélation moyenne élevée (${avgScore.toFixed(1)}%) - ${selectedPair.value} est très réactive aux événements économiques.`)
      else if (avgScore > 30) obs.push(`Corrélation moyenne modérée (${avgScore.toFixed(1)}%) - Impact événementiel mesuré.`)
      else obs.push(`Corrélation moyenne faible (${avgScore.toFixed(1)}%) - ${selectedPair.value} peu affectée par les événements économiques.`)
    }
    return obs
  })

  async function loadPairCorrelation() {
    if (isArchiveMode) return
    if (!selectedPair.value) return
    loading.value = true
    try {
      const result = await invoke<PairCorrelationData>('get_pair_event_correlation', { symbol: selectedPair.value, monthsBack: 12 })
      pairCorrelation.value = result
    } catch (error) {
      console.error('Erreur corrélation paire:', error)
      pairCorrelation.value = { pair: selectedPair.value, events: [] }
    } finally {
      loading.value = false
    }
  }

  function getScoreClass(score: number): string {
    if (score >= 75) return 'score-green'
    if (score >= 50) return 'score-orange'
    return 'score-red'
  }

  return { selectedPair, pairCorrelation, topEvents, observations, loading, loadPairCorrelation, getScoreClass }
}
