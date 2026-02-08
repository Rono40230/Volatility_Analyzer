// composables/useStraddleAnalysis.ts - Composable pour les calculs Straddle
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface OptimalOffset { offset_points: number; percentile_95_wicks: number; with_margin: number; sl_adjusted_points: number; hard_tp_points?: number }
export interface WinRateMetric { total_trades: number; wins: number; losses: number; whipsaws: number; win_rate_percentage: number; win_rate_adjusted: number }
export interface WhipsawDetailResponse { entry_candle_index: number; trigger_minute: number; entry_price: number; buy_stop: number; sell_stop: number }
export interface WhipsawMetric { total_trades: number; whipsaw_count: number; whipsaw_frequency_percentage: number; risk_level: string; risk_color: string; sl_adjusted_points: number; win_rate_adjusted: number; trailing_stop_adjusted: number; timeout_adjusted_minutes: number; whipsaw_details: WhipsawDetailResponse[] }
export interface ConfidenceMetric { score: number; sample_size_warning: boolean }
export interface StraddleMetricsResponse { symbol: string; hour: number; candle_count: number; offset_optimal: OptimalOffset; win_rate: WinRateMetric; whipsaw: WhipsawMetric; confidence: ConfidenceMetric; spread_cost: number }

export function useStraddleAnalysis() {
  const isLoading = ref(false)
  const offsetOptimal = ref<OptimalOffset | null>(null)
  const winRate = ref<WinRateMetric | null>(null)
  const whipsawAnalysis = ref<WhipsawMetric | null>(null)
  const whipsawDetails = ref<WhipsawDetailResponse[]>([])
  const confidence = ref<ConfidenceMetric | null>(null)
  const spreadCost = ref<number>(0)
  const error = ref<string | null>(null)

  const chargerBougiesPourQuart = async (symbol: string, hour: number, quarter: number): Promise<any[]> => {
    try {
      const response = await invoke<any>('get_candles_for_quarter', { symbol, hour, quarter })
      return response.candles || []
    } catch (err) {
      return []
    }
  }

  const analyzeStraddleMetrics = async (symbol: string, hour: number, quarter: number) => {
    try {
      isLoading.value = true
      error.value = null
      try {
        await invoke<string>('load_pair_candles', { symbol })
      } catch (preloadErr) {
        // Préchargement échoué - peut-être déjà chargée
      }
      const result = await invoke<StraddleMetricsResponse>('analyze_straddle_metrics', { symbol, hour, quarter })
      offsetOptimal.value = result.offset_optimal
      winRate.value = result.win_rate
      whipsawAnalysis.value = result.whipsaw
      whipsawDetails.value = result.whipsaw.whipsaw_details || []
      confidence.value = result.confidence
      spreadCost.value = result.spread_cost

      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      return null
    } finally {
      isLoading.value = false
    }
  }

  const winRateColor = computed(() => {
    if (!winRate.value) return '#6b7280'
    if (winRate.value.win_rate_percentage >= 60) return '#22c55e'
    if (winRate.value.win_rate_percentage >= 40) return '#eab308'
    return '#ef4444'
  })

  return {
    isLoading, offsetOptimal, winRate, whipsawAnalysis, whipsawDetails, confidence, spreadCost, error,
    analyzeStraddleMetrics, chargerBougiesPourQuart, winRateColor,
    // Alias
    loadCandlesForQuarter: chargerBougiesPourQuart
  }
}
