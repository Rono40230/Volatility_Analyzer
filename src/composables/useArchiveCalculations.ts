import type { NormalizedArchive, EventStats, PairStats } from './useArchiveTypes'

/**
 * Extraire et normaliser données heatmap
 */
export function extractHeatmapData(archives: NormalizedArchive[]) {
  const heatmapArchives = archives.filter(a => a.type === 'Heatmap')
  const pairs = [...new Set(heatmapArchives.map(a => a.pair))].sort()
  const events = [...new Set(heatmapArchives.map(a => a.eventType))].sort()
  const impactMatrix: Record<string, Record<string, number>> = {}

  for (const pair of pairs) {
    impactMatrix[pair] = {}
    for (const event of events) {
      const archive = heatmapArchives.find(a => a.pair === pair && a.eventType === event)
      impactMatrix[pair][event] = archive?.impactScore || 0
    }
  }

  return { pairs, events, impactMatrix }
}

/**
 * Calculer paramètres Straddle optimaux
 */
export function calculateOptimalStraddleParams(eventStats: EventStats) {
  const sl = eventStats.avgATR * 1.5
  const tp = sl * 2.0

  return {
    sl: Math.round(sl * 100) / 100,
    tp: Math.round(tp * 100) / 100,
    ratio: '1:2',
    placementSeconds: 60,
    exitMinutes: eventStats.avgDecayTimeout,
    estimatedGain: `${Math.round((tp / sl) * 10) / 10}R`
  }
}

/**
 * Générer conseils dynamiques
 */
export function generateAdvice(
  eventStats: Record<string, EventStats>,
  pairStats: Record<string, PairStats>
): string[] {
  const advice: string[] = []

  const goodEvents = Object.values(eventStats)
    .filter(e => e.tradabilityScore! >= 75)
    .sort((a, b) => (b.tradabilityScore || 0) - (a.tradabilityScore || 0))

  if (goodEvents.length > 0) {
    const top = goodEvents[0]
    const bestPair = Object.entries(pairStats)
      .sort((a, b) => b[1].avgConfidence - a[1].avgConfidence)[0]?.[0]

    if (bestPair) {
      advice.push(`🎯 MEILLEUR: ${top.eventType}/${bestPair} (${top.tradabilityScore}% confiance)`)
    }
  }

  const riskyEvents = Object.values(eventStats)
    .filter(e => e.tradabilityScore! < 50)
    .map(e => `${e.eventType} (${e.tradabilityScore?.toFixed(0)}%)`)

  if (riskyEvents.length > 0) {
    advice.push(`⚠️ ÉVITER: ${riskyEvents.slice(0, 3).join(', ')}`)
  }

  const bestRatio = Object.entries(eventStats)
    .sort((a, b) => (b[1].avgATR / a[1].avgATR) - (a[1].avgATR / b[1].avgATR))[0]

  if (bestRatio) {
    const params = calculateOptimalStraddleParams(bestRatio[1])
    advice.push(`💰 MEILLEUR RATIO: ${bestRatio[0]} (~${params.estimatedGain})`)
  }

  const fastest = Object.entries(eventStats)
    .sort((a, b) => a[1].avgPeakDelay - b[1].avgPeakDelay)[0]

  if (fastest) {
    advice.push(`⚡ PLUS RAPIDE: ${fastest[0]} (T+${fastest[1].avgPeakDelay.toFixed(1)}min)`)
  }

  return advice
}

