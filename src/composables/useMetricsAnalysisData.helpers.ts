import { invoke } from '@tauri-apps/api/core'
import type { Stats15Min } from '../stores/volatilityTypes'
import { calculateStraddleScore } from '../utils/straddleAnalysis'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import { buildTradingPlanFromBackend } from '../utils/straddleCalculators.helpers'
export { obtenirProfilVolatiliteAssemble } from '../utils/volatilityProfile'
export type { VolatilityDuration, MovementQuality, RecurringEvent } from './metricsAnalysisTypes'

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

export async function loadEntryWindowAnalysis(
  symbol: string,
  hour: number,
  quarter: number
): Promise<{ optimal_offset: number; optimal_win_rate: number; optimal_entry_minutes: number } | null> {
  try {
    const result = await invoke<any>('analyze_quarter_entry_timing', { 
      symbol, 
      hour: Math.floor(hour),
      quarter: Math.floor(quarter)
    })
    
    if (result) {
      return {
        optimal_offset: 0, // Deprecated: was holding time erroneously
        optimal_win_rate: result.optimal_win_rate ?? 0,
        optimal_entry_minutes: result.optimal_offset_minutes ?? 0 // Correct mapping: Time in minutes
      }
    }
  } catch (error) {
    // Erreur silencieuse (fallback à valeurs par défaut)
  }
  return null
}

export function extractVolatilityDuration(bestSliceStats: Stats15Min): VolatilityDuration | null {
  if (!bestSliceStats) return null

  // Récupère les valeurs du créneau élu (moyennes historiques)
  const peak = bestSliceStats.peak_duration_mean ?? 0
  const halfLife = bestSliceStats.volatility_half_life_mean ?? 0
  const tradeDuration = bestSliceStats.recommended_trade_expiration_mean ?? 0

  if (peak === 0 && halfLife === 0 && tradeDuration === 0) {
    return null
  }

  return {
    peak_duration_minutes: peak,
    volatility_half_life_minutes: halfLife,
    recommended_trade_expiration_minutes: tradeDuration,
    confidence_score: 100, // Valeurs du tableau = 100% confiance
    sample_size: 1
  }
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
    tradingPlan: buildTradingPlanFromBackend(bestSliceStats, finalScore)
  }
}

export async function loadQuarterEvents(
  symbol: string,
  hour: number,
  quarter: number
): Promise<RecurringEvent[]> {
  try {
    const events = await invoke<RecurringEvent[]>('get_quarter_events', { symbol, hour, quarter })
    return events
  } catch (e) {
    // Silent fail is acceptable here as it's auxiliary data
    return []
  }
}

