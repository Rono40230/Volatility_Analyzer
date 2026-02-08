// Utilitaires pour HourlyTable
import type { Stats15Min } from '../stores/volatility'

export function formatATR(atr: number, price: number): string {
  const atrPercent = (atr / price) * 100
  return `${atrPercent.toFixed(2)}%`
}

export function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}

export function isTradeExpTooLong(slice: Stats15Min): boolean {
  return (slice.recommended_trade_expiration_minutes ?? 0) > 150
}
