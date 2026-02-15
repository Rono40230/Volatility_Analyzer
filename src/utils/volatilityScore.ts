// utils/volatilityScore.ts - Score de volatilité composite pour classement des quarters
// Remplace l'ancien calculateStraddleScore après suppression du module straddle

import type { Stats15Min } from '../stores/volatilityTypes'

/**
 * Analyse d'un créneau (slice) 15 minutes
 * Utilisé par MetricsAnalysisModal, BestSliceCard, MetricsGrid, etc.
 */
export interface SliceAnalysis {
  rank: number
  slice: {
    hour: number
    quarter: number
    startTime: string
    stats: Stats15Min
    straddleScore: number // Score composite de volatilité (0-100)
  }
  combos: unknown[]
  traps: unknown[]
  tradingPlan: TradingPlan | null
}

export interface TradingPlan {
  recommendation: string
  confidence: number
}

/**
 * Calcule un score composite de volatilité pour un créneau 15 minutes.
 * 
 * Le score combine :
 * - ATR (force du mouvement)
 * - Noise Ratio (qualité du mouvement — inversé, plus bas = mieux)
 * - Breakout % (capacité à casser un range)
 * - Body/Range (directionnalité)
 * - Volume Imbalance (déséquilibre acheteur/vendeur)
 * 
 * @param stats - Statistiques du créneau 15min
 * @param movementQualityBonus - Bonus optionnel de qualité de mouvement (0-10)
 * @returns Score composite (0-100)
 */
export function calculateVolatilityScore(stats: Stats15Min, movementQualityBonus = 0): number {
  if (!stats) return 0

  // Composantes du score (pondérées sur 100)
  const atrScore = Math.min((stats.atr_mean || 0) * 2, 30)              // 30 pts max
  const noiseScore = Math.max(0, 20 - (stats.noise_ratio_mean || 3) * 5) // 20 pts max (inversé)
  const breakoutScore = ((stats.breakout_percentage || 0) / 100) * 20    // 20 pts max
  const bodyRangeScore = ((stats.body_range_mean || 0) / 100) * 15       // 15 pts max
  const volumeScore = Math.min(Math.abs(stats.volume_imbalance_mean || 0) * 10, 15) // 15 pts max

  const base = atrScore + noiseScore + breakoutScore + bodyRangeScore + volumeScore
  return Math.min(100, Math.max(0, base + movementQualityBonus))
}

/**
 * Alias de compatibilité — même logique que calculateVolatilityScore.
 * Utilisé par les fichiers qui n'ont pas encore été renommés.
 */
export const calculateStraddleScore = calculateVolatilityScore

/**
 * Construit un plan de trading simplifié depuis les stats backend.
 */
export function buildTradingPlanFromBackend(stats: Stats15Min, score: number): TradingPlan | null {
  if (!stats || score <= 0) return null

  let recommendation: string
  if (score >= 70) {
    recommendation = 'Forte volatilité — Créneau optimal'
  } else if (score >= 50) {
    recommendation = 'Volatilité correcte — Créneau viable'
  } else if (score >= 30) {
    recommendation = 'Volatilité modérée — Prudence'
  } else {
    recommendation = 'Faible volatilité — Éviter'
  }

  return {
    recommendation,
    confidence: Math.min(100, score)
  }
}
