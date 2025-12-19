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

export interface RecurringEvent {
  time: string
  name: string
  impact: string
  currency: string
  frequency: number
}
