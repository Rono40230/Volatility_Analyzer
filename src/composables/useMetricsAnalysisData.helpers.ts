import { invoke } from '@tauri-apps/api/core'
import type { Stats15Min } from '../stores/volatility'
import { calculateStraddleScore, calculateTradingPlan } from '../utils/straddleAnalysis'
import type { SliceAnalysis } from '../utils/straddleAnalysis'

export interface VolatilityDuration {
  peak_duration_minutes: number
  volatility_half_life_minutes: number
  recommended_trade_expiration_minutes: number
  confidence_score: number
  sample_size: number
}

export interface MovementQuality {
  id?: number | null
  symbol: string
  event_type: string
  score?: number
  label?: string
  quality_score?: number
  quality_label?: string
  trend_score?: number
  smoothness_score?: number
  candle_consistency?: number
  directional_move_rate?: number
  whipsaw_rate?: number
  avg_pips_moved?: number
  success_rate?: number
  sample_size?: number
  created_at?: number
  updated_at?: number
}

export function formatSliceTime(hour: number, quarter: number): string {
  const startMin = quarter * 15
  const endMin = startMin + 15
  
  if (endMin >= 60) {
    const endHour = (hour + 1) % 24
    return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(endHour).padStart(2, '0')}:00`
  }
  return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(hour).padStart(2, '0')}:${String(endMin).padStart(2, '0')}`
}

export async function loadMovementQuality(
  symbol: string,
  hour: number,
  quarter: number
): Promise<{ score?: number; qualities: Record<string, MovementQuality> }> {
  try {
    const realMetrics = await invoke<any>('analyze_slice_metrics', { symbol, hour, quarter })
    if (realMetrics) {
      return {
        score: realMetrics.movement_quality_score,
        qualities: {
          [hour + '-' + quarter]: {
            score: realMetrics.movement_quality_score,
            label: realMetrics.movement_quality_label,
            symbol,
            event_type: 'N/A'
          }
        }
      }
    }
  } catch (error) {
    // Ignore
  }
  return { qualities: {} }
}

export async function loadVolatilityDuration(
  symbol: string,
  hour: number,
  quarter: number
): Promise<VolatilityDuration | null> {
  try {
    const result = await invoke<any>('analyze_volatility_duration_for_slice', { symbol, hour, quarter })
    if (result) {
      return {
        peak_duration_minutes: result.peak_duration_minutes,
        volatility_half_life_minutes: result.volatility_half_life_minutes,
        recommended_trade_expiration_minutes: result.recommended_trade_expiration_minutes,
        confidence_score: result.confidence_score,
        sample_size: result.sample_size
      }
    }
  } catch (error) {
    // Ignore
  }
  return null
}

export function createBestSlice(
  bestSliceStats: Stats15Min,
  hour: number,
  quarter: number,
  movementQualityScore?: number
): SliceAnalysis {
  const finalScore = calculateStraddleScore(bestSliceStats, movementQualityScore)
  return {
    rank: 1,
    slice: {
      hour: bestSliceStats.hour,
      quarter: bestSliceStats.quarter,
      startTime: formatSliceTime(hour, quarter),
      stats: bestSliceStats,
      straddleScore: finalScore
    },
    combos: [],
    traps: [],
    tradingPlan: calculateTradingPlan(bestSliceStats, 100000, finalScore)
  }
}
