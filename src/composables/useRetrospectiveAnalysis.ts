import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Types from Tauri command results (minimal exports)
export interface PeakDelayData { event_type: string; peak_delay_minutes: number; sample_count: number; consistency_percent: number }
export interface DecayProfileData { peak_atr: number; decay_rate_pips_per_minute: number; decay_speed: string; recommended_timeout_minutes: number }
export interface EntryTimingData { entry_offset_minutes: number; win_rate: number; whipsaw_rate: number; avg_profit_pips: number; sample_size: number; quality_score: number; is_best: boolean }
export interface DirectionalBiasData { up_wins_count: number; down_wins_count: number; whipsaw_count: number; bias_value: number; asymmetry_percent: number; classification: string; confidence_level: string }
export interface WhipsawRootCauseData { early_count: number; early_percentage: number; early_avg_loss_pips: number; late_count: number; late_percentage: number; late_avg_loss_pips: number; total_whipsaws: number; dominant_type: string }

export function useRetrospectiveAnalysis() {
  // Peak Delay
  const peakDelayLoading = ref(false)
  const peakDelayError = ref<string | null>(null)
  const peakDelayResults = ref<PeakDelayData[]>([])

  const analyzePeakDelay = async (candles: any[]) => {
    peakDelayLoading.value = true
    peakDelayError.value = null
    try {
      const result = await invoke('analyze_peak_delay', { candles, event_minute: 0 })
      peakDelayResults.value = Array.isArray(result) ? result : [result as PeakDelayData]
    } catch (e) {
      peakDelayError.value = String(e)
      peakDelayResults.value = []
    } finally {
      peakDelayLoading.value = false
    }
  }

  // Decay Profile
  const decayLoading = ref(false)
  const decayError = ref<string | null>(null)
  const decayResults = ref<DecayProfileData | null>(null)

  const analyzeDecayProfile = async (candles: any[]) => {
    decayLoading.value = true
    decayError.value = null
    try {
      const result = await invoke('analyze_decay_profile', { candles })
      decayResults.value = result as DecayProfileData
    } catch (e) {
      decayError.value = String(e)
      decayResults.value = null
    } finally {
      decayLoading.value = false
    }
  }

  // Entry Timing
  const entryTimingLoading = ref(false)
  const entryTimingError = ref<string | null>(null)
  const entryTimingResults = ref<EntryTimingData[]>([])

  const analyzeEntryTiming = async (results: any[]) => {
    entryTimingLoading.value = true
    entryTimingError.value = null
    try {
      const data = await invoke('analyze_entry_timing', { results })
      entryTimingResults.value = data as EntryTimingData[]
    } catch (e) {
      entryTimingError.value = String(e)
      entryTimingResults.value = []
    } finally {
      entryTimingLoading.value = false
    }
  }

  // Directional Bias
  const biasLoading = ref(false)
  const biasError = ref<string | null>(null)
  const biasResults = ref<DirectionalBiasData | null>(null)

  const analyzeDirectionalBias = async (up_wins: number, down_wins: number, whipsaws: number) => {
    biasLoading.value = true
    biasError.value = null
    try {
      const result = await invoke('analyze_directional_bias', { up_wins, down_wins, whipsaws })
      biasResults.value = result as DirectionalBiasData
    } catch (e) {
      biasError.value = String(e)
      biasResults.value = null
    } finally {
      biasLoading.value = false
    }
  }

  // Whipsaw Root Cause
  const whipsawLoading = ref(false)
  const whipsawError = ref<string | null>(null)
  const whipsawResults = ref<WhipsawRootCauseData | null>(null)

  const analyzeWhipsawRootCause = async (early_whipsaws: number, early_avg_loss: number, late_whipsaws: number, late_avg_loss: number) => {
    whipsawLoading.value = true
    whipsawError.value = null
    try {
      const result = await invoke('analyze_whipsaw_root_cause', {
        early_whipsaws,
        early_avg_loss,
        late_whipsaws,
        late_avg_loss
      })
      whipsawResults.value = result as WhipsawRootCauseData
    } catch (e) {
      whipsawError.value = String(e)
      whipsawResults.value = null
    } finally {
      whipsawLoading.value = false
    }
  }

  return {
    // Peak Delay
    peakDelayLoading,
    peakDelayError,
    peakDelayResults,
    analyzePeakDelay,
    // Decay
    decayLoading,
    decayError,
    decayResults,
    analyzeDecayProfile,
    // Entry Timing
    entryTimingLoading,
    entryTimingError,
    entryTimingResults,
    analyzeEntryTiming,
    // Directional Bias
    biasLoading,
    biasError,
    biasResults,
    analyzeDirectionalBias,
    // Whipsaw
    whipsawLoading,
    whipsawError,
    whipsawResults,
    analyzeWhipsawRootCause
  }
}
