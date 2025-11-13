// stores/analysisStore.ts
// Persiste l'état des analyses entre changements d'onglets
// Conforme .clinerules: < 100 lignes

import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface EventCorrelation {
  name: string
  impact: string
  count: number
  volatility_before: number
  volatility_after: number
  volatility_total: number
  volatility_before_fmt: string
  volatility_after_fmt: string
  volatility_total_fmt: string
  correlation_score: number
}

export interface AnalysisData {
  pair?: string
  events?: EventCorrelation[]
  event?: string
  pairs?: EventCorrelation[]
  heatmapData?: unknown
}

export const useAnalysisStore = defineStore('analysis', () => {
  // État persisté
  const selectedPair = ref<string>('')
  const selectedEvent = ref<string>('')
  const selectedCalendarId = ref<number | null>(null)
  const pairCorrelationData = ref<AnalysisData | null>(null)
  const eventCorrelationData = ref<AnalysisData | null>(null)
  const heatmapData = ref<AnalysisData | null>(null)

  // Actions
  function setPairSelection(pair: string, calId: number | null = null) {
    selectedPair.value = pair
    selectedCalendarId.value = calId
  }

  function setEventSelection(event: string, calId: number | null = null) {
    selectedEvent.value = event
    selectedCalendarId.value = calId
  }

  function setPairCorrelationData(data: AnalysisData | null) {
    pairCorrelationData.value = data
  }

  function setEventCorrelationData(data: AnalysisData | null) {
    eventCorrelationData.value = data
  }

  function setHeatmapData(data: AnalysisData | null) {
    heatmapData.value = data
  }

  function clearAnalysis() {
    selectedPair.value = ''
    selectedEvent.value = ''
    pairCorrelationData.value = null
    eventCorrelationData.value = null
    heatmapData.value = null
  }

  return {
    // État
    selectedPair,
    selectedEvent,
    selectedCalendarId,
    pairCorrelationData,
    eventCorrelationData,
    heatmapData,
    // Actions
    setPairSelection,
    setEventSelection,
    setPairCorrelationData,
    setEventCorrelationData,
    setHeatmapData,
    clearAnalysis,
  }
})
