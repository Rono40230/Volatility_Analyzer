import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getEventTranslation } from '../stores/eventTranslations'
import { useAnalysisStore } from '../stores/analysisStore'

export interface HeatmapData {
  period_start?: string
  period_end?: string
  pairs: string[]
  event_types: Array<{ name: string; count: number; has_data?: boolean }>
  data: Record<string, Record<string, number>>
  volatility_percentages?: Record<string, Record<string, number>>
  counts?: Record<string, Record<string, number>>
}

export interface ArchivedCell {
  pair: string
  event_type: string
}

export function useEventCorrelationHeatmap(isArchiveMode = false, archiveData?: HeatmapData) {
  const loadingHeatmap = ref(false)
  const analysisStore = useAnalysisStore()
  const archivedCells = ref<Set<string>>(new Set()) // Format: "PAIR|EVENTTYPE"

  const heatmapData = computed<HeatmapData | null>(() => {
    if (isArchiveMode) return archiveData || null
    return analysisStore.persistentHeatmapData
  })

  const minVolatilityThreshold = ref(0)
  const maxEventsToDisplay = ref(50)

  async function loadArchivedCells() {
    try {
      const archived = await invoke<ArchivedCell[]>('get_archived_pairs_events')
      archivedCells.value = new Set(archived.map(item => `${item.pair}|${item.event_type}`))
    } catch (error) {
      console.error('Failed to load archived cells:', error)
    }
  }

  function isArchived(eventType: string, pair: string): boolean {
    return archivedCells.value.has(`${pair}|${eventType}`)
  }

  async function loadHeatmapData(pairs: string[], calendarId: number | null = null) {
    if (!pairs || pairs.length === 0) return

    if (!analysisStore.shouldReloadHeatmap(pairs, calendarId)) return

    loadingHeatmap.value = true
    try {
      const result = await invoke<HeatmapData>('get_correlation_heatmap', { 
        pairs: pairs,
        calendar_id: calendarId
      })
      analysisStore.setPersistentHeatmapData(result, pairs, calendarId)
    } catch (_error) {
      analysisStore.setPersistentHeatmapData({ pairs: [], event_types: [], data: {} }, pairs, calendarId)
    } finally {
      loadingHeatmap.value = false
    }
  }

  function getHeatmapValue(eventType: string, pair: string): number {
    if (!heatmapData.value?.data[eventType]) return 0
    return heatmapData.value.data[eventType][pair] || 0
  }

  function getHeatmapPercentage(eventType: string, pair: string): number {
    if (!heatmapData.value?.volatility_percentages?.[eventType]) {
      console.debug(`[getHeatmapPercentage] No data for ${eventType} / ${pair}`, heatmapData.value?.volatility_percentages)
      return 0
    }
    const val = heatmapData.value.volatility_percentages![eventType][pair] || 0
    console.debug(`[getHeatmapPercentage] ${eventType}/${pair} = ${val}%`)
    return val
  }

  function getHeatmapCount(eventType: string, pair: string): number {
    if (!heatmapData.value?.counts?.[eventType]) return 0
    return heatmapData.value.counts[eventType][pair] ?? 0
  }

  const eventAverages = computed(() => {
    if (!heatmapData.value?.pairs.length) return new Map<string, number>()
    const map = new Map<string, number>()
    for (const eventType of heatmapData.value.event_types) {
      const data = heatmapData.value.data[eventType.name]
      if (!data) {
        map.set(eventType.name, 0)
        continue
      }
      let sum = 0
      let count = 0
      for (const value of Object.values(data)) {
        if (typeof value === 'number') {
          sum += value
          count += 1
        }
      }
      map.set(eventType.name, count > 0 ? sum / count : 0)
    }
    return map
  })

  const sortedEventTypes = computed(() => {
    if (!heatmapData.value) return []
    const averages = eventAverages.value
    const sorted = [...heatmapData.value.event_types].sort((a, b) => {
      return (averages.get(b.name) || 0) - (averages.get(a.name) || 0)
    })
    if (maxEventsToDisplay.value <= 0) return sorted
    return sorted.slice(0, maxEventsToDisplay.value)
  })

  function getHeatmapClass(value: number): string {
    // Score Straddle (0-100) — 5 niveaux de discrimination
    if (value >= 80) return 'heat-extreme'    // Exceptionnel (Vert vif)
    if (value >= 65) return 'heat-very-high'  // Excellent (Vert)
    if (value >= 50) return 'heat-high'       // Bon (Vert clair)
    if (value >= 35) return 'heat-medium'     // Moyen (Orange)
    if (value >= 20) return 'heat-low'        // Faible (Rouge clair)
    return 'heat-very-low'                    // Très faible (Rouge)
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
    
    // Réinitialiser temporairement pour forcer la réactivité et montrer le chargement
    analysisStore.setPersistentHeatmapData({ pairs: [], event_types: [], data: {} }, pairs, calendarId)

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
    getHeatmapPercentage,
    getHeatmapCount,
    getHeatmapClass,
    getFormattedEventName,
    loadArchivedCells,
    isArchived
  }
}
