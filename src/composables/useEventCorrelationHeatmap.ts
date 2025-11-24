import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useEventTranslation } from './useEventTranslation'

interface HeatmapData {
  pairs: string[]
  event_types: Array<{ name: string; has_data?: boolean }>
  data: Record<string, Record<string, number>>
}

export function useEventCorrelationHeatmap(availablePairs: string[] = [], isArchiveMode = false, archiveData?: any) {
  const { getEventTranslation } = useEventTranslation()
  const loadingHeatmap = ref(false)
  const heatmapData = ref<HeatmapData | null>(null)
  const minVolatilityThreshold = ref(3)
  const maxEventsToDisplay = ref(10)

  async function loadHeatmapData() {
    loadingHeatmap.value = true
    try {
      const result = await invoke<HeatmapData>('get_event_pair_heatmap', { monthsBack: 12 })
      heatmapData.value = result
    } catch (error) {
      console.error('Erreur heatmap:', error)
      heatmapData.value = { pairs: [], event_types: [], data: {} }
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

  return {
    loadingHeatmap,
    heatmapData,
    minVolatilityThreshold,
    maxEventsToDisplay,
    sortedEventTypes,
    loadHeatmapData,
    getHeatmapValue,
    getHeatmapClass,
    getFormattedEventName
  }
}
