import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Types from Tauri command results
export interface PeakDelayData { peak_delay_minutes: number; peak_atr: number; event_minute: number; confidence: number; event_count: number; event_type: string }
export interface DecayProfileData { peak_atr: number; decay_rate_pips_per_minute: number; decay_speed: string; recommended_timeout_minutes: number; event_count: number; event_type: string }
export interface EventTypeData { types: string[] }

export function useRetrospectiveAnalysis() {
  const peakDelayLoading = ref(false), peakDelayError = ref<string | null>(null), peakDelayResults = ref<PeakDelayData | null>(null)
  const decayLoading = ref(false), decayError = ref<string | null>(null), decayResults = ref<DecayProfileData | null>(null)
  const eventTypesLoading = ref(false), eventTypesError = ref<string | null>(null), eventTypes = ref<string[]>([])

  const analyzePeakDelay = async (candles: any[], eventType: string) => {
    peakDelayLoading.value = true; peakDelayError.value = null
    try { peakDelayResults.value = await invoke<PeakDelayData>('analyze_peak_delay', { candles, event_type: eventType }) }
    catch (e) { peakDelayError.value = String(e); peakDelayResults.value = null }
    finally { peakDelayLoading.value = false }
  }

  const analyzeDecayProfile = async (candles: any[], eventType: string) => {
    decayLoading.value = true; decayError.value = null
    try { decayResults.value = await invoke<DecayProfileData>('analyze_decay_profile', { candles, event_type: eventType }) }
    catch (e) { decayError.value = String(e); decayResults.value = null }
    finally { decayLoading.value = false }
  }

  const loadEventTypes = async () => {
    eventTypesLoading.value = true; eventTypesError.value = null
    try { const data = await invoke<EventTypeData>('get_event_types'); eventTypes.value = data.types }
    catch (e) { eventTypesError.value = String(e); eventTypes.value = [] }
    finally { eventTypesLoading.value = false }
  }

  return { peakDelayLoading, peakDelayError, peakDelayResults, analyzePeakDelay, decayLoading, decayError, decayResults, analyzeDecayProfile, eventTypesLoading, eventTypesError, eventTypes, loadEventTypes }
}
