import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
export interface SymbolInfo {
  symbol: string
  file_path: string
}

export interface EventInHour {
  event_name: string
  impact: string
  datetime: string
  volatility_increase: number
}

export interface HourlyStats {
  hour: number
  candle_count: number
  atr_mean: number
  atr_max: number
  volatility_mean: number
  range_mean: number
  body_range_mean: number
  shadow_ratio_mean: number
  volume_imbalance_mean: number
  noise_ratio_mean: number
  breakout_percentage: number
  events: EventInHour[]
}

export interface Stats15Min {
  hour: number           // 0-23
  quarter: number        // 0-3 (00-15min, 15-30min, 30-45min, 45-60min)
  candle_count: number
  atr_mean: number
  atr_max: number
  volatility_mean: number
  range_mean: number
  body_range_mean: number
  shadow_ratio_mean: number
  volume_imbalance_mean: number
  noise_ratio_mean: number
  breakout_percentage: number
  events: EventInHour[]
  peak_duration_minutes?: number
  volatility_half_life_minutes?: number
  recommended_trade_expiration_minutes?: number
}

export interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  mean_range: number
  total_candles: number
}

export interface CalendarEvent {
  id: number
  symbol: string
  event_time: string
  impact: string
  description: string
  actual: number | null
  forecast: number | null
  previous: number | null
  created_at: string
}

export interface CorrelatedEvent {
  event: CalendarEvent
  volatility_hour: number
  volatility_increase: number
  correlation_score: number
}

export interface AnalysisResult {
  symbol: string
  period_start: string
  period_end: string
  timeframe: string
  hourly_stats: HourlyStats[]
  stats_15min: Stats15Min[]      // Nouvelles stats pour scalping
  best_quarter: [number, number] // [hour, quarter] - meilleur quarter de la journée
  confidence_score: number
  recommendation: string
  risk_level: string
  global_metrics: GlobalMetrics
  correlated_events: CorrelatedEvent[]
}

export const useVolatilityStore = defineStore('volatility', () => {
  const symbols = ref<SymbolInfo[]>([])
  const selectedSymbol = ref('')
  const analysisResult = ref<AnalysisResult | null>(null)
  const loading = ref(false)
  const error = ref('')
  const dataRefreshTrigger = ref(0)
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
    loadSymbols,
    analyzeSymbol,
    getHourlyStats,
    clearAnalysis,
    triggerDataRefresh,
  }
})
