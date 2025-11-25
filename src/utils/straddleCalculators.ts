// Calculateurs spécialisés pour l'analyse STRADDLE
import type { Stats15Min } from '../stores/volatility'
import type { GoldenCombo, DetectedTrap, TradingPlan } from './straddleTypes'

/**
 * Calcule la durée optimale du trade basée sur ATR, type d'événement et heure du jour
 */
export function calculateTradeDuration(
  atrMean: number,
  eventType: string = 'AUTRE',
  hourOfDay: number = 12
): number {
  // Base duration from ATR (en points)
  let baseDuration = 240 // 4h default
  if (atrMean > 0.005) baseDuration = 120       // ATR > 50 pips
  else if (atrMean > 0.004) baseDuration = 150  // ATR > 40 pips
  else if (atrMean > 0.0025) baseDuration = 180 // ATR > 25 pips
  
  // Adjust for event type
  const eventFactors: Record<string, number> = {
    'nfp': 0.5,           // Pic court, intense
    'cpi': 0.7,           // Pic moyen
    'interest rate': 0.8, // Pic long
    'gdp': 1.0,           // Pic très long
    'jobless claims': 0.6,
    'pce': 0.7,
    'autre': 1.0          // Default
  }
  const normalizedEventType = eventType.toLowerCase()
  let eventFactor = 1.0
  for (const [key, factor] of Object.entries(eventFactors)) {
    if (normalizedEventType.includes(key)) {
      eventFactor = factor
      break
    }
  }
  
  // Adjust for hour of day
  const hourFactors: Record<number, number> = {
    8: 0.8,   // London open - pic court
    13: 0.6,  // NY open - pic très court
    14: 0.7,  // Overlap - pic court
    // Autres heures: 1.0 (normal)
  }
  const hourFactor = hourFactors[hourOfDay] ?? 1.0
  
  return Math.round(baseDuration * eventFactor * hourFactor)
}

export function calculateStraddleScore(slice: Stats15Min): number {
  if (slice.candle_count === 0) return 0
  let score = 0
  if (slice.range_mean > 0.0025) score += 60
  else if (slice.range_mean > 0.0020) score += 50
  else if (slice.range_mean > 0.0015) score += 40
  else if (slice.range_mean > 0.0010) score += 20
  if (slice.atr_mean > 0.0020) score += 25
  else if (slice.atr_mean > 0.0015) score += 20
  else if (slice.atr_mean > 0.0010) score += 15
  else if (slice.atr_mean > 0.0005) score += 8
  if (slice.body_range_mean > 45.0) score += 15
  else if (slice.body_range_mean > 35.0) score += 12
  else if (slice.body_range_mean > 25.0) score += 8
  else if (slice.body_range_mean > 15.0) score += 3
  return Math.min(score, 100)
}

export function detectGoldenCombos(slice: Stats15Min): GoldenCombo[] {
  const combos: GoldenCombo[] = []
  if (slice.range_mean > 0.0025 && slice.body_range_mean > 40 && slice.noise_ratio_mean < 2.0) {
    combos.push({
      name: 'Range Pur',
      description: 'Movement directionnel avec range excellent et peu de bruit',
      confidence: 'EXCELLENT',
      winRate: 0.65,
      avgGainR: 0.45
    })
  }
  if (slice.atr_mean > 0.002 && slice.volume_imbalance_mean > 0.2 && slice.breakout_percentage > 15) {
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
  if (slice.range_mean < 0.0010) {
    traps.push({
      name: 'Range Insuffisant',
      description: 'Marché trop calme, movement insuffisant pour straddle',
      severity: 'CRITIQUE',
      metric: 'Range',
      value: slice.range_mean,
      threshold: 0.0015,
      recommendation: 'SKIP ce créneau, pas d\'opportunity'
    })
  }
  return traps
}

export function calculateTradingPlan(slice: Stats15Min, estimatedPrice: number, confidenceScore: number): TradingPlan {
  const atrPoints = slice.atr_mean
  const rangePoints = slice.range_mean
  const slPips = Math.round((atrPoints * 1.5) * 10000)
  const tpPips = Math.round((atrPoints * 2.5) * 10000)
  const winProb = confidenceScore / 100
  const avgGainPips = tpPips * winProb
  const avgLossPips = slPips * (1 - winProb)
  
  // Déterminer le type d'événement principal de cette heure
  const primaryEvent = slice.events?.[0]?.event_name ?? 'AUTRE'
  
  // Calculer la durée optimale du trade
  const tradeDurationMinutes = calculateTradeDuration(atrPoints, primaryEvent, slice.hour)
  
  return {
    entryTime: '—',
    slPips,
    slPoints: atrPoints * 1.5,
    slUsd: Math.round((atrPoints * 1.5) * estimatedPrice),
    tpPips,
    tpPoints: atrPoints * 2.5,
    tpUsd: Math.round((atrPoints * 2.5) * estimatedPrice),
    tpRatio: tpPips / slPips,
    atrPercentage: (atrPoints / estimatedPrice) * 100,
    atrPoints,
    winProbability: Math.round(winProb * 100),
    avgGainR: (avgGainPips - avgLossPips) / slPips,
    avgLossR: Math.max(0, avgLossPips / slPips),
    trailingStopCoefficient: 1.5 + (Math.max(0, atrPoints - 0.0015) * 500),
    recommendation: confidenceScore >= 75 ? 'TRADE' : 'CAUTION',
    confidence: confidenceScore,
    riskLevel: confidenceScore >= 75 ? 'LOW' : confidenceScore >= 50 ? 'MEDIUM' : 'HIGH',
    tradeDurationMinutes
  }
}
