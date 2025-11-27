import type { SliceAnalysis } from '../../utils/straddleAnalysis'

export interface MetricConfig {
  label: string
  value15: number
  valueGlobal: number
  goodThreshold: number
  excellentThreshold: number
  suffix?: string
  decimals?: number
}

export function getEstimatedPrice(analysisData: any): number {
  const globalMetrics = analysisData?.globalMetrics
  if (!globalMetrics) return 100000
  const atr = globalMetrics.mean_atr
  if (atr > 1000) return 100000
  if (atr > 10) return 10000
  return 1.0
}

export function buildMetricsConfig(analysis: SliceAnalysis, analysisData: any): MetricConfig[] {
  const stats = analysis.slice.stats
  const globals = analysisData?.globalMetrics || {}
  const price = getEstimatedPrice(analysisData)

  return [
    {
      label: 'ATR Moyen',
      value15: (stats.atr_mean / price) * 100,
      valueGlobal: (globals.mean_atr / price) * 100,
      goodThreshold: 1.5,
      excellentThreshold: 2.0,
      suffix: '%',
      decimals: 2
    },
    {
      label: 'True Range',
      value15: (stats.range_mean / price) * 100,
      valueGlobal: (globals.mean_range / price) * 100,
      goodThreshold: 1.5,
      excellentThreshold: 2.5,
      suffix: '%',
      decimals: 2
    },
    {
      label: 'Volatilité %',
      value15: stats.volatility_mean * 100,
      valueGlobal: globals.mean_volatility ? globals.mean_volatility * 100 : 0,
      goodThreshold: 15.0,
      excellentThreshold: 25.0,
      suffix: '%',
      decimals: 1
    },
    {
      label: 'Body Range %',
      value15: stats.body_range_mean,
      valueGlobal: globals.mean_body_range || 0,
      goodThreshold: 35.0,
      excellentThreshold: 45.0,
      suffix: '%',
      decimals: 1
    },
    {
      label: 'Direction Strength',
      value15: stats.volume_imbalance_mean * 100,
      valueGlobal: globals.mean_volume_imbalance ? globals.mean_volume_imbalance * 100 : 0,
      goodThreshold: 15.0,
      excellentThreshold: 20.0,
      suffix: '%',
      decimals: 1
    },
    {
      label: 'Noise Ratio',
      value15: stats.noise_ratio_mean,
      valueGlobal: globals.mean_noise_ratio || 0,
      goodThreshold: 2.0,
      excellentThreshold: 1.5,
      decimals: 2
    },
    {
      label: 'Breakout %',
      value15: stats.breakout_percentage,
      valueGlobal: globals.mean_breakout_percentage || 0,
      goodThreshold: 15.0,
      excellentThreshold: 20.0,
      suffix: '%',
      decimals: 1
    }
  ]
}

export function getMetricStatus(value: number, goodThreshold: number, excellentThreshold: number): string {
  if (value > excellentThreshold) return 'excellent'
  if (value > goodThreshold) return 'good'
  if (value > (goodThreshold / 2)) return 'acceptable'
  return 'poor'
}

export function getMetricStatusText(value: number, goodThreshold: number, excellentThreshold: number): string {
  if (value > excellentThreshold) return '🟢 Excellent'
  if (value > goodThreshold) return '🔵 Bon'
  if (value > (goodThreshold / 2)) return '🟡 Acceptable'
  return '🔴 Faible'
}

export function formatNumber(value: number, decimals: number = 2): string {
  if (typeof value !== 'number') return '0'
  return value.toFixed(decimals)
}

export function getMetricClass(value: number, goodThreshold: number, excellentThreshold: number): string {
  const status = getMetricStatus(value, goodThreshold, excellentThreshold)
  const statusMap: Record<string, string> = {
    excellent: 'excellent',
    good: 'good',
    acceptable: 'acceptable',
    poor: 'poor'
  }
  return statusMap[status] || 'poor'
}
