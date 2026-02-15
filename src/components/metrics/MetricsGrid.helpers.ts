import type { SliceAnalysis } from '../../utils/volatilityScore'
import { pipsToDisplayValue } from '../../utils/assetUnit'

interface GlobalMetrics {
  mean_atr?: number
  mean_max_true_range?: number
  mean_range?: number
  mean_volatility?: number
  mean_body_range?: number
  mean_volume_imbalance?: number
  mean_noise_ratio?: number
  mean_breakout_percentage?: number
}

interface AnalysisData {
  globalMetrics?: GlobalMetrics
  [key: string]: unknown
}

export interface MetricConfig {
  label: string
  value15: number
  valueGlobal: number
  goodThreshold: number
  excellentThreshold: number
  suffix?: string
  prefix?: string
  decimals?: number
  definition?: string
  usage?: string
  scoring?: string
  realUseCases?: string
}

export function buildMetricsConfig(analysis: SliceAnalysis, analysisData: AnalysisData): MetricConfig[] {
  const stats = analysis.slice.stats
  const globals: GlobalMetrics = analysisData?.globalMetrics || {}
  const unit = (analysisData?.unit as string) || 'pts'
  const symbol = (analysisData?.symbol as string) || ''
  const isCrypto = unit === '$'
  const suffix = isCrypto ? '' : unit
  const prefix = isCrypto ? '$' : ''
  const toDisplay = (v: number) => symbol ? pipsToDisplayValue(v, symbol) : v

  return [
    {
      label: 'ATR Moyen',
      value15: toDisplay(stats.atr_mean),
      valueGlobal: toDisplay(globals.mean_atr ?? 0),
      goodThreshold: 50,
      excellentThreshold: 100,
      suffix,
      prefix,
      decimals: 1,
      definition: 'Average True Range en points sur la periode. Mesure la volatilite exploitable pour regler SL/TP.',
      usage: 'ATR eleve = mouvements amples et spreads proportionnellement plus faibles. ATR faible = mouvement limite, risque de faux depart.',
      scoring: '🟢 Excellent > 100 pts | 🔵 Bon 50-100 pts | 🟡 Acceptable 20-50 pts | 🔴 Faible < 20 pts',
      realUseCases: 'Ex: ATR 120 pts -> SL ~180, TP ~300, conditions agressives possibles.\nEx: ATR 30 pts -> SL ~45, TP ~75, risques de slippage plus eleves.'
    },
    {
      label: 'Max Spike',
      value15: toDisplay(stats.max_true_range ?? 0),
      valueGlobal: toDisplay(globals.mean_max_true_range ?? 0),
      goodThreshold: 50,
      excellentThreshold: 100,
      suffix,
      prefix,
      decimals: 1,
      definition: 'Pic de mouvement (P95 du true range) sur le quarter. Reflete les pointes extremes possibles.',
      usage: 'Utile pour plafonner SL/TP et calibrer les niveaux de securite (SL recovery, hard TP).',
      scoring: '🟢 Excellent > 100 pts | 🔵 Bon 50-100 pts | 🟡 Acceptable 20-50 pts | 🔴 Faible < 20 pts',
      realUseCases: 'Ex: Max Spike 140 pts -> SL recovery cap a ~210 pts (1.5x).\nEx: Max Spike 25 pts -> plafonner agressivite, preferer objectifs courts.'
    },
    {
      label: 'Volatilité %',
      value15: stats.volatility_mean * 100,
      valueGlobal: (globals.mean_volatility ?? 0) * 100,
      goodThreshold: 15.0,
      excellentThreshold: 25.0,
      suffix: '%',
      decimals: 1,
      definition: 'Volatilite relative (ATR / prix) en %. Indique l amplitude potentielle du mouvement.',
      usage: 'Plus haut = potentiel de profit plus fort, execution plus exigeante. Trop bas = range.',
      scoring: '🟢 Excellent > 25% | 🔵 Bon 15-25% | 🟡 Acceptable 5-15% | 🔴 Faible < 5%',
      realUseCases: 'Ex: 22% -> straddle viable, offset normal.\nEx: 3% -> eviter, mouvement trop faible.'
    },
    {
      label: 'Body Range %',
      value15: stats.body_range_mean,
      valueGlobal: globals.mean_body_range ?? 0,
      goodThreshold: 35.0,
      excellentThreshold: 45.0,
      suffix: '%',
      decimals: 1,
      definition: 'Part du body dans le range total. Mesure la purete directionnelle du mouvement.',
      usage: 'Eleve = bougies pleines (direction claire). Bas = meches longues, bruit et indecision.',
      scoring: '🟢 Excellent > 45% | 🔵 Bon 35-45% | 🟡 Acceptable 15-35% | 🔴 Faible < 15%',
      realUseCases: 'Ex: 52% -> impulsion propre, TP plus ambitieux.\nEx: 18% -> meches dominantes, elargir SL ou eviter.'
    },
    {
      label: 'Direction Strength',
      value15: stats.volume_imbalance_mean * 100,
      valueGlobal: (globals.mean_volume_imbalance ?? 0) * 100,
      goodThreshold: 15.0,
      excellentThreshold: 20.0,
      suffix: '%',
      decimals: 1,
      definition: 'Force directionnelle combinee (body + breakout). Indique la conviction du mouvement.',
      usage: 'Elevee = tendance nette, meilleure probabilite de poursuite. Faible = indecision.',
      scoring: '🟢 Excellent > 20% | 🔵 Bon 15-20% | 🟡 Acceptable 5-15% | 🔴 Faible < 5%',
      realUseCases: 'Ex: 0.22 (22%) -> straddle directionnel agressif.\nEx: 0.04 (4%) -> reduire taille ou passer.'
    },
    {
      label: 'Noise Ratio',
      value15: stats.noise_ratio_mean,
      valueGlobal: globals.mean_noise_ratio ?? 0,
      goodThreshold: 2.0,
      excellentThreshold: 1.5,
      suffix: 'x',
      decimals: 2,
      definition: 'Ratio bruit/signal (meches vs body). Bas = mouvement propre, haut = erratique.',
      usage: 'Plus bas est meilleur. Un ratio eleve implique plus de faux signaux et SL plus larges.',
      scoring: '🟢 Excellent < 1.5x | 🔵 Bon 1.5-2.0x | 🟡 Acceptable 2.0-3.0x | 🔴 Faible > 3.0x',
      realUseCases: 'Ex: 1.7x -> execution standard.\nEx: 3.4x -> augmenter SL de 20-30% ou eviter.'
    },
    {
      label: 'Breakout %',
      value15: stats.breakout_percentage,
      valueGlobal: globals.mean_breakout_percentage ?? 0,
      goodThreshold: 15.0,
      excellentThreshold: 20.0,
      suffix: '%',
      decimals: 1,
      definition: 'Frequence des cassures de niveau. Mesure l impulsivite du marche.',
      usage: 'Haut = marche actif, bon pour straddle. Bas = range, nombreux faux departs.',
      scoring: '🟢 Excellent > 15% | 🔵 Bon 10-15% | 🟡 Acceptable 5-10% | 🔴 Faible < 5%',
      realUseCases: 'Ex: 18% -> impulsions frequentes, TP rapides possibles.\nEx: 3% -> marche endormi, mieux vaut attendre.'
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
