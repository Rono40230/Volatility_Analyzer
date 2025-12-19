// Utilitaires pour HourlyTable
import type { GlobalMetrics, Stats15Min } from '../stores/volatility'

export function getEstimatedPrice(globalMetrics?: GlobalMetrics): number {
  if (!globalMetrics) return 100000
  const atr = globalMetrics.mean_atr
  if (atr > 1000) return 100000
  if (atr > 10) return 10000
  return 1.0
}

export function formatATR(atr: number, price: number): string {
  const atrPercent = (atr / price) * 100
  return `${atrPercent.toFixed(2)}%`
}

export function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}


export function calculateSliceScore(slice: Stats15Min): number {
  if (slice.candle_count === 0) return 0
  let score = 0

  if (slice.range_mean > 0.0025) {
    score += 60
  } else if (slice.range_mean > 0.0020) {
    score += 50
  } else if (slice.range_mean > 0.0015) {
    score += 40
  } else if (slice.range_mean > 0.0010) {
    score += 20
  }

  if (slice.atr_mean > 0.0020) {
    score += 25
  } else if (slice.atr_mean > 0.0015) {
    score += 20
  } else if (slice.atr_mean > 0.0010) {
    score += 15
  } else if (slice.atr_mean > 0.0005) {
    score += 8
  }

  if (slice.body_range_mean > 45.0) {
    score += 15
  } else if (slice.body_range_mean > 35.0) {
    score += 12
  } else if (slice.body_range_mean > 25.0) {
    score += 8
  } else if (slice.body_range_mean > 15.0) {
    score += 3
  }

  return Math.min(score, 100)
}

export function isTradeExpTooLong(slice: Stats15Min): boolean {
  return (slice.recommended_trade_expiration_minutes ?? 0) > 150
}
