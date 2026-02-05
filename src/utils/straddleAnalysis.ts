// Re-exports consolidés pour straddleAnalysis
export { calculateStraddleScore, detectGoldenCombos, detectTraps, calculateTradingPlan, calculateTradeDuration } from './straddleCalculators'
export type { Slice15minWithScore, GoldenCombo, DetectedTrap, TradingPlan, SliceAnalysis } from './straddleTypes'
import type { Stats15Min } from '../stores/volatility'
import type { Slice15minWithScore, SliceAnalysis } from './straddleTypes'
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

export function obtenirRangTop3(slice: Slice15minWithScore, top3Slices: Slice15minWithScore[]): number {
  const idx = top3Slices.findIndex(s => s.hour === slice.hour && s.quarter === slice.quarter)
  return idx === -1 ? 0 : idx + 1
}


function formatSliceTime(hour: number, quarter: number): string {
  const startMin = quarter * 15
  const endMin = startMin + 15
  
  if (endMin >= 60) {
    const endHour = (hour + 1) % 24
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(endHour).padStart(2, '0')}:00`
  } else {
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(hour).padStart(2, '0')}:${String(endMin).padStart(2, '0')}`
  }
}
