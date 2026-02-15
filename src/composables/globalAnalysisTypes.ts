/** Types for Global Analysis */

export interface GlobalStats {
  average_confidence: number
  average_volatility: number
}

export interface BestPairGlobal {
  symbol: string
  analysis_count: number
  score: number
}

export interface GoldenHourGlobal {
  hour: number
  reliability: number
}

export interface TradableEventGlobal {
  event_name: string
  tradability_score: number
  avg_volatility_increase: number
  occurrence_count: number
  affected_pairs: string[]
}

export interface OptimalTimeWindowGlobal {
  event_type: string
  consistency_score: number
  avg_peak_time_minutes: number
  avg_entry_window_minutes: number
  avg_return_to_normal_minutes: number
  occurrence_count: number
  affected_pairs: string[]
}

export interface GlobalAnalysisResult {
  total_analyses: number
  total_days_analyzed: number
  global_stats: GlobalStats
  best_pairs: BestPairGlobal[]
  golden_hours: GoldenHourGlobal[]
  tradable_events: TradableEventGlobal[]
  optimal_time_windows: OptimalTimeWindowGlobal[]
  generated_at: string
}
