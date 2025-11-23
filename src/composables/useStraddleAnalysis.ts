// composables/useStraddleAnalysis.ts - Composable pour les calculs Straddle
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface OptimalOffset {
  offset_pips: number
  percentile_95_wicks: number
  with_margin: number
}

export interface WinRateMetric {
  total_trades: number
  wins: number
  losses: number
  whipsaws: number
  win_rate_percentage: number
}

export interface WhipsawMetric {
  total_trades: number
  whipsaw_count: number
  whipsaw_frequency_percentage: number
  risk_level: string
  risk_color: string
}

export interface StraddleMetricsResponse {
  symbol: string
  hour: number
  candle_count: number
  offset_optimal: OptimalOffset
  win_rate: WinRateMetric
  whipsaw: WhipsawMetric
}

export function useStraddleAnalysis() {
  const isLoading = ref(false)
  const offsetOptimal = ref<OptimalOffset | null>(null)
  const winRate = ref<WinRateMetric | null>(null)
  const whipsawAnalysis = ref<WhipsawMetric | null>(null)
  const error = ref<string | null>(null)

  /**
   * Analyse complète des métriques Straddle avec VRAIES données
   * Appelle la command Tauri qui charge les candles du DB
   */
  const analyzeStraddleMetrics = async (
    symbol: string,
    hour: number,
    candles: any[]
  ) => {
    try {
      isLoading.value = true
      error.value = null

      // Validation: si pas de candles, retourner des valeurs par défaut
      if (candles.length === 0) {
        console.warn('⚠️ Pas de candles fournis - utilisation de valeurs par défaut')
        offsetOptimal.value = {
          offset_pips: 0,
          percentile_95_wicks: 0,
          with_margin: 0,
        }
        winRate.value = {
          total_trades: 0,
          wins: 0,
          losses: 0,
          whipsaws: 0,
          win_rate_percentage: 0,
        }
        whipsawAnalysis.value = {
          total_trades: 0,
          whipsaw_count: 0,
          whipsaw_frequency_percentage: 0,
          risk_level: 'N/A',
          risk_color: '#6b7280',
        }
        return null
      }

      const result = await invoke<StraddleMetricsResponse>(
        'analyze_straddle_metrics',
        {
          symbol,
          hour,
          candles,
        }
      )

      // Extraire chaque métrique
      offsetOptimal.value = result.offset_optimal
      winRate.value = result.win_rate
      whipsawAnalysis.value = result.whipsaw

      console.log('✅ TÂCHE 5 - Analyse Straddle complète:')
      console.log('   - Offset optimal:', offsetOptimal.value.offset_pips, 'pips')
      console.log('   - Win Rate:', winRate.value.win_rate_percentage.toFixed(1), '%')
      console.log('   - Whipsaw:', whipsawAnalysis.value.whipsaw_frequency_percentage.toFixed(1), '%')

      return result
    } catch (err) {
      error.value = err instanceof Error ? err.message : String(err)
      console.error('❌ Erreur analyse Straddle:', error.value)
      return null
    } finally {
      isLoading.value = false
    }
  }

  // Computed pour les couleurs
  const winRateColor = computed(() => {
    if (!winRate.value) return '#6b7280'
    if (winRate.value.win_rate_percentage >= 60) return '#22c55e' // Vert
    if (winRate.value.win_rate_percentage >= 40) return '#eab308' // Jaune
    return '#ef4444' // Rouge
  })

  return {
    // État
    isLoading,
    offsetOptimal,
    winRate,
    whipsawAnalysis,
    error,

    // Méthodes
    analyzeStraddleMetrics,

    // Computed
    winRateColor,
  }
}
