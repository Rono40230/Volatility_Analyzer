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
      
      // Analyse du timing (avant vs après)
      const beforeEvents = topEvents.value.filter(e => e.volatility_before > e.volatility_after)
      const afterEvents = topEvents.value.filter(e => e.volatility_after > e.volatility_before)
      const balancedEvents = topEvents.value.filter(e => Math.abs(e.volatility_after - e.volatility_before) <= 0.1)
      
      if (beforeEvents.length > afterEvents.length && beforeEvents.length > 0) {
        obs.push(`📊 Volatilité d'anticipation dominante (${beforeEvents.length}/${topEvents.value.length} événements) - Le marché se positionne AVANT les annonces.`)
      } else if (afterEvents.length > beforeEvents.length && afterEvents.length > 0) {
        obs.push(`📊 Volatilité de réaction dominante (${afterEvents.length}/${topEvents.value.length} événements) - ${selectedPair.value} réagit APRÈS les annonces.`)
      } else if (balancedEvents.length >= 3) {
        obs.push(`📊 Volatilité équilibrée - Réactions mixtes avant/après selon les événements.`)
      }
      
      // Volatilité moyenne
      const avgVolatility = topEvents.value.reduce((sum, e) => sum + e.volatility_total, 0) / topEvents.value.length
      if (avgVolatility > 5) {
        obs.push(`⚡ Volatilité événementielle élevée (${avgVolatility.toFixed(2)} pips) - Les annonces économiques impactent fortement ${selectedPair.value}.`)
      } else if (avgVolatility > 2) {
        obs.push(`⚡ Volatilité événementielle modérée (${avgVolatility.toFixed(2)} pips) - Impact mesuré sur ${selectedPair.value}.`)
      } else {
        obs.push(`⚡ Volatilité événementielle faible (${avgVolatility.toFixed(2)} pips) - ${selectedPair.value} peu sensible aux événements économiques.`)
      }
      
      // Score corrélation moyen
      const avgScore = topEvents.value.reduce((sum, e) => sum + e.correlation_score, 0) / topEvents.value.length
      if (avgScore > 60) obs.push(`🎯 Corrélation moyenne élevée (${avgScore.toFixed(1)}%) - ${selectedPair.value} est très réactive aux événements économiques.`)
      else if (avgScore > 30) obs.push(`🎯 Corrélation moyenne modérée (${avgScore.toFixed(1)}%) - Impact événementiel mesuré.`)
      else obs.push(`🎯 Corrélation moyenne faible (${avgScore.toFixed(1)}%) - ${selectedPair.value} peu affectée par les événements économiques.`)
      
      // Anomalies intéressantes
      const zeroAfterCount = topEvents.value.filter(e => e.volatility_after < 0.1).length
      if (zeroAfterCount >= 5) {
        obs.push(`🔍 Anomalie : ${zeroAfterCount} événements affichent 0 pips APRÈS → Possibilité d'offset horaire ou données manquantes après événement.`)
      }
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
