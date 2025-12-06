import type { EventStats, PairStats } from './useArchiveTypes'

/**
 * Score de tradabilité (0-100)
 */
export function calculateTradabilityScore(eventStats: EventStats): number {
  const confidenceScore = Math.min(eventStats.avgConfidence, 1) * 100 * 0.4
  const stability = eventStats.variance ? Math.max(0, 1 - (eventStats.variance / eventStats.avgPeakDelay)) : 1
  const stabilityScore = stability * 100 * 0.3
  const impactScore = (eventStats.heatmapImpact || 0) * 0.3
  const total = confidenceScore + stabilityScore + impactScore
  return Math.min(Math.max(total, 0), 100)
}

export function getPairRating(confidence: number): string {
  const conf = Math.min(confidence, 1) * 100
  if (conf >= 80) return '🟢 TRÈS BON'
  if (conf >= 65) return '🟡 BON'
  if (conf >= 50) return '🟠 MOYEN'
  return '🔴 FAIBLE'
}
