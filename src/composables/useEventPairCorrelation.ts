import { computed } from 'vue'
import { useArchiveStatistics } from './useArchiveStatistics'

/**
 * Composable pour extraire la corrélation événement × paire depuis les archives
 * Combine les données de Heatmap ET Rétrospectives
 */
export function useEventPairCorrelation() {
  const { archives } = useArchiveStatistics()

  /**
   * Récupère les meilleures paires pour un événement donné
   * Combine Heatmap (volatilité) ET Rétrospectives (confiance)
   */
  const getPairsByEvent = computed(() => {
    return (eventType: string) => {
      const pairsForEvent: Record<string, { pair: string; impact: number; confidence: number }> = {}

      // 1. D'abord, ajouter les données Heatmap (volatilité)
      const heatmapArchives = archives.value.filter(a => a.type === 'Heatmap' && a.eventType === eventType)
      for (const archive of heatmapArchives) {
        const key = archive.pair
        if (!pairsForEvent[key]) {
          pairsForEvent[key] = {
            pair: archive.pair,
            impact: archive.impactScore || 0,
            confidence: archive.confidence || 0,
          }
        } else {
          pairsForEvent[key].impact = Math.max(pairsForEvent[key].impact, archive.impactScore || 0)
          pairsForEvent[key].confidence = Math.max(pairsForEvent[key].confidence, archive.confidence || 0)
        }
      }

      // 2. Ensuite, ajouter/compléter avec les données Rétrospectives (confiance)
      const retrospectiveArchives = archives.value.filter(a => a.type === 'Métriques Rétrospectives' && a.eventType === eventType)
      for (const archive of retrospectiveArchives) {
        const key = archive.pair
        if (!pairsForEvent[key]) {
          // Créer une nouvelle entrée si pas de Heatmap
          pairsForEvent[key] = {
            pair: archive.pair,
            impact: 0,
            confidence: archive.confidence || 0,
          }
        } else {
          // Mettre à jour la confiance avec la valeur Rétrospective (plus fiable)
          pairsForEvent[key].confidence = archive.confidence || pairsForEvent[key].confidence
        }
      }

      return Object.values(pairsForEvent).sort((a, b) => {
        // Trier d'abord par impact (Heatmap), puis par confiance (Rétrospectives) en cas d'égalité
        if (b.impact !== a.impact) {
          return b.impact - a.impact
        }
        return b.confidence - a.confidence
      })
    }
  })

  /**
   * Retourne true si on a des données disponibles
   */
  const hasHeatmapData = computed(() => {
    return archives.value.some(a => a.type === 'Heatmap' || a.type === 'Métriques Rétrospectives')
  })

  return {
    getPairsByEvent,
    hasHeatmapData,
  }
}
