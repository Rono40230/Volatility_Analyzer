import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { GlobalAnalysisResult } from './globalAnalysisTypes'

export { type GlobalStats, type BestPairGlobal, type GoldenHourGlobal, type TradableEventGlobal, type OptimalTimeWindowGlobal, type GlobalAnalysisResult } from './globalAnalysisTypes'

export function useGlobalAnalysis() {
  const loading = ref(false)
  const result = ref<GlobalAnalysisResult | null>(null)
  const error = ref<string | null>(null)
  const loadingStep = ref('Initialisation...')
  const progress = ref(0)
  const logs = ref<string[]>([])
  const startDate = ref('')
  const endDate = ref('')
  const selectedPairs = ref<string[]>([])
  const availablePairs = ref<string[]>([])

  const sortedGoldenHours = computed(() => {
    if (!result.value) return []
    return [...result.value.golden_hours]
      .sort((a, b) => b.reliability - a.reliability)
      .slice(0, 8)
      .sort((a, b) => a.hour - b.hour)
  })

  const bestHour = computed(() => {
    if (!result.value || result.value.golden_hours.length === 0) return '?'
    const best = [...result.value.golden_hours].sort((a, b) => b.reliability - a.reliability)[0]
    return best.hour
  })

  const bestHourReliability = computed(() => {
    if (!result.value || result.value.golden_hours.length === 0) return '0'
    const best = [...result.value.golden_hours].sort((a, b) => b.reliability - a.reliability)[0]
    return best.reliability.toFixed(0)
  })

  const bestPair = computed(() => {
    if (!result.value || result.value.best_pairs.length === 0) return '?'
    return result.value.best_pairs[0].symbol
  })

  function addLog(message: string) {
    logs.value.unshift(message)
    if (logs.value.length > 5) logs.value.pop()
  }

  async function loadAvailablePairs() {
    try {
      const pairs = await invoke<string[]>('get_available_pairs')
      availablePairs.value = pairs
    } catch (e) {
      // Fallback silencieux, on garde une liste vide
      availablePairs.value = []
    }
  }

  // Charger les paires à l'initialisation du composable
  loadAvailablePairs()

  async function runAnalysis(animate = true) {
    loading.value = true
    error.value = null
    result.value = null
    progress.value = 0
    logs.value = []

    if (animate) {
      const steps = [
        { msg: 'Lecture des archives...', p: 10 },
        { msg: 'Désérialisation des données JSON...', p: 30 },
        { msg: 'Agrégation des métriques de volatilité...', p: 50 },
        { msg: 'Calcul des corrélations croisées...', p: 70 },
        { msg: 'Identification des Golden Hours...', p: 90 },
        { msg: 'Génération du rapport IA...', p: 100 }
      ]

      for (const step of steps) {
        loadingStep.value = step.msg
        addLog(step.msg)
        progress.value = step.p
        await new Promise(resolve => setTimeout(resolve, 300))
      }
    }

    try {
      const filters = {
        start_date: startDate.value || null,
        end_date: endDate.value || null,
        pairs: selectedPairs.value.length > 0 ? selectedPairs.value : null
      }
      const data = await invoke<GlobalAnalysisResult>('analyze_all_archives', { filters })
      result.value = data
    } catch (e: Error | unknown) {
      error.value = typeof e === 'string' ? e : (e instanceof Error ? e.message : "Erreur inconnue lors de l'analyse")
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    result,
    error,
    loadingStep,
    progress,
    logs,
    startDate,
    endDate,
    selectedPairs,
    availablePairs,
    sortedGoldenHours,
    bestHour,
    bestHourReliability,
    bestPair,
    runAnalysis
  }
}
