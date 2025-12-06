// Type definitions for archive analysis
export interface NormalizedArchive {
  id: string
  type: 'Volatilité' | 'Métriques Rétrospectives' | 'Heatmap'
  pair: string
  eventType: string
  peakAtr: number
  peakDelay: number
  decayTimeout: number
  confidence: number
  impactScore?: number
  eventCount?: number
  timestamp: string
}

export interface RawArchive {
  id: number
  title: string
  archive_type: string
  period_start: string
  period_end: string
  comment?: string
  created_at: string
  data_json: string
}

export interface EventStats {
  eventType: string
  avgATR: number
  avgPeakDelay: number
  avgDecayTimeout: number
  avgConfidence: number
  count: number
  variance?: number
  heatmapImpact?: number
  tradabilityScore?: number
}

export interface PairStats {
  pair: string
  avgConfidence: number
  avgATR: number
  count: number
  eventSensitivity: Record<string, number>
  performanceRating?: string
}

export interface EventPairStats {
  key: string // "EVENT|PAIR"
  eventType: string
  pair: string
  avgATR: number
  avgConfidence: number
  count: number
  slAdjusted?: number
  trailingStopCoefficient?: number
}
