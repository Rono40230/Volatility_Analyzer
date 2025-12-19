import type { NormalizedArchive, RawArchive } from './useArchiveTypes'

export function parseArchiveByType(raw: RawArchive): NormalizedArchive | NormalizedArchive[] | null {
  switch (raw.archive_type) {
    case 'Volatilité':
    case 'Volatilité brute':
      return parseVolatilityArchive(raw)
    case 'Métriques Rétrospectives':
      return parseRetrospectiveArchive(raw)
    case 'Heatmap':
      return parseHeatmapArchive(raw)
    default:
      return null
  }
}

function parseVolatilityArchive(raw: RawArchive): NormalizedArchive | null {
  try {
    const data = JSON.parse(raw.data_json)
    const symbol = data.symbol || data.analysisResult?.symbol
    if (!symbol) return null

    const metrics = data.global_metrics || data.analysisResult?.global_metrics
    return {
      id: String(raw.id),
      type: 'Volatilité',
      pair: symbol,
      eventType: 'Non spécifié',
      peakAtr: metrics?.mean_atr || 20,
      peakDelay: 3,
      decayTimeout: 18,
      confidence: data.confidence_score || data.analysisResult?.confidence_score || 0.5,
      timestamp: raw.created_at,
      unit: data.unit || data.analysisResult?.unit || 'pts'
    }
  } catch { return null }
}

function parseRetrospectiveArchive(raw: RawArchive): NormalizedArchive | null {
  try {
    const data = JSON.parse(raw.data_json)
    const pair = data.pair || data.symbol || (raw.title.match(/([A-Z]{6})/) || [])[1]
    if (!pair) return null

    return {
      id: String(raw.id),
      type: 'Métriques Rétrospectives',
      pair,
      eventType: data.eventType || 'Non spécifié',
      peakAtr: data.peakDelayResults?.peak_atr || 0.004,
      peakDelay: data.peakDelayResults?.peak_delay_minutes || 3.2,
      decayTimeout: data.decayResults?.recommended_timeout_minutes || 18.5,
      confidence: (data.peakDelayResults?.confidence || 0.75) * 100,
      eventCount: data.peakDelayResults?.event_count || 1,
      timestamp: raw.created_at,
      unit: data.unit || 'pts'
    }
  } catch { return null }
}

function parseHeatmapArchive(raw: RawArchive): NormalizedArchive[] {
  try {
    const { heatmapData = {} } = JSON.parse(raw.data_json)
    const { pairs = [], event_types = [], data: volatilityMatrix = {} } = heatmapData
    const events = event_types.map((et: any) => et.name || String(et))
    const results: NormalizedArchive[] = []

    for (const eventType of events) {
      const eventData = volatilityMatrix[eventType] || {}
      for (const pair of pairs) {
        const val = eventData[pair] || 0
        results.push({
          id: `${raw.id}-${eventType}-${pair}`,
          type: 'Heatmap',
          pair,
          eventType,
          peakAtr: val || 30,
          peakDelay: 3.2,
          decayTimeout: 18,
          confidence: Math.min(100, (val / 100) * 80 + 20),
          impactScore: val,
          timestamp: raw.created_at
        })
      }
    }
    return results
  } catch { return [] }
}
