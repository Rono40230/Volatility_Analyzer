import { ref } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import {
  formatSliceTime,
  loadMovementQuality,
  loadEntryWindowAnalysis,
  extractVolatilityDuration,
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

  async function updateAnalysis(analysisResult: AnalysisResult, isArchiveMode = false) {
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
      bestHours: `${bestHour}:${bestQuarter * 15}-${bestHour}:${(bestQuarter + 1) * 15}`,
      stats_15min: result.stats_15min ?? []
    }

    if (!result.stats_15min?.length) return

    const bestSliceStats = result.stats_15min.find(s => s.hour === bestHour && s.quarter === bestQuarter)
    if (!bestSliceStats) return

    // En mode archive, ne pas appeler les APIs de recalcul
    if (!isArchiveMode) {
      // Load movement quality first
      const { score: movementQualityScore, qualities } = await loadMovementQuality(
        result.symbol,
        bestHour,
        bestQuarter
      )
      movementQualities.value = qualities

      // Load entry window analysis (optimal offset and win rate for entry)
      const entryAnalysis = await loadEntryWindowAnalysis(result.symbol, bestHour, bestQuarter)
      if (entryAnalysis) {
        entryWindowAnalysis.value = entryAnalysis
      }
    }

    // Create best slice with movement quality score
    const bestSlice = createBestSlice(bestSliceStats, bestHour, bestQuarter, movementQualities.value?.score ?? 0)
    sliceAnalyses.value = [bestSlice]
    tradingPlan.value = bestSlice.tradingPlan

    // Extract volatility duration from table data (no API call, copy/paste from tableau)
    volatilityDuration.value = extractVolatilityDuration(bestSliceStats)
  }

  async function updateAnalysisForQuarter(analysisResult: AnalysisResult, selectedHour: number, selectedQuarter: number) {
    const result = analysisResult
    analysisData.value = {
      globalMetrics: result.global_metrics,
      symbol: result.symbol,
      confidence: Math.round(result.confidence_score),
      strategy: 'SCALPING STANDARD',
      bestHours: `${selectedHour}:${selectedQuarter * 15}-${selectedHour}:${(selectedQuarter + 1) * 15}`,
      stats_15min: result.stats_15min ?? []
    }

    if (!result.stats_15min?.length) return

    const selectedSliceStats = result.stats_15min.find(s => s.hour === selectedHour && s.quarter === selectedQuarter)
    if (!selectedSliceStats) return

    // Load movement quality for selected quarter
    const { score: movementQualityScore, qualities } = await loadMovementQuality(
      result.symbol,
      selectedHour,
      selectedQuarter
    )
    movementQualities.value = qualities

    // Create slice for selected quarter
    const selectedSlice = createBestSlice(selectedSliceStats, selectedHour, selectedQuarter, movementQualityScore)
    sliceAnalyses.value = [selectedSlice]
    tradingPlan.value = selectedSlice.tradingPlan

    // Extract volatility duration
    volatilityDuration.value = extractVolatilityDuration(selectedSliceStats)

    // Load entry window analysis for selected quarter
    const entryAnalysis = await loadEntryWindowAnalysis(result.symbol, selectedHour, selectedQuarter)
    if (entryAnalysis) {
      entryWindowAnalysis.value = entryAnalysis
    }
  }

  return {
    analysisData,
    sliceAnalyses,
    movementQualities,
    volatilityDuration,
    tradingPlan,
    entryWindowAnalysis,
    updateAnalysis,
    updateAnalysisForQuarter
  }
}
