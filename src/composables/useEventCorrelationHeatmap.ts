import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getEventTranslation } from '../stores/eventTranslations'
import { useAnalysisStore } from '../stores/analysisStore'

export interface HeatmapData {
  pairs: string[]
  event_types: Array<{ name: string; has_data?: boolean }>
  data: Record<string, Record<string, number>>
}

export function useEventCorrelationHeatmap(isArchiveMode = false, archiveData?: HeatmapData) {
  const loadingHeatmap = ref(false)
  const analysisStore = useAnalysisStore()

  const heatmapData = computed(() => {
    if (isArchiveMode) return archiveData || null
    return analysisStore.persistentHeatmapData
  })

  const minVolatilityThreshold = ref(3)
  const maxEventsToDisplay = ref(10)

  async function loadHeatmapData(pairs: string[], calendarId: number | null = null) {
    if (!pairs || pairs.length === 0) {
      return
    }

    // Vérifier si on doit recharger
    if (!analysisStore.shouldReloadHeatmap(pairs, calendarId)) {
      return
    }

    loadingHeatmap.value = true
    try {
      const result = await invoke<HeatmapData>('get_correlation_heatmap', { 
        pairs: pairs,
        calendar_id: calendarId
      })
      // Stocker dans le store pour la persistance
      analysisStore.setPersistentHeatmapData(result, pairs, calendarId)
    } catch (error) {
      analysisStore.setPersistentHeatmapData({ pairs: [], event_types: [], data: {} }, pairs, calendarId)
    } finally {
      loadingHeatmap.value = false
    }
  }

  function getHeatmapValue(eventType: string, pair: string): number {
    if (!heatmapData.value?.data[eventType]) return 0
    return heatmapData.value.data[eventType][pair] || 0
  }

  function getEventAverage(eventType: string): number {
    if (!heatmapData.value?.data[eventType] || !heatmapData.value?.pairs.length) return 0
    const values = Object.values(heatmapData.value.data[eventType]).filter((v) => typeof v === 'number')
    return values.length > 0 ? values.reduce((a: number, b: number) => a + b, 0) / values.length : 0
  }

  const sortedEventTypes = computed(() => {
    if (!heatmapData.value) return []
    let sorted = [...heatmapData.value.event_types].sort((a, b) => getEventAverage(b.name) - getEventAverage(a.name))
    sorted = sorted.slice(0, maxEventsToDisplay.value)
    return sorted
  })

  function getHeatmapClass(value: number): string {
    if (value >= 12) return 'heat-very-low'
    if (value >= 9) return 'heat-low'
    if (value >= 6) return 'heat-medium'
    if (value >= 3) return 'heat-high'
    return 'heat-very-high'
  }

  function getFormattedEventName(eventName: string): string {
    const translation = getEventTranslation(eventName)
    return `${eventName} (${translation.fr}) ${translation.flag}`
  }

  async function forceReloadHeatmap(pairs: string[], calendarId: number | null = null) {
    if (!pairs || pairs.length === 0) {
      return
    }

    loadingHeatmap.value = true
    try {
      const result = await invoke<HeatmapData>('get_correlation_heatmap', { 
        pairs: pairs,
        calendar_id: calendarId
      })
      // Forcer la sauvegarde (remplacer l'ancienne)
      analysisStore.setPersistentHeatmapData(result, pairs, calendarId)
    } catch {
      analysisStore.setPersistentHeatmapData({ pairs: [], event_types: [], data: {} }, pairs, calendarId)
    } finally {
      loadingHeatmap.value = false
    }
  }

  return {
    loadingHeatmap,
    heatmapData,
    minVolatilityThreshold,
    maxEventsToDisplay,
    sortedEventTypes,
    loadHeatmapData,
    forceReloadHeatmap,
    getHeatmapValue,
    getHeatmapClass,
    getFormattedEventName
  }
}
