import { defineStore } from 'pinia'
import { ref } from 'vue'

// Fonctions utilitaires pour persistance localStorage
const HEATMAP_CACHE_VERSION = 3 // Incrémenter pour invalider le cache existant

function sauvegarderHeatmapStockage(data: HeatmapData, pairs: string[], calendarId: number | null) {
  const heatmapCache = {
    data,
    pairs,
    calendarId,
    timestamp: Date.now(),
    version: HEATMAP_CACHE_VERSION,
  }
  localStorage.setItem('heatmapCache', JSON.stringify(heatmapCache))
  localStorage.setItem('heatmapCalendarId', String(calendarId || 0))
}

function chargerHeatmapStockage(): { data: HeatmapData; pairs: string[]; calendarId: number | null } | null {
  try {
    const cached = localStorage.getItem('heatmapCache')
    if (!cached) return null
    const parsed = JSON.parse(cached)
    // Invalider si version obsolète
    if (parsed.version !== HEATMAP_CACHE_VERSION) {
      localStorage.removeItem('heatmapCache')
      return null
    }
    return {
      data: parsed.data,
      pairs: parsed.pairs,
      calendarId: parsed.calendarId || null,
    }
  } catch {
    return null
  }
}

function effacerHeatmapStockage() {
  localStorage.removeItem('heatmapCache')
  localStorage.removeItem('heatmapCalendarId')
}

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
}

export interface PairImpact {
  symbol: string
  event_volatility: number
  baseline_volatility: number
  event_volatility_formatted: string
  baseline_volatility_formatted: string
  points: number
  points_formatted: string
  price: number
  price_formatted: string
  multiplier: number
  direction: string
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

export interface HeatmapData {
  pairs: string[]
  event_types: Array<{ name: string; count: number; has_data?: boolean }>
  data: Record<string, Record<string, number>>
  counts?: Record<string, Record<string, number>>
}

export interface HeatmapLoadedFor {
  pairs: string[]
  calendarId: number | null
}

export interface AnalysisData {
  pair?: string
  events?: EventCorrelation[]
  event?: string
  pairs?: EventCorrelation[]
  eventImpact?: EventImpactResult
  heatmapData?: unknown
}

export const useAnalysisStore = defineStore('analysis', () => {
  const selectedPair = ref<string>('')
  const selectedEvent = ref<string>('')
  const selectedCalendarId = ref<number | null>(null)
  const pairCorrelationData = ref<AnalysisData | null>(null)
  const eventCorrelationData = ref<AnalysisData | null>(null)
  const heatmapData = ref<AnalysisData | null>(null)
  const persistentHeatmapData = ref<HeatmapData | null>(null)
  const heatmapLoadedFor = ref<HeatmapLoadedFor | null>(null)
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

  function setPersistentHeatmapData(data: HeatmapData | null, pairs: string[], calendarId: number | null) {
    persistentHeatmapData.value = data
    heatmapLoadedFor.value = { pairs, calendarId }
    if (data) {
      sauvegarderHeatmapStockage(data, pairs, calendarId)
    }
  }

  function resetHeatmapData() {
    persistentHeatmapData.value = null
    heatmapLoadedFor.value = null
    effacerHeatmapStockage()
  }

  function restoreHeatmapFromStorage() {
    const cached = chargerHeatmapStockage()
    if (cached) {
      persistentHeatmapData.value = cached.data
      heatmapLoadedFor.value = { pairs: cached.pairs, calendarId: cached.calendarId }
      selectedCalendarId.value = cached.calendarId
      return true
    }
    return false
  }

  function obtenirIdCalendrierHeatmapStocke(): number | null {
    try {
      const calId = localStorage.getItem('heatmapCalendarId')
      return calId ? parseInt(calId, 10) || null : null
    } catch {
      return null
    }
  }

  function shouldReloadHeatmap(pairs: string[], calendarId: number | null): boolean {
    if (!heatmapLoadedFor.value) return true
    if (heatmapLoadedFor.value.calendarId !== calendarId) return true
    if (pairs.length !== heatmapLoadedFor.value.pairs.length) return true
    return !pairs.every((p, i) => p === heatmapLoadedFor.value.pairs[i])
  }

  function clearAnalysis() {
    selectedPair.value = ''
    selectedEvent.value = ''
    pairCorrelationData.value = null
    eventCorrelationData.value = null
    heatmapData.value = null
    resetHeatmapData()
  }

  return {
    selectedPair,
    selectedEvent,
    selectedCalendarId,
    pairCorrelationData,
    eventCorrelationData,
    heatmapData,
    persistentHeatmapData,
    heatmapLoadedFor,
    setPairSelection,
    setEventSelection,
    setPairCorrelationData,
    setEventCorrelationData,
    setHeatmapData,
    setPersistentHeatmapData,
    resetHeatmapData,
    restoreHeatmapFromStorage,
    obtenirIdCalendrierHeatmapStocke,
    shouldReloadHeatmap,
    clearAnalysis,
  }
})
