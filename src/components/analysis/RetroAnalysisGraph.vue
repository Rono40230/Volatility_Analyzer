<template>
  <div class="graph-section">
    <RetroAnalysisGraphMetrics
      :noise-ratio-before="noiseRatioBefore"
      :noise-ratio-during="noiseRatioDuring"
      :noise-ratio-after="noiseRatioAfter"
      :volatility-increase-percent="volatilityIncreasePercent"
      :event-count="eventCount"
    />

    <!-- Graphique 2 courbes comparatives -->
    <div class="graph-container">
      <svg viewBox="0 0 1000 450" class="graph">
        <defs>
          <linearGradient id="beforeGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" style="stop-color:#58a6ff;stop-opacity:0.3" />
            <stop offset="100%" style="stop-color:#58a6ff;stop-opacity:0" />
          </linearGradient>
          <linearGradient id="afterGradient" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" style="stop-color:#f85149;stop-opacity:0.3" />
            <stop offset="100%" style="stop-color:#f85149;stop-opacity:0" />
          </linearGradient>
        </defs>

        <line :x1="svgMargins.left" :y1="yAxisBaseline" :x2="svgMargins.right" :y2="yAxisBaseline" stroke="#4a5568" stroke-width="2" />
        <line :x1="svgMargins.left" y1="50" :x2="svgMargins.left" y2="380" stroke="#4a5568" stroke-width="2" />

        <line :x1="svgMargins.left" :y1="yMidLine" :x2="svgMargins.right" :y2="yMidLine" stroke="#2d3748" stroke-width="1" stroke-dasharray="5,5" />

        <line :x1="svgMargins.t0" y1="50" :x2="svgMargins.t0" y2="380" stroke="#fbbf24" stroke-width="2.5" stroke-dasharray="4,4" opacity="0.8" />
        <text :x="svgMargins.t0" y="35" font-size="12" text-anchor="middle" fill="#fbbf24" font-weight="bold">T0 (Événement)</text>



        <line v-if="props.meilleurMoment > 0" :x1="bestMomentX" y1="50" :x2="bestMomentX" y2="380" stroke="#10b981" stroke-width="2" stroke-dasharray="6,3" opacity="0.7" />
        <text v-if="props.meilleurMoment > 0" :x="bestMomentX" y="45" font-size="11" text-anchor="middle" fill="#10b981" font-weight="600">Entrée (T0 - {{ props.meilleurMoment }} mn)</text>

        <!-- Graduations Y (tous les 1 pip) -->
        <template v-for="tick in yAxisTicks" :key="`y-tick-${tick}`">
          <line :x1="svgMargins.left" :y1="mapPipToY(tick)" :x2="svgMargins.right" :y2="mapPipToY(tick)" stroke="#718096" stroke-width="1" stroke-dasharray="3,3" opacity="0.6" />
          <text :x="svgMargins.labelY" :y="mapPipToY(tick) + 4" font-size="11" text-anchor="end" fill="#cbd5e0" font-weight="500">{{ tick }}</text>
        </template>

        <!-- Marqueurs X: AVANT (-30 à 0) -->
        <template v-for="minute in [-30, -20, -10, 0]" :key="`tick-before-${minute}`">
          <line :x1="getXPositionBefore(minute)" :y1="yAxisBaseline + 5" :x2="getXPositionBefore(minute)" :y2="yAxisBaseline + 12" stroke="#4a5568" stroke-width="1.5" />
          <text :x="getXPositionBefore(minute)" :y="yAxisBaseline + 28" font-size="10" text-anchor="middle" fill="#8b949e">{{ getTimeLabel(minute) }}</text>
        </template>

        <!-- Marqueurs X: APRÈS (0 à 90, tous les 15 min) -->
        <template v-for="minute in [0, 15, 30, 45, 60, 75, 90]" :key="`tick-after-${minute}`">
          <line :x1="getXPositionAfter(minute)" :y1="yAxisBaseline + 5" :x2="getXPositionAfter(minute)" :y2="yAxisBaseline + 12" stroke="#4a5568" stroke-width="1.5" />
          <text :x="getXPositionAfter(minute)" :y="yAxisBaseline + 28" font-size="10" text-anchor="middle" fill="#8b949e">{{ getTimeLabel(minute) }}</text>
        </template>

        <template v-if="atrTimelineBefore && atrTimelineBefore.length > 1">
          <path :d="curvePathBefore" fill="url(#beforeGradient)" stroke="none" />
          <polyline :points="beforePointsString" fill="none" stroke="#58a6ff" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round" />
          <text x="130" y="100" font-size="13" fill="#58a6ff" font-weight="bold">ATR Avant: {{ formatValue(atrTimelineBefore[Math.floor(atrTimelineBefore.length / 2)]) }}</text>
        </template>

        <template v-if="atrTimelineAfter && atrTimelineAfter.length > 1">
          <path :d="curvePathAfter" fill="url(#afterGradient)" stroke="none" />
          <polyline :points="afterPointsString" fill="none" stroke="#f85149" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round" />
          <text x="600" y="100" font-size="13" fill="#f85149" font-weight="bold">ATR Après: {{ formatValue(atrTimelineAfter[Math.floor(atrTimelineAfter.length / 2)]) }}</text>
        </template>
      </svg>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRetroGraphDataPoints } from '../../composables/useRetroGraphDataPoints'
import RetroAnalysisGraphMetrics from './RetroAnalysisGraphMetrics.vue'

interface Props {
  atrTimelineBefore?: number[]
  atrTimelineAfter?: number[]
  bodyTimelineBefore?: number[]
  bodyTimelineAfter?: number[]
  noiseRatioBefore: number
  noiseRatioDuring: number
  noiseRatioAfter: number
  volatilityIncreasePercent: number
  eventCount: number
  eventType: string
  pair: string
  eventDatetime?: string
  timezoneOffset?: string
  isArchiveMode?: boolean
  eventLabel?: string
  meilleurMoment?: number
  pointValue?: number
}

const props = withDefaults(defineProps<Props>(), {
  atrTimelineBefore: () => [],
  atrTimelineAfter: () => [],
  bodyTimelineBefore: () => [],
  bodyTimelineAfter: () => [],
  noiseRatioBefore: 0,
  noiseRatioDuring: 0,
  noiseRatioAfter: 0,
  volatilityIncreasePercent: 0,
  eventCount: 0,
  eventType: '',
  pair: '',
  eventDatetime: '',
  timezoneOffset: 'UTC+0',
  isArchiveMode: false,
  eventLabel: '',
  meilleurMoment: 0,
  pointValue: 1.0
})

const {
  svgMargins,
  yAxisBaseline,
  yMidLine,
  minAtrLabel,
  maxAtrLabel,
  midAtrLabel,
  getTimeLabel,
  getXPositionBefore,
  getXPositionAfter,
  ceilValue,
  formatValue,
  bestMomentX,
  beforePointsString,
  afterPointsString,
  curvePathBefore,
  curvePathAfter,
  yAxisTicks,
  mapPipToY
} = useRetroGraphDataPoints({
  atrTimelineBefore: props.atrTimelineBefore,
  atrTimelineAfter: props.atrTimelineAfter,
  meilleurMoment: props.meilleurMoment,
  eventDatetime: props.eventDatetime,
  pointValue: props.pointValue
})
</script>

<style scoped>
.graph-section {
  background: #161b22;
  padding: 15px;
  border-radius: 8px;
  border: 1px solid #30363d;
  height: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

.graph-section h3 { margin: 0; color: #58a6ff; font-size: 1em; white-space: nowrap; }
.quick-conclusion { font-size: 0.85em; font-weight: 600; margin: 0; white-space: nowrap; }
.quick-conclusion .conclusion-positive { color: #3fb950; }
.quick-conclusion .conclusion-negative { color: #f85149; }

.graph-container {
  width: 100%;
  background: #0d1117;
  border-radius: 6px;
  padding: 10px;
  flex: 1;
  min-height: 0;
  border: 1px solid #30363d;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 0;
  box-sizing: border-box;
}

.graph { width: 100%; height: 100%; }
.btn-archive { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; padding: 8px 16px; border-radius: 6px; font-weight: 600; cursor: pointer; transition: all 0.2s; font-size: 0.9em; flex-shrink: 0; }
.btn-archive:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }
.btn-archive:active { transform: translateY(0); }

@media (max-width: 1024px) {
  .graph-container { min-height: 320px; padding: 8px; }
}
@media (max-width: 768px) {
  .graph-section { padding: 12px; }
  .graph-section h3 { font-size: 0.9em; }
  .graph-container { min-height: 280px; padding: 6px; margin-bottom: 10px; }
  .btn-archive { padding: 6px 12px; font-size: 0.8em; }
}
@media (max-width: 480px) {
  .graph-section { padding: 10px; }
  .graph-section h3 { font-size: 0.8em; }
  .graph-container { min-height: 240px; padding: 4px; margin-bottom: 8px; }
  .btn-archive { padding: 6px 10px; font-size: 0.75em; }
}
</style>
