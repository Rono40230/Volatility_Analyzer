// Calculateurs spécialisés pour l'analyse STRADDLE
import type { Stats15Min } from '../stores/volatility'
import type { GoldenCombo, DetectedTrap } from './straddleTypes'
import {
  estimatePrice,
  calculateVolatilityMetrics,
} from './straddleCalculators.helpers'

export function calculateStraddleScore(slice: Stats15Min, movementQualityScore?: number): number {
  if (slice.candle_count === 0) return 0
  
  const estimatedPriceValue = estimatePrice(slice)
  const metrics = calculateVolatilityMetrics(slice, estimatedPriceValue)
  const atrPercent = metrics.atrPercent
  const rangePercent = metrics.rangePercent
  const bodyRange = metrics.bodyRange
  const noiseRatio = metrics.noiseRatio
  const volumeImbalance = metrics.volumeImbalance
  const breakoutPercent = metrics.breakoutPercent
  
  let score = 0
  
  // Scoring Range (en %)
  if (rangePercent > 2.5) score += 60
  else if (rangePercent > 2.0) score += 50
  else if (rangePercent > 1.5) score += 40
  else if (rangePercent > 1.0) score += 20
  
  // Scoring ATR (en %)
  if (atrPercent > 2.0) score += 25
  else if (atrPercent > 1.5) score += 20
  else if (atrPercent > 1.0) score += 15
  else if (atrPercent > 0.5) score += 8
  
  // Scoring Body Range (pureté du signal)
  if (bodyRange > 45.0) score += 15
  else if (bodyRange > 35.0) score += 12
  else if (bodyRange > 25.0) score += 8
  else if (bodyRange > 15.0) score += 3
  
  // Scoring Noise Ratio (moins c'est bruité, mieux c'est)
  if (noiseRatio < 1.5) score += 10
  else if (noiseRatio < 2.0) score += 8
  else if (noiseRatio < 2.5) score += 5
  
  // Scoring Volume Imbalance (direction strength)
  if (volumeImbalance > 20) score += 10
  else if (volumeImbalance > 15) score += 7
  else if (volumeImbalance > 10) score += 5
  
  // Scoring Breakout
  if (breakoutPercent > 20) score += 5
  else if (breakoutPercent > 15) score += 3
  
  // Pondération qualité du mouvement (optionnel, poids 20%)
  if (movementQualityScore !== undefined) {
    const movementWeight = Math.min(100, Math.max(0, movementQualityScore)) * 0.2
    score = (score * 0.8) + movementWeight
  }
  
  return Math.min(score, 100)
}

export function detectGoldenCombos(slice: Stats15Min): GoldenCombo[] {
  const combos: GoldenCombo[] = []
  const price = estimatePrice(slice)
  const rangePercent = (slice.range_mean / price) * 100
  const atrPercent = (slice.atr_mean / price) * 100

  if (rangePercent > 1.5 && slice.body_range_mean > 40 && slice.noise_ratio_mean < 2.0) {
    combos.push({
      name: 'Range Pur',
      description: 'Movement directionnel avec range excellent et peu de bruit',
      confidence: 'EXCELLENT',
      winRate: 0.65,
      avgGainR: 0.45
    })
  }
  if (atrPercent > 1.0 && slice.volume_imbalance_mean > 0.2 && slice.breakout_percentage > 15) {
    combos.push({
      name: 'Volatilité Haute + Imbalance',
      description: 'Conditions haute volatilité avec imbalance volume confirmée',
      confidence: 'BON',
      winRate: 0.58,
      avgGainR: 0.35
    })
  }
  return combos
}

export function detectTraps(slice: Stats15Min): DetectedTrap[] {
  const traps: DetectedTrap[] = []
  const price = estimatePrice(slice)
  const rangePercent = (slice.range_mean / price) * 100

  if (slice.noise_ratio_mean > 3.0) {
    traps.push({
      name: 'Bruit Excessif',
      description: 'Ratio bruit/signal trop élevé, movements chaotiques',
      severity: 'HAUTE',
      metric: 'Noise Ratio',
      value: slice.noise_ratio_mean,
      threshold: 2.5,
      recommendation: 'Augmenter SL de 20-30%, réduire position size'
    })
  }
  if (rangePercent < 0.5) {
    traps.push({
      name: 'Range Insuffisant',
      description: 'Marché trop calme, movement insuffisant pour straddle',
      severity: 'CRITIQUE',
      metric: 'Range %',
      value: rangePercent,
      threshold: 0.5,
      recommendation: 'SKIP ce créneau, pas d\'opportunity'
    })
  }
  return traps
}


