import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AnalysisResult } from '../stores/volatility'
import type { SliceAnalysis } from '../utils/straddleAnalysis'
import { analyzeTop3Slices, calculateBidiParameters, calculateStraddleScore, calculateTradingPlan } from '../utils/straddleAnalysis'

interface MovementQuality {
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

export interface VolatilityDuration {
  peak_duration_minutes: number
  volatility_half_life_minutes: number
  recommended_trade_expiration_minutes: number
  confidence_score: number
  sample_size: number
}

export function useMetricsAnalysisData() {
  const analysisData = ref<any>(null)
  const sliceAnalyses = ref<SliceAnalysis[] | null>(null)
  const movementQualities = ref<Record<string, MovementQuality>>({})
  const volatilityDuration = ref<VolatilityDuration | null>(null)
  const tradingPlan = ref<any>(null)
  const entryWindowAnalysis = ref<any>({ optimal_offset: 0, optimal_win_rate: 0 })

  async function updateAnalysis(analysisResult: AnalysisResult) {
    sliceAnalyses.value = []
    movementQualities.value = {}
    volatilityDuration.value = null
    tradingPlan.value = null
    entryWindowAnalysis.value = { optimal_offset: 0, optimal_win_rate: 0 }

    const result = analysisResult
    const [bestHour, bestQuarter] = result.best_quarter
    analysisData.value = {
      globalMetrics: result.global_metrics,
      symbol: result.symbol,
      confidence: Math.round(result.confidence_score),
      strategy: 'SCALPING STANDARD',
      bestHours: `${bestHour}:${bestQuarter * 15}-${bestHour}:${(bestQuarter + 1) * 15}`
    }

    if (result.stats_15min && result.stats_15min.length > 0) {
      const slices = analyzeTop3Slices(result.stats_15min)

      for (const s of slices) {
        if (s.slice) {
          try {
            const realMetrics = await invoke<any>('analyze_slice_metrics', {
              symbol: result.symbol,
              hour: s.slice.hour,
              quarter: s.slice.quarter
            })

            if (s.slice.stats && realMetrics) {
              Object.assign(s.slice.stats, {
                atr_mean: realMetrics.atr_mean,
                volatility_mean: realMetrics.volatility_mean,
                range_mean: realMetrics.range_mean,
                body_range_mean: realMetrics.body_range_mean,
                noise_ratio_mean: realMetrics.noise_ratio_mean,
                breakout_percentage: realMetrics.breakout_percentage,
                volume_imbalance_mean: realMetrics.volume_imbalance_mean,
                shadow_ratio_mean: realMetrics.shadow_ratio_mean,
                candle_count: realMetrics.candle_count
              })
            }

            s.slice.straddleScore = calculateStraddleScore(s.slice.stats)
            s.tradingPlan = calculateTradingPlan(s.slice.stats, 100000, s.slice.straddleScore)

            if (s.rank === 1) {
              entryWindowAnalysis.value = {
                optimal_offset: realMetrics.optimal_entry_offset,
                optimal_win_rate: realMetrics.optimal_entry_win_rate
              }

              movementQualities.value = {
                [s.slice.hour + '-' + s.slice.quarter]: {
                  score: realMetrics.movement_quality_score,
                  label: realMetrics.movement_quality_label,
                  symbol: result.symbol,
                  event_type: 'N/A'
                }
              }
            }
          } catch (error) {
          }
        }
      }

      sliceAnalyses.value = slices

      if (sliceAnalyses.value && sliceAnalyses.value.length > 0) {
        const bestSlice = sliceAnalyses.value[0]

        try {
          const durationResult = await invoke<any>('analyze_volatility_duration_for_slice', {
            symbol: result.symbol,
            hour: bestSlice.slice.hour,
            quarter: bestSlice.slice.quarter
          })

          if (durationResult) {
            volatilityDuration.value = {
              peak_duration_minutes: durationResult.peak_duration_minutes,
              volatility_half_life_minutes: durationResult.volatility_half_life_minutes,
              recommended_trade_expiration_minutes: durationResult.recommended_trade_expiration_minutes,
              confidence_score: durationResult.confidence_score,
              sample_size: durationResult.sample_size
            }
          }
        } catch (error) {
          volatilityDuration.value = null
        }

        tradingPlan.value = bestSlice.tradingPlan
      }
    }
  }

  return {
    analysisData,
    sliceAnalyses,
    movementQualities,
    volatilityDuration,
    tradingPlan,
    entryWindowAnalysis,
    updateAnalysis
  }
}
