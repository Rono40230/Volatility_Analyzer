import type { NormalizedArchive, EventStats, PairStats, EventPairStats } from './useArchiveTypes'
import { calculateTrailingStop } from './useTrailingStopCalculation'
import { calculateTradabilityScore, getPairRating } from './useArchiveScoring'

/**
 * Score de tradabilité (0-100)
 */
export function calculateEventStatistics(archives: NormalizedArchive[]): Record<string, EventStats> {
  const stats: Record<string, EventStats> = {}

  for (const archive of archives) {
    if (!stats[archive.eventType]) {
      stats[archive.eventType] = {
        eventType: archive.eventType,
        avgATR: 0,
        avgPeakDelay: 0,
        avgDecayTimeout: 0,
        avgConfidence: 0,
        count: 0
      }
    }

    const stat = stats[archive.eventType]
    stat.count++
    stat.avgATR += archive.peakAtr
    stat.avgPeakDelay += archive.peakDelay
    stat.avgDecayTimeout += archive.decayTimeout
    stat.avgConfidence += archive.confidence
  }

  for (const stat of Object.values(stats)) {
    stat.avgATR = stat.count > 0 ? stat.avgATR / stat.count : 0
    stat.avgPeakDelay = stat.count > 0 ? stat.avgPeakDelay / stat.count : 0
    stat.avgDecayTimeout = stat.count > 0 ? stat.avgDecayTimeout / stat.count : 0
    stat.avgConfidence = stat.count > 0 ? stat.avgConfidence / stat.count : 0

    const archivesForEvent = archives.filter(a => a.eventType === stat.eventType)
    if (archivesForEvent.length > 1) {
      const mean = stat.avgPeakDelay
      const squaredDiffs = archivesForEvent.map(a => Math.pow(a.peakDelay - mean, 2))
      const variance = squaredDiffs.reduce((a, b) => a + b, 0) / archivesForEvent.length
      stat.variance = Math.sqrt(variance)
    }

    stat.tradabilityScore = calculateTradabilityScore(stat)

    const heatmapArchives = archivesForEvent.filter(a => a.type === 'Heatmap')
    if (heatmapArchives.length > 0) {
      const impacts = heatmapArchives.map(a => a.impactScore || 0)
      stat.heatmapImpact = impacts.reduce((a, b) => a + b, 0) / impacts.length
    }
  }

  return stats
}

/**
 * Calculer statistiques par paire
 */
export function calculatePairStatistics(archives: NormalizedArchive[]): Record<string, PairStats> {
  const stats: Record<string, PairStats> = {}

  for (const archive of archives) {
    if (!stats[archive.pair]) {
      stats[archive.pair] = {
        pair: archive.pair,
        avgConfidence: 0,
        avgATR: 0,
        count: 0,
        eventSensitivity: {}
      }
    }

    const stat = stats[archive.pair]
    stat.count++
    stat.avgConfidence += archive.confidence
    stat.avgATR += archive.peakAtr

    if (!stat.eventSensitivity[archive.eventType]) {
      stat.eventSensitivity[archive.eventType] = 0
    }
    stat.eventSensitivity[archive.eventType] += archive.confidence
  }

  for (const stat of Object.values(stats)) {
    stat.avgConfidence = stat.count > 0 ? stat.avgConfidence / stat.count : 0
    stat.avgATR = stat.count > 0 ? stat.avgATR / stat.count : 0

    for (const event in stat.eventSensitivity) {
      const eventArchives = archives.filter(a => a.pair === stat.pair && a.eventType === event)
      stat.eventSensitivity[event] = eventArchives.length > 0
        ? stat.eventSensitivity[event] / eventArchives.length
        : 0
    }

    stat.performanceRating = getPairRating(stat.avgConfidence)
  }

  return stats
}

/**
 * Calculer statistiques par [Événement × Paire]
 * Chaque paire a son propre ATR selon l'événement
 */
export function calculateEventPairStatistics(archives: NormalizedArchive[]): Record<string, EventPairStats> {
  const stats: Record<string, EventPairStats> = {}

  for (const archive of archives) {
    const key = `${archive.eventType}|${archive.pair}`
    
    if (!stats[key]) {
      stats[key] = {
        key,
        eventType: archive.eventType,
        pair: archive.pair,
        avgATR: 0,
        avgConfidence: 0,
        count: 0,
        slAdjusted: 0,
        trailingStopCoefficient: 0
      }
    }

    const stat = stats[key]
    stat.count++
    stat.avgATR += archive.peakAtr
    stat.avgConfidence += archive.confidence
  }

  // Moyenne + calcul SL/Trailing Stop
  for (const stat of Object.values(stats)) {
    stat.avgATR = stat.count > 0 ? stat.avgATR / stat.count : 0
    stat.avgConfidence = stat.count > 0 ? stat.avgConfidence / stat.count : 0
    
    // SL = ATR × 1.5
    stat.slAdjusted = Math.round((stat.avgATR * 1.5) * 10) / 10
    
    // Trailing Stop = ATR × 0.75 (50% du SL)
    stat.trailingStopCoefficient = calculateTrailingStop(stat.avgATR)
  }

  return stats
}