// types/entryAnalysis.ts — Types pour l'analyse de point d'entrée (Phase 2/4)

export interface MinuteDetail {
  offset: number
  win_rate: number
  avg_net_profit_pips: number
  avg_spread_pips: number
  sample_size: number
  tradable: boolean
}

export interface EntryAnalysisResult {
  symbol: string
  event_type: string
  optimal_offset_minutes: number
  optimal_entry_time_label: string
  real_win_rate: number
  avg_net_profit_pips: number
  avg_spread_at_entry_pips: number
  avg_movement_pips: number
  peak_minute: number
  movement_duration_minutes: number
  decay_speed: string
  consistency_score: number
  sample_size: number
  non_tradable_minutes: number[]
  minute_details: MinuteDetail[]
  unit: string
}

export interface EntryPointParams {
  symbol: string
  hour: number
  quarter: number
  event_type?: string
  forward_minutes?: number
  spread_threshold_pips?: number
  min_samples?: number
}
