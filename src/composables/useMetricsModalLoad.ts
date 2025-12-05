// composables/useMetricsModalLoad.ts - Logic for loading metrics in MetricsAnalysisModal
import { watch, onMounted, Ref } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import { useMetricsAnalysisData } from './useMetricsAnalysisData'
import { useStraddleAnalysis } from './useStraddleAnalysis'

interface ModalProps {
  analysisResult: AnalysisResult | null
  isArchiveMode?: boolean
  preSelectedHour?: number
  preSelectedQuarter?: number
  archivedData?: any
}

export function useMetricsModalLoad(props: ModalProps, isOpen: Ref<boolean>) {
  const { analysisData, sliceAnalyses, movementQualities, volatilityDuration, tradingPlan, entryWindowAnalysis, updateAnalysis, updateAnalysisForQuarter } = useMetricsAnalysisData()
  const { offsetOptimal, winRate, whipsawAnalysis, analyzeStraddleMetrics } = useStraddleAnalysis()

  const loadAnalysis = async () => {
    if (!props.analysisResult) return
    try {
      // En mode archive, restaurer directement les données sauvegardées
      if (props.isArchiveMode && props.archivedData) {
        if (props.archivedData.sliceAnalyses) sliceAnalyses.value = props.archivedData.sliceAnalyses
        if (props.archivedData.movementQualities) movementQualities.value = props.archivedData.movementQualities
        if (props.archivedData.volatilityDuration) volatilityDuration.value = props.archivedData.volatilityDuration
        if (props.archivedData.tradingPlan) tradingPlan.value = props.archivedData.tradingPlan
        if (props.archivedData.entryWindowAnalysis) entryWindowAnalysis.value = props.archivedData.entryWindowAnalysis
        if (props.archivedData.offsetOptimal) offsetOptimal.value = props.archivedData.offsetOptimal
        if (props.archivedData.winRate) winRate.value = props.archivedData.winRate
        if (props.archivedData.whipsawAnalysis) whipsawAnalysis.value = props.archivedData.whipsawAnalysis
        if (props.archivedData.analysisResult) {
          analysisData.value = {
            symbol: props.analysisResult.symbol,
            period_start: props.analysisResult.period_start,
            period_end: props.analysisResult.period_end
          }
        }
        return
      }
      // Mode normal: recalculer les analyses
      if (props.preSelectedHour !== undefined && props.preSelectedQuarter !== undefined) {
        await updateAnalysisForQuarter(props.analysisResult, props.preSelectedHour, props.preSelectedQuarter)
        const symbol = props.analysisResult.symbol || 'EURUSD'
        await analyzeStraddleMetrics(symbol, props.preSelectedHour, props.preSelectedQuarter)
      } else {
        await updateAnalysis(props.analysisResult, false)
        const symbol = props.analysisResult.symbol || 'EURUSD'
        const [bestHour, bestQuarter] = props.analysisResult.best_quarter
        await analyzeStraddleMetrics(symbol, bestHour, bestQuarter)
      }
    } catch (error) {
      // Error handling
    }
  }

  watch(() => props.analysisResult, loadAnalysis)
  watch(() => isOpen.value, (isOpenVal) => { if (isOpenVal) loadAnalysis() })
  watch(() => ({ hour: props.preSelectedHour, quarter: props.preSelectedQuarter }), async (newSelection) => {
    if (newSelection.hour !== undefined && newSelection.quarter !== undefined && props.analysisResult) {
      if (props.isArchiveMode) return
      try {
        await updateAnalysisForQuarter(props.analysisResult, newSelection.hour, newSelection.quarter)
        const symbol = props.analysisResult.symbol || 'EURUSD'
        await analyzeStraddleMetrics(symbol, newSelection.hour, newSelection.quarter)
      } catch (error) {
        // Error handling
      }
    }
  })
  onMounted(loadAnalysis)

  return { analysisData, sliceAnalyses, movementQualities, volatilityDuration, tradingPlan, entryWindowAnalysis, offsetOptimal, winRate, whipsawAnalysis }
}
