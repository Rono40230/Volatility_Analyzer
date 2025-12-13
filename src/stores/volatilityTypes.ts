export interface SymbolInfo {
  symbol: string
  file_path: string
}

export interface EventInHour {
  event_name: string
  impact: string
  datetime: string
  volatility_increase: number
}

export interface HourlyStats {
  hour: number
  candle_count: number
  atr_mean: number
  atr_max: number
  volatility_mean: number
  range_mean: number
  body_range_mean: number
  shadow_ratio_mean: number
  volume_imbalance_mean: number
  noise_ratio_mean: number
  breakout_percentage: number
  events: EventInHour[]
}

export interface Stats15Min {
  hour: number           // 0-23
  quarter: number        // 0-3 (00-15min, 15-30min, 30-45min, 45-60min)
  candle_count: number
  atr_mean: number
  atr_max: number
  volatility_mean: number
  range_mean: number
  body_range_mean: number
  shadow_ratio_mean: number
  volume_imbalance_mean: number
  noise_ratio_mean: number
  breakout_percentage: number
  events: EventInHour[]
  peak_duration_minutes?: number
  volatility_half_life_minutes?: number
  recommended_trade_expiration_minutes?: number
  peak_duration_mean?: number
  volatility_half_life_mean?: number
  recommended_trade_expiration_mean?: number
  straddle_parameters?: {
    offset_pips: number
    stop_loss_pips: number
    trailing_stop_pips: number
    timeout_minutes: number
    sl_recovery_pips: number
    risk_reward_ratio: number
  }
  volatility_profile?: number[]
  optimal_entry_minute?: number
}

export interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  mean_range: number
  total_candles: number
}

export interface CalendarEvent {
  id: number
  symbol: string
  event_time: string
  impact: string
  description: string
  actual: number | null
  forecast: number | null
  previous: number | null
  created_at: string
}

export interface CorrelatedEvent {
  event: CalendarEvent
  volatility_hour: number
  volatility_increase: number
  correlation_score: number
}

export interface AnalysisResult {
  symbol: string
  period_start: string
  period_end: string
  timeframe: string
  hourly_stats: HourlyStats[]
  stats_15min: Stats15Min[]      // Nouvelles stats pour scalping
  best_quarter: [number, number] // [hour, quarter] - meilleur quarter de la journée
  confidence_score: number
  recommendation: string
  risk_level: string
  global_metrics: GlobalMetrics
  correlated_events: CorrelatedEvent[]
}
