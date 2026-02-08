// Types et interfaces pour l'analyse STRADDLE
import type { Stats15Min } from '../stores/volatility'

export interface Slice15minWithScore {
  hour: number; quarter: number; startTime: string; stats: Stats15Min; straddleScore: number
}

export interface GoldenCombo {
  name: string; description: string; confidence: 'JACKPOT' | 'EXCELLENT' | 'BON' | 'MOYEN' | 'FAIBLE'; winRate: number; avgGainR: number
}

export interface DetectedTrap {
  name: string; description: string; severity: 'CRITIQUE' | 'HAUTE' | 'MOYENNE' | 'BASSE'; metric: string; value: number; threshold: number; recommendation: string
}

export interface TradingPlan {
  entryTime: string
  slPips: number // Stop Loss en points MetaTrader 5
  slPoints: number // Stop Loss en points MetaTrader 5 (même que slPips)
  slUsd: number // Stop Loss en USD
  tpPips: number // Take Profit en points MetaTrader 5
  tpPoints: number // Take Profit en points MetaTrader 5 (même que tpPips)
  tpUsd: number // Take Profit en USD
  tpRatio: number // Ratio TP/SL
  atrPercentage: number // ATR en % du prix
  atrPoints: number // ATR en points MetaTrader 5
  winProbability: number // Probabilité gain %
  avgGainR: number // Gain moyen en multiples du risque
  avgLossR: number // Perte moyenne en multiples du risque
  tradeExpiration?: number // Durée du trade en minutes
  trailingStopCoefficient: number // Coefficient Trailing Stop en points
  recommendation: string // Recommandation
  confidence: number // Confiance 0-100
  riskLevel: string // Niveau risque
  tradeDurationMinutes?: number // Durée estimée en minutes
  // Champs normalisés utilisés par les rapports PDF
  offset?: number // Offset en pips (depuis backend straddle_parameters)
  tp?: number // Take Profit en pips
  sl?: number // Stop Loss en pips
  duration?: number // Durée timeout en minutes
}

export interface SliceAnalysis {
  rank: 1 | 2 | 3; slice: Slice15minWithScore; combos: GoldenCombo[]; traps: DetectedTrap[]; tradingPlan: TradingPlan
}

