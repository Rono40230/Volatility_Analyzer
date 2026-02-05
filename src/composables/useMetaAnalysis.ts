import { computed } from 'vue'
import { useArchiveStore } from '../stores/archiveStore'

export function useMetaAnalysis() {
  const archiveStore = useArchiveStore()

  // Extraction des données
  const parsedData = computed(() => {
    return archiveStore.archives
      .filter(a => a.archive_type === 'RETRO_ANALYSIS' || a.archive_type === 'Métriques Rétrospectives' || a.archive_type === 'Correlation de la volatilité Paire/Evenement')
      .map(a => {
        try {
          const data = JSON.parse(a.data_json)
          return {
            id: a.id,
            title: a.title,
            eventType: data.eventType || 'Inconnu',
            pair: data.pair || 'Inconnu',
            volatilityIncrease: data.volatilityIncreasePercent || 0,
            noiseRatio: data.noiseRatioDuring || 0,
            offset: data.offsetSimultaneous || 0,
            stopLoss: data.stopLossRecoverySimultaneous || 0,
            timeout: data.timeout || 0,
            eventCount: data.eventCount || 0
          }
        } catch (e) {
          return null
        }
      })
      .filter(d => d !== null)
  })

  // 1. Divergence Graph (Scatter Plot)
  const divergenceData = computed(() => {
    const data = parsedData.value
    if (data.length === 0) return []

    const maxVol = Math.max(...data.map(d => d.volatilityIncrease)) || 100
    const maxNoise = Math.max(...data.map(d => d.noiseRatio)) || 5

    return data.map(d => ({
      ...d,
      x: (d.volatilityIncrease / maxVol) * 100, // %
      y: 100 - ((d.noiseRatio / maxNoise) * 100) // Inverted (Low noise is better)
    }))
  })

  // 2. Matrice Rentabilité (Heatmap)
  const matrixData = computed(() => {
    const data = parsedData.value
    const pairs = [...new Set(data.map(d => d.pair))].sort()
    const events = [...new Set(data.map(d => d.eventType))].sort()

    const matrix: Record<string, Record<string, number>> = {}

    events.forEach(event => {
      matrix[event] = {}
      pairs.forEach(pair => {
        const items = data.filter(d => d.eventType === event && d.pair === pair)
        if (items.length > 0) {
          // Score simple : Volatilité / Noise
          const avgVol = items.reduce((sum, d) => sum + d.volatilityIncrease, 0) / items.length
          const avgNoise = items.reduce((sum, d) => sum + d.noiseRatio, 0) / items.length
          matrix[event][pair] = avgVol / (avgNoise || 1)
        } else {
          matrix[event][pair] = 0
        }
      })
    })

    return { pairs, events, matrix }
  })

  // 3. Leaderboard
  const leaderboardData = computed(() => {
    const data = parsedData.value
    const events = [...new Set(data.map(d => d.eventType))]

    return events.map(event => {
      const items = data.filter(d => d.eventType === event)
      const avgVol = items.reduce((sum, d) => sum + d.volatilityIncrease, 0) / items.length
      const avgNoise = items.reduce((sum, d) => sum + d.noiseRatio, 0) / items.length
      const score = avgVol / (avgNoise || 1)

      return {
        event,
        count: items.length,
        avgVol,
        avgNoise,
        score
      }
    }).sort((a, b) => b.score - a.score)
  })

  // 4. Optimiseur
  const optimizerData = computed(() => {
    const data = parsedData.value
    const events = [...new Set(data.map(d => d.eventType))]

    return events.map(event => {
      const items = data.filter(d => d.eventType === event)
      return {
        event,
        avgOffset: items.reduce((sum, d) => sum + d.offset, 0) / items.length,
        avgSL: items.reduce((sum, d) => sum + d.stopLoss, 0) / items.length,
        avgTimeout: items.reduce((sum, d) => sum + d.timeout, 0) / items.length
      }
    })
  })

  function getHeatmapColor(score: number): string {
    if (score === 0) return '#333'
    // Echelle simple : 0 -> 50 (vert)
    const intensity = Math.min(score / 50, 1)
    return `rgba(76, 175, 80, ${intensity})`
  }

  return {
    parsedData,
    divergenceData,
    matrixData,
    leaderboardData,
    optimizerData,
    getHeatmapColor
  }
}
