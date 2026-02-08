// Helper functions pour straddleCalculators.ts
// Contient les logiques de scoring et calcul détaillés

import type { Stats15Min } from '../stores/volatility'
import type { TradingPlan } from './straddleTypes'

/**
 * Estime le prix basé sur la moyenne ATR et le symbole optionnel.
 * Utilisé uniquement par calculateStraddleScore() pour la normalisation en %.
 * Le backend fournit `point_value` pour les calculs précis (straddle_parameters).
 */
export function estimatePrice(slice: Stats15Min, symbol?: string): number {
  // Par symbole si disponible (le plus fiable)
  if (symbol) {
    if (symbol.includes('JPY')) return 150.0
    if (symbol.includes('XAU') || symbol.includes('GOLD')) return 2000.0
    if (symbol.includes('BTC')) return 60000.0
    if (symbol.includes('NAS') || symbol.includes('US100') || symbol.includes('NDX')) return 18000.0
    if (symbol.includes('US30') || symbol.includes('DJI')) return 38000.0
    if (symbol.includes('SP') || symbol.includes('US500')) return 5000.0
    if (symbol.includes('DAX') || symbol.includes('DE40')) return 18000.0
  }
  // Heuristique ATR (fallback sans symbole)
  if (slice.atr_mean > 1000) return 60000 // Crypto (BTC)
  if (slice.atr_mean > 100) return 18000  // Indices majeurs
  if (slice.atr_mean > 10) return 2000    // Or / Indices petits
  if (slice.atr_mean > 1) return 150.0    // Paires JPY
  return 1.10                              // Forex majors (EUR/USD, GBP/USD...)
}

/**
 * Calcule les métriques de volatilité normalisées
 */
export function calculateVolatilityMetrics(slice: Stats15Min, estimatedPrice: number) {
  const atrPoints = slice.atr_mean
  const atrPercent = (atrPoints / estimatedPrice) * 100
  const rangePercent = (slice.range_mean / estimatedPrice) * 100
  
  return {
    atrPoints,
    atrPercent,
    rangePercent,
    bodyRange: slice.body_range_mean,
    noiseRatio: slice.noise_ratio_mean,
    volumeImbalance: slice.volume_imbalance_mean * 100,
    breakoutPercent: slice.breakout_percentage
  }
}

/**
 * Construit un TradingPlan à partir des straddle_parameters du backend.
 * Source unique de vérité = Rust StraddleParameterService.
 * Fallback sur des valeurs neutres si le backend n'a pas calculé de paramètres.
 */
export function buildTradingPlanFromBackend(slice: Stats15Min, confidenceScore: number): TradingPlan {
  const params = slice.straddle_parameters
  const timeout = params?.timeout_minutes
    ?? slice.recommended_trade_expiration_minutes
    ?? slice.recommended_trade_expiration_mean
    ?? 15

  if (params) {
    return {
      entryTime: '—',
      slPips: params.stop_loss_pips,
      slPoints: params.stop_loss_pips,
      slUsd: 0,
      tpPips: params.hard_tp_pips ?? 0,
      tpPoints: params.hard_tp_pips ?? 0,
      tpUsd: 0,
      tpRatio: params.risk_reward_ratio,
      atrPercentage: 0,
      atrPoints: slice.atr_mean,
      winProbability: Math.round(confidenceScore),
      avgGainR: params.risk_reward_ratio,
      avgLossR: 1.0,
      trailingStopCoefficient: params.trailing_stop_pips,
      recommendation: confidenceScore >= 75 ? 'TRADE' : 'CAUTION',
      confidence: confidenceScore,
      riskLevel: confidenceScore >= 75 ? 'LOW' : confidenceScore >= 50 ? 'MEDIUM' : 'HIGH',
      tradeDurationMinutes: timeout,
      // Champs utilisés par les rapports PDF
      offset: params.offset_pips,
      tp: params.hard_tp_pips ?? 0,
      sl: params.stop_loss_pips,
      duration: timeout
    }
  }

  // Fallback si aucun straddle_parameters (rare : archive ancienne)
  return {
    entryTime: '—',
    slPips: 0, slPoints: 0, slUsd: 0,
    tpPips: 0, tpPoints: 0, tpUsd: 0,
    tpRatio: 0, atrPercentage: 0,
    atrPoints: slice.atr_mean,
    winProbability: Math.round(confidenceScore),
    avgGainR: 0, avgLossR: 0,
    trailingStopCoefficient: 0,
    recommendation: 'CAUTION',
    confidence: confidenceScore,
    riskLevel: 'HIGH',
    tradeDurationMinutes: timeout,
    offset: 0, tp: 0, sl: 0, duration: timeout
  }
}
