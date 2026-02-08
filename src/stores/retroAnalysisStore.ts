import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { RetroGraphData } from '../composables/useRetroAnalysisGraphData'
import type { PeakDelayData, DecayProfileData } from '../composables/useRetrospectiveAnalysis'

export const useRetroAnalysisStore = defineStore('retroAnalysis', () => {
  const selectedPair = ref('')
  const selectedEventType = ref('')
  
  const graphData = ref<RetroGraphData | null>(null)
  const peakDelayResults = ref<PeakDelayData | null>(null)
  const decayResults = ref<DecayProfileData | null>(null)
  
  const loading = ref(false)
  const error = ref<string | null>(null)

  function reset() {
    selectedPair.value = ''
    selectedEventType.value = ''
    graphData.value = null
    peakDelayResults.value = null
    decayResults.value = null
    loading.value = false
    error.value = null
  }

  return {
    selectedPair,
    selectedEventType,
    graphData,
    peakDelayResults,
    decayResults,
    loading,
    error,
    reset
  }
})
