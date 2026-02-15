import { ref, watch, onMounted } from 'vue'
import type { AnalysisResult } from '../stores/volatility'
import { useMetricsAnalysisData } from './useMetricsAnalysisData'

interface ModalProps {
  isOpen: boolean
  analysisResult: AnalysisResult | null
  preSelectedHour?: number
  preSelectedQuarter?: number
}

export function useMetricsModalLogic(props: ModalProps) {
  const { updateAnalysis, updateAnalysisForQuarter } = useMetricsAnalysisData()
  
  const showArchiveModal = ref(false)
  const archivePeriodStart = ref('')
  const archivePeriodEnd = ref('')
  const archiveDataJson = ref('')

  const loadAnalysis = async () => {
    if (!props.analysisResult) return
    
    try {
      if (props.preSelectedHour !== undefined && props.preSelectedQuarter !== undefined) {
        await updateAnalysisForQuarter(props.analysisResult, props.preSelectedHour, props.preSelectedQuarter)
      } else {
        await updateAnalysis(props.analysisResult)
      }
    } catch (error) {
      // Error handling
    }
  }

  watch(() => props.analysisResult, loadAnalysis)
  watch(() => props.isOpen, (isOpen) => { if (isOpen) loadAnalysis() })
  watch(() => ({ hour: props.preSelectedHour, quarter: props.preSelectedQuarter }), async (newSelection) => {
    if (newSelection.hour !== undefined && newSelection.quarter !== undefined && props.analysisResult) {
      try {
        await updateAnalysisForQuarter(props.analysisResult, newSelection.hour, newSelection.quarter)
      } catch (error) {
        // Error handling
      }
    }
  })

  onMounted(loadAnalysis)

  const openArchiveModal = (result: AnalysisResult, sliceAnalyses: Record<string, unknown>, movementQualities: Record<string, unknown>, volatilityDuration: Record<string, unknown>, tradingPlan: Record<string, unknown>, entryWindowAnalysis: Record<string, unknown>) => {
    if (result.period_start && result.period_end) {
      archivePeriodStart.value = result.period_start
      archivePeriodEnd.value = result.period_end
    } else {
      const now = new Date()
      const oneYearAgo = new Date(now.getFullYear() - 1, now.getMonth(), now.getDate())
      archivePeriodStart.value = oneYearAgo.toISOString()
      archivePeriodEnd.value = now.toISOString()
    }
    
    archiveDataJson.value = JSON.stringify({
      analysisResult: result,
      sliceAnalyses,
      movementQualities,
      volatilityDuration,
      tradingPlan,
      entryWindowAnalysis
    })
    
    showArchiveModal.value = true
  }

  return {
    showArchiveModal,
    archivePeriodStart,
    archivePeriodEnd,
    archiveDataJson,
    openArchiveModal
  }
}
