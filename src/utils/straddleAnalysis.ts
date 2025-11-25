// Re-exports consolidés pour straddleAnalysis
export { calculateStraddleScore, detectGoldenCombos, detectTraps, calculateTradingPlan, calculateTradeDuration } from './straddleCalculators'
export type { Slice15minWithScore, GoldenCombo, DetectedTrap, TradingPlan, SliceAnalysis, BidiParameters } from './straddleTypes'
import type { Stats15Min } from '../stores/volatility'
import type { Slice15minWithScore, SliceAnalysis, BidiParameters } from './straddleTypes'
import { calculateStraddleScore, calculateTradingPlan, calculateTradeDuration } from './straddleCalculators'

export function findTop3Slices(stats15min: Stats15Min[]): Slice15minWithScore[] {
  return stats15min.map(stat => ({
    hour: stat.hour, quarter: stat.quarter, startTime: formatSliceTime(stat.hour, stat.quarter),
    stats: stat, straddleScore: calculateStraddleScore(stat)
  })).sort((a, b) => b.straddleScore - a.straddleScore).slice(0, 3)
}

export function analyzeTop3Slices(stats15min: Stats15Min[]): SliceAnalysis[] {
  const top3 = findTop3Slices(stats15min)
  return top3.map((slice, idx) => ({
    rank: (idx + 1) as 1|2|3,
    slice,
    combos: [],
    traps: [],
    tradingPlan: calculateTradingPlan(slice.stats, 100000, slice.straddleScore)
  }))
}

export function isInTop3(slice: Slice15minWithScore, top3Slices: Slice15minWithScore[]): boolean {
  return top3Slices.some(s => s.hour === slice.hour && s.quarter === slice.quarter)
}

export function getTop3Rank(slice: Slice15minWithScore, top3Slices: Slice15minWithScore[]): number {
  const idx = top3Slices.findIndex(s => s.hour === slice.hour && s.quarter === slice.quarter)
  return idx === -1 ? 0 : idx + 1
}

export function calculateBidiParameters(bestSlice: Stats15Min, allSlices: Stats15Min[]): BidiParameters {
  const primaryEvent = bestSlice.events?.[0]?.event_name ?? 'AUTRE'
  const tradeDurationMinutes = calculateTradeDuration(bestSlice.atr_mean, primaryEvent, bestSlice.hour)
  
  return {
    entryTime: '—',
    slPips: Math.round((bestSlice.atr_mean * 1.5) * 10000),
    tpPips: Math.round((bestSlice.atr_mean * 2.5) * 10000),
    winRate: 0.55,
    avgGain: 0.35,
    tradeExpiration: tradeDurationMinutes,
    bestHourReliability: 65
  }
}

function formatSliceTime(hour: number, quarter: number): string {
  const h = String(hour).padStart(2, '0')
  const start = String(quarter * 15).padStart(2, '0')
  const end = String(Math.min(quarter * 15 + 15, 60)).padStart(2, '0')
  return `${h}:${start}-${h}:${end}`
}
