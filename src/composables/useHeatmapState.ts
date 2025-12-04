// Composable pour gérer l'état de la heatmap et les watchers
import { ref, watch, onMounted, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAnalysisStore } from '../stores/analysisStore'
import { useDataRefresh } from './useDataRefresh'

export function useHeatmapState(props: any) {
  const analysisStore = useAnalysisStore()
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
      selectedCalendarId.value = analysisStore.getStoredHeatmapCalendarId()
    }
    await loadAvailablePairs()
  })

  const { onPairDataRefresh } = useDataRefresh()
  const unsubscribe = onPairDataRefresh(loadAvailablePairs)
  onBeforeUnmount(() => unsubscribe())

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

  async function handleCalendarSelected(filename: string) {
    const calendarId = await invoke<number | null>('get_calendar_id_by_filename', { filename })
    selectedCalendarId.value = calendarId
  }

  async function loadAvailablePairs() {
    try {
      const data = await invoke<Array<{ symbol: string; file_path: string }>>('load_symbols')
      availablePairs.value = data.map(item => item.symbol)
    } catch {
      availablePairs.value = ['EURUSD', 'GBPUSD', 'USDJPY', 'XAUUSD', 'BTCUSD']
    }
  }

  return {
    viewMode,
    availablePairs,
    selectedCalendarId,
    handleCalendarSelected,
  }
}
