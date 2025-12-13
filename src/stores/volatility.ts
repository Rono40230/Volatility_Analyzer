import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useRetroAnalysisCache } from '../composables/useRetroAnalysisCache'
import type { SymbolInfo, AnalysisResult, HourlyStats } from './volatilityTypes'

export const useVolatilityStore = defineStore('volatility', () => {
  const symbols = ref<SymbolInfo[]>([])
  const selectedSymbol = ref('')
  const analysisResult = ref<AnalysisResult | null>(null)
  const loading = ref(false)
  const error = ref('')
  const dataRefreshTrigger = ref(0)
  
  // Utiliser le composable cache
  const { retroAnalysisCache, cacheRetroAnalysis, getRetroAnalysisCache, clearRetroAnalysisCache, clearRetroAnalysisCacheForPair } = useRetroAnalysisCache()
  
  const hasAnalysis = computed(() => analysisResult.value !== null)
  const bestQuarterStats = computed(() => {
    if (!analysisResult.value) return null
    const [hour, quarter] = analysisResult.value.best_quarter
    return analysisResult.value.stats_15min.find(q => q.hour === hour && q.quarter === quarter)
  })

  async function loadSymbols() {
    loading.value = true
    error.value = ''
    try {
      symbols.value = await invoke<SymbolInfo[]>('load_symbols')
    } catch (e: Error | unknown) {
      error.value = `Erreur chargement symboles: ${e instanceof Error ? e.message : String(e)}`
    } finally {
      loading.value = false
    }
  }

  async function analyzeSymbol(symbol: string, calendarId?: number | null) {
    loading.value = true
    error.value = ''
    selectedSymbol.value = symbol
    try {
      // Récupérer le calendar_id depuis localStorage si pas fourni
      const cid = calendarId ?? parseInt(localStorage.getItem('activeCalendarId') || '0', 10)
      
      // Valider que le calendrier est sélectionné
      if (!cid || cid <= 0) {
        throw new Error('Veuillez sélectionner un calendrier avant de lancer l\'analyse')
      }
      
      analysisResult.value = await invoke<AnalysisResult>('analyze_symbol', { symbol, calendarId: cid })
    } catch (e: Error | unknown) {
      error.value = `Erreur analyse: ${e instanceof Error ? e.message : String(e)}`
      analysisResult.value = null
    } finally {
      loading.value = false
    }
  }

  async function getHourlyStats(symbol: string, hour: number, calendarId?: number | null) {
    try {
      const cid = calendarId ?? parseInt(localStorage.getItem('activeCalendarId') || '0', 10)
      
      // Valider que le calendrier est sélectionné
      if (!cid || cid <= 0) {
        throw new Error('Veuillez sélectionner un calendrier avant de lancer l\'analyse')
      }
      
      return await invoke<HourlyStats>('get_hourly_stats', { symbol, hour, calendarId: cid })
    } catch (e: Error | unknown) {
      error.value = `Erreur stats horaires: ${e instanceof Error ? e.message : String(e)}`
      return null
    }
  }

  function clearAnalysis() {
    analysisResult.value = null
    selectedSymbol.value = ''
    error.value = ''
  }

  function triggerDataRefresh() {
    dataRefreshTrigger.value++
  }

  return {
    symbols,
    selectedSymbol,
    analysisResult,
    loading,
    error,
    dataRefreshTrigger,
    hasAnalysis,
    bestQuarterStats,
    retroAnalysisCache,
    loadSymbols,
    analyzeSymbol,
    getHourlyStats,
    clearAnalysis,
    triggerDataRefresh,
    cacheRetroAnalysis,
    getRetroAnalysisCache,
    clearRetroAnalysisCache,
    clearRetroAnalysisCacheForPair,
  }
})
