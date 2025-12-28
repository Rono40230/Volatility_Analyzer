import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'

export interface RetroGraphData {
  atr_timeline_before: number[]      // 30 points (T-30 à T0)
  atr_timeline_after: number[]       // 90 points (T0 à T+90)
  body_timeline_before: number[]     // Body% (T-30 à T0)
  body_timeline_after: number[]      // Body% (T0 à T+90)
  noise_ratio_before: number         // Ratio bruit AVANT
  noise_ratio_during: number         // Ratio bruit PENDANT
  noise_ratio_after: number          // Ratio bruit APRÈS
  volatility_increase_percent: number // Impact en %
  event_count: number
  event_type: string
  pair: string
  event_datetime: string             // ISO 8601: heure de l'événement
  timezone_offset: string            // Ex: "UTC+0"
  meilleur_moment: number            // Offset optimal en minutes
  stop_loss: number                  // SL en pips
  trailing_stop: number              // TS coefficient
  timeout: number                    // Timeout en minutes
  offset: number                     // Offset d'entrée en points
  stop_loss_recovery: number         // SL Recovery en points
  
  // Nouveaux champs Simultané
  stop_loss_simultaneous: number
  trailing_stop_simultaneous: number
  offset_simultaneous: number
  stop_loss_recovery_simultaneous: number

  point_value: number                // Valeur d'un point pour normalisation
  avg_deviation: number
  surprise_event_count: number
}

export function useRetroAnalysisGraphData() {
  const store = useVolatilityStore()
  const graphData = ref<RetroGraphData | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function chargerDonnéesGraph(pair: string, eventType: string, minDeviation: number | null = null) {
    // Vérifier la cache d'abord (TODO: inclure minDeviation dans la clé de cache)
    // Pour l'instant, on bypass le cache si minDeviation est défini
    if (minDeviation === null) {
        const cached = store.getRetroAnalysisCache(pair, eventType)
        if (cached) {
        graphData.value = cached
        loading.value = false
        error.value = null
        return
        }
    }

    loading.value = true
    error.value = null
    try {
      const data = await invoke<RetroGraphData>('analyze_volatility_profile', {
        pair,
        eventType,
        minDeviation
      })
      graphData.value = data
      // Sauvegarder dans la cache seulement si pas de filtre (pour l'instant)
      if (minDeviation === null) {
          store.cacheRetroAnalysis(pair, eventType, data)
      }
    } catch (e) {
      error.value = typeof e === 'string' ? e : (e instanceof Error ? e.message : 'Erreur inconnue')
      graphData.value = null
    } finally {
      loading.value = false
    }
  }

  return {
    graphData,
    loading,
    error,
    chargerDonnéesGraph
  }
}
