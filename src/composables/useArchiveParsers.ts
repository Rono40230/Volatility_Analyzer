import type { NormalizedArchive, RawArchive } from './useArchiveTypes'

/**
 * Parser une archive selon son type
 */
export function parseArchiveByType(raw: RawArchive): NormalizedArchive | NormalizedArchive[] | null {
  const type = raw.archive_type

  if (type === 'Volatilité' || type === 'Volatilité brute') {
    return parseVolatilityArchive(raw)
  } else if (type === 'Métriques Rétrospectives') {
    return parseRetrospectiveArchive(raw)
  } else if (type === 'Heatmap') {
    return parseHeatmapArchive(raw)
  }

  return null
}

/**
 * Parser type "Volatilité" (4 archives)
 */
function parseVolatilityArchive(raw: RawArchive): NormalizedArchive | null {
  try {
    const data = JSON.parse(raw.data_json)
    const symbol = data.symbol || data.analysisResult?.symbol

    if (!symbol) return null

    const meanVol = data.global_metrics?.mean_volatility || data.analysisResult?.global_metrics?.mean_volatility || 0

    return {
      id: String(raw.id),
      type: 'Volatilité',
      pair: symbol,
      eventType: 'Non spécifié',
      peakAtr: meanVol * 1.5 || 20,
      peakDelay: 3,
      decayTimeout: 18,
      confidence: data.confidence_score || data.analysisResult?.confidence_score || 0.5,
      timestamp: raw.created_at
    }
  } catch {
    return null
  }
}

/**
 * Parser type "Métriques Rétrospectives" (20 archives)
 * Utilise directement les ATR en points MetaTrader 5 (pas de conversion)
 */
function parseRetrospectiveArchive(raw: RawArchive): NormalizedArchive | null {
  try {
    const data = JSON.parse(raw.data_json)
    
    // Les archives Métriques Rétrospectives ont directement les champs au top level
    const pair = data.pair || data.symbol || ''

    if (!pair) {
      // Fallback: extraire depuis le titre
      const titleMatch = raw.title.match(/([A-Z]{6})/)
      if (!titleMatch) return null
    }

    // ATR déjà en points MetaTrader 5 - pas de conversion
    const peakAtrPoints = data.peakDelayResults?.peak_atr || 0.004

    const peakDelay = data.peakDelayResults?.peak_delay_minutes || 3.2
    const decayTimeout = data.decayResults?.recommended_timeout_minutes || 18.5
    const confidence = ((data.peakDelayResults?.confidence || 0.75) * 100)

    let eventType = data.eventType || 'Non spécifié'

    const finalPair = pair || (raw.title.match(/([A-Z]{6})/)?.[1]) || 'UNKNOWN'

    return {
      id: String(raw.id),
      type: 'Métriques Rétrospectives',
      pair: finalPair,
      eventType,
      peakAtr: peakAtrPoints,
      peakDelay,
      decayTimeout,
      confidence,
      eventCount: data.peakDelayResults?.event_count || 1,
      timestamp: raw.created_at
    }
  } catch {
    return null
  }
}

/**
 * Parser type "Heatmap" (1 archive)
 * Volatilité déjà en points MetaTrader 5
 * Structure réelle sauvegardée:
 * {
 *   heatmapData: {
 *     pairs: ["ADAUSD", "BTCUSD", ...],
 *     event_types: [{ name: "NFP", count: N, has_data: true }, ...],
 *     data: {
 *       "EventName": { "ADAUSD": 12.1, "BTCUSD": 74.5, ... },
 *       ...
 *     }
 *   }
 * }
 */
function parseHeatmapArchive(raw: RawArchive): NormalizedArchive[] {
  const results: NormalizedArchive[] = []

  try {
    const data = JSON.parse(raw.data_json)
    const heatmapData = data.heatmapData || {}
    const pairs = heatmapData.pairs || []
    const eventTypes = heatmapData.event_types || []
    const volatilityMatrix = heatmapData.data || {}

    // Construire liste d'événements depuis event_types
    const events = eventTypes.map((et: any) => et.name || String(et))

    for (const eventType of events) {
      const eventData = volatilityMatrix[eventType] || {}

      for (const pair of pairs) {
        const volatilityValue = eventData[pair] || 0

        // volatilityValue est déjà en points MetaTrader 5 (ex: 12.1, 74.5)
        results.push({
          id: `${raw.id}-${eventType}-${pair}`,
          type: 'Heatmap',
          pair,
          eventType,
          peakAtr: volatilityValue || 30,
          peakDelay: 3.2,
          decayTimeout: 18,
          confidence: Math.min(100, (volatilityValue / 100) * 80 + 20),
          impactScore: volatilityValue,
          timestamp: raw.created_at
        })
      }
    }
  } catch {
    // Retourner tableau vide si parse échoue
  }

  return results
}
