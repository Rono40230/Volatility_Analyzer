// composables/useMetricsModalLoad.ts - Logic for loading metrics in MetricsAnalysisModal
import { watch, Ref } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import { useMetricsAnalysisData } from './useMetricsAnalysisData'
import type { RecurringEvent, MovementQuality, VolatilityDuration } from './useMetricsAnalysisData.helpers'

export interface ArchivedAnalysisData {
  analysisResult: AnalysisResult
  movementQualities: Record<string, MovementQuality>
  volatilityDuration: VolatilityDuration | null
  entryWindowAnalysis: Record<string, unknown>
  recurringEvents?: RecurringEvent[]
}

interface ModalProps {
  analysisResult: AnalysisResult | null
  isArchiveMode?: boolean
  preSelectedHour?: number
  preSelectedQuarter?: number
  archivedData?: ArchivedAnalysisData
}

export function useMetricsModalLoad(props: ModalProps, isOpen: Ref<boolean>) {
  const { analysisData, sliceAnalyses, movementQualities, volatilityDuration, tradingPlan, entryWindowAnalysis, recurringEvents, updateAnalysis, updateAnalysisForQuarter } = useMetricsAnalysisData()

  const loadAnalysis = async () => {
    if (!props.analysisResult || !isOpen.value) return
    try {
      // En mode archive, restaurer directement les données sauvegardées
      if (props.isArchiveMode && props.archivedData) {
        if (props.archivedData.movementQualities) movementQualities.value = props.archivedData.movementQualities
        if (props.archivedData.volatilityDuration) volatilityDuration.value = props.archivedData.volatilityDuration
        if (props.archivedData.entryWindowAnalysis) entryWindowAnalysis.value = props.archivedData.entryWindowAnalysis
        if (props.archivedData.recurringEvents) recurringEvents.value = props.archivedData.recurringEvents
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
      } else {
        await updateAnalysis(props.analysisResult, false)
      }
    } catch (error) {
      // Error handling
    }
  }

  // Ne déclencher les calculs lourds QUE quand le modal s'ouvre
  watch(() => isOpen.value, (isOpenVal) => { if (isOpenVal) loadAnalysis() })
  watch(() => ({ hour: props.preSelectedHour, quarter: props.preSelectedQuarter }), async (newSelection) => {
    if (!isOpen.value) return
    if (newSelection.hour !== undefined && newSelection.quarter !== undefined && props.analysisResult) {
      if (props.isArchiveMode) return
      try {
        await updateAnalysisForQuarter(props.analysisResult, newSelection.hour, newSelection.quarter)
      } catch (error) {
        // Error handling
      }
    }
  })

  return { analysisData, sliceAnalyses, movementQualities, volatilityDuration, tradingPlan, entryWindowAnalysis, recurringEvents }
}

