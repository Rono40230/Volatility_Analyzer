import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { parseArchiveByType } from './useArchiveParsers'
import { calculateEventStatistics, calculatePairStatistics, calculateEventPairStatistics } from './useArchiveMetrics'
import { extractHeatmapData, generateAdvice } from './useArchiveCalculations'
import { calculateTrailingStop } from './useTrailingStopCalculation'
import { NormalizedArchive, EventStats, PairStats, EventPairStats } from './useArchiveTypes'

export { calculateTrailingStop }
export { NormalizedArchive, EventStats, PairStats, EventPairStats } from './useArchiveTypes'

export interface RawArchive {
  id: number
  title: string
  archive_type: string
  period_start: string
  period_end: string
  comment?: string
  created_at: string
  data_json: string
}

// ============================================================================
// COMPOSABLE - Instance globale partagée
// ============================================================================

let sharedInstance: ReturnType<typeof createArchiveStatistics> | null = null

function createArchiveStatistics() {
  const archives = ref<NormalizedArchive[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadAllArchives() {
    loading.value = true
    error.value = null
    archives.value = []

    try {
      const rawArchives = await invoke<RawArchive[]>('list_all_archives')
      
      for (const raw of rawArchives) {
        try {
          const normalized = parseArchiveByType(raw)
          if (normalized) {
            if (Array.isArray(normalized)) {
              archives.value.push(...normalized)
            } else {
              archives.value.push(normalized)
            }
          }
        } catch {
          // Archive non parsable, ignorer silencieusement
        }
      }
    } catch (e) {
      error.value = `Erreur lors du chargement des archives: ${String(e)}`
    } finally {
      loading.value = false
    }
  }

  const archivesByEvent = computed(() => {
    const grouped: Record<string, NormalizedArchive[]> = {}
    for (const archive of archives.value) {
      if (!grouped[archive.eventType]) grouped[archive.eventType] = []
      grouped[archive.eventType].push(archive)
    }
    return grouped
  })

  const archivesByPair = computed(() => {
    const grouped: Record<string, NormalizedArchive[]> = {}
    for (const archive of archives.value) {
      if (!grouped[archive.pair]) grouped[archive.pair] = []
      grouped[archive.pair].push(archive)
    }
    return grouped
  })

  const eventStatistics = computed(() => calculateEventStatistics(archives.value))
  const pairStatistics = computed(() => calculatePairStatistics(archives.value))
  const eventPairStatistics = computed(() => calculateEventPairStatistics(archives.value))
  const heatmapData = computed(() => extractHeatmapData(archives.value))
  const dynamicAdvice = computed(() => generateAdvice(eventStatistics.value, pairStatistics.value))

  const globalStats = computed(() => ({
    totalArchives: archives.value.length,
    totalEvents: Object.keys(eventStatistics.value).length,
    totalPairs: Object.keys(pairStatistics.value).length,
    avgConfidence: archives.value.length > 0
      ? (archives.value.reduce((sum, a) => sum + a.confidence, 0) / archives.value.length).toFixed(0)
      : '0',
    estimatedWinRate: archives.value.length > 0
      ? (Object.values(eventStatistics.value).reduce((sum, e) => sum + (e.tradabilityScore || 0), 0) / Object.keys(eventStatistics.value).length).toFixed(0)
      : '0'
  }))

  return {
    archives: computed(() => archives.value),
    loading: computed(() => loading.value),
    error: computed(() => error.value),
    loadAllArchives,
    archivesByEvent,
    archivesByPair,
    eventStatistics,
    pairStatistics,
    eventPairStatistics,
    heatmapData,
    dynamicAdvice,
    globalStats
  }
}

export function useArchiveStatistics() {
  if (!sharedInstance) {
    sharedInstance = createArchiveStatistics()
    // Auto-load au premier appel du composable
    sharedInstance.loadAllArchives()
  }
  return sharedInstance
}
