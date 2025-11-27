import { ref } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import {
  formatSliceTime,
  loadMovementQuality,
  loadVolatilityDuration,
  createBestSlice,
  type MovementQuality,
  type VolatilityDuration
} from './useMetricsAnalysisData.helpers'

export function useMetricsAnalysisData() {
  const analysisData = ref<any>(null)
  const sliceAnalyses = ref<SliceAnalysis[] | null>(null)
  const movementQualities = ref<Record<string, MovementQuality>>({})
  const volatilityDuration = ref<VolatilityDuration | null>(null)
  const tradingPlan = ref<any>(null)
  const entryWindowAnalysis = ref<any>({ optimal_offset: 0, optimal_win_rate: 0 })

  async function updateAnalysis(analysisResult: AnalysisResult) {
    sliceAnalyses.value = []
    movementQualities.value = {}
    volatilityDuration.value = null
    tradingPlan.value = null
    entryWindowAnalysis.value = { optimal_offset: 0, optimal_win_rate: 0 }

    const result = analysisResult
    const [bestHour, bestQuarter] = result.best_quarter
    analysisData.value = {
      globalMetrics: result.global_metrics,
      symbol: result.symbol,
      confidence: Math.round(result.confidence_score),
      strategy: 'SCALPING STANDARD',
      bestHours: `${bestHour}:${bestQuarter * 15}-${bestHour}:${(bestQuarter + 1) * 15}`
    }

    if (!result.stats_15min?.length) return

    const bestSliceStats = result.stats_15min.find(s => s.hour === bestHour && s.quarter === bestQuarter)
    if (!bestSliceStats) return

    // Load movement quality first
    const { score: movementQualityScore, qualities } = await loadMovementQuality(
      result.symbol,
      bestHour,
      bestQuarter
    )
    movementQualities.value = qualities

    // Create best slice with movement quality score
    const bestSlice = createBestSlice(bestSliceStats, bestHour, bestQuarter, movementQualityScore)
    sliceAnalyses.value = [bestSlice]
    tradingPlan.value = bestSlice.tradingPlan

    // Load volatility duration
    volatilityDuration.value = await loadVolatilityDuration(result.symbol, bestHour, bestQuarter)
  }

  return {
    analysisData,
    sliceAnalyses,
    movementQualities,
    volatilityDuration,
    tradingPlan,
    entryWindowAnalysis,
    updateAnalysis
  }
}
