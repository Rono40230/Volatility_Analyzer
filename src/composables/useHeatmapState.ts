// Composable pour gérer l'état de la heatmap et les watchers
import { ref, watch, onMounted, onBeforeUnmount, onDeactivated } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'
import { useConversionStore } from '../stores/conversionStore'
import { useDataRefresh } from './useDataRefresh'

interface HeatmapStateProps {
  viewMode?: 'heatmap' | 'retrospective'
}

export function useHeatmapState(props: HeatmapStateProps) {
  const analysisStore = useAnalysisStore()
  const conversionStore = useConversionStore()
  const viewMode = ref<'heatmap' | 'retrospective'>('heatmap')
  const availablePairs = ref<string[]>([])
  const selectedCalendarId = ref<number | null>(null)

  // ViewMode watcher
  watch(() => props.viewMode, (newViewMode) => {
    if (newViewMode) viewMode.value = newViewMode
  }, { immediate: true })

  // Lifecycle
  onMounted(async () => {
    const hasRestoredHeatmap = analysisStore.restoreHeatmapFromStorage()
    if (hasRestoredHeatmap) {
      selectedCalendarId.value = analysisStore.obtenirIdCalendrierHeatmapStocke()
    }
    await chargerPairesDisponibles()
  })

  const { onPairDataRefresh } = useDataRefresh()
  const unsubscribe = onPairDataRefresh(chargerPairesDisponibles)
  // KeepAlive : onBeforeUnmount est appelé quand le KeepAlive est détruit (fermeture app)
  // onDeactivated est appelé quand le composant est mis en cache (switch onglet)
  onBeforeUnmount(() => unsubscribe())
  onDeactivated(() => {
    // Le listener reste actif pendant le cache pour recevoir les refresh
    // Pas d'unsubscribe ici : on veut que les données se mettent à jour en arrière-plan
  })

  // Calendar change watcher
  let isFirstCalendarChange = true
  watch(() => selectedCalendarId.value, (newCalendarId) => {
    if (isFirstCalendarChange) {
      isFirstCalendarChange = false
      return
    }
    analysisStore.resetHeatmapData()
  }, { immediate: false })

  // Pairs change watcher
  let isFirstPairsChange = true
  watch(() => availablePairs.value, (newPairs) => {
    if (isFirstPairsChange) {
      isFirstPairsChange = false
      return
    }
    if (newPairs && newPairs.length > 0) {
      analysisStore.resetHeatmapData()
    }
  }, { deep: true })

  async function gererSelectionCalendrier(filename: string) {
    const calendarId = await invoke<number | null>('get_calendar_id_by_filename', { filename })
    selectedCalendarId.value = calendarId
  }

  async function chargerPairesDisponibles() {
    try {
      // Charger le store des conversions pour accéder à l'info 'hidden'
      await conversionStore.loadConversions()
      const data = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
      // Filtrer les paires cachées (supprimées par l'utilisateur)
      availablePairs.value = data
        .map(item => item.symbol)
        .filter(symbol => !conversionStore.isSymbolHidden(symbol))
    } catch {
      availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
    }
  }

  return {
    viewMode,
    availablePairs,
    selectedCalendarId,
    gererSelectionCalendrier,
    chargerPairesDisponibles,
    // Alias pour compatibilité
    handleCalendarSelected: gererSelectionCalendrier,
    loadAvailablePairs: chargerPairesDisponibles
  }
}
