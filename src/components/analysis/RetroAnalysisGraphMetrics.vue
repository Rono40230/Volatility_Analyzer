<template>
  <div class="graph-header-metrics">
    <div class="card signal-card">
      <div class="card-heading">
        <span class="card-title">Signal Straddle</span>
        <span class="verdict-pill" :class="verdictTone">{{ verdictText }}</span>
      </div>
      <div class="score-grid">
        <div class="score-block">
          <span class="block-label">Score avant</span>
          <span class="block-value" :class="scoreToneBefore">{{ Math.round(volatilityScoreBefore) }}</span>
          <span class="block-hint">ATR {{ formatPercent(atrPercentBefore) }}</span>
        </div>
        <div class="score-block">
          <span class="block-label">Score après</span>
          <span class="block-value" :class="scoreToneAfter">{{ Math.round(volatilityScoreAfter) }}</span>
          <span class="block-hint">ATR {{ formatPercent(atrPercentAfter) }}</span>
        </div>
      </div>
      <div class="signal-meta">
        <div class="stat-item">
          <span class="stat-label">Noise pendant</span>
          <span class="stat-value" :class="noiseQualityDuring">{{ noiseRatioDuring.toFixed(2) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Impact volatilité</span>
          <span class="stat-value impact-value">{{ formattedImpact }}</span>
        </div>
      </div>
    </div>

    <div class="card volatility-card">
      <div class="card-heading">
        <span class="card-title">Volatilité locale</span>
      </div>
      <div class="volatility-grid">
        <div class="volatility-cell">
          <span class="cell-label">ATR avant</span>
          <span class="cell-value">{{ formatPercent(atrPercentBefore) }}</span>
        </div>
        <div class="volatility-cell">
          <span class="cell-label">ATR après</span>
          <span class="cell-value">{{ formatPercent(atrPercentAfter) }}</span>
        </div>
        <div class="volatility-cell" v-if="contextVolatilityPercent > 0">
          <span class="cell-label">Volatilité horaire</span>
          <span class="cell-value">{{ formatPercent(contextVolatilityPercent) }}</span>
        </div>
        <div class="volatility-cell" v-if="avgDeviation > 0">
          <span class="cell-label">Écart moyen</span>
          <span class="cell-value">{{ avgDeviation.toFixed(2) }}</span>
        </div>
      </div>
      <div class="noise-row">
        <div class="stat-item">
          <span class="stat-label">Noise avant</span>
          <span class="stat-value" :class="noiseQualityBefore">{{ noiseRatioBefore.toFixed(2) }}</span>
        </div>
        <div class="stat-item">
          <span class="stat-label">Noise après</span>
          <span class="stat-value" :class="noiseQualityAfter">{{ noiseRatioAfter.toFixed(2) }}</span>
        </div>
      </div>
    </div>

    <div class="card history-card">
      <div class="card-heading">
        <span class="card-title">Historique &amp; confiance</span>
      </div>
      <div class="history-grid">
        <div class="history-cell">
          <span class="cell-label">Occurrences</span>
          <span class="cell-value">{{ eventCount }}</span>
        </div>
        <div class="history-cell">
          <span class="cell-label">Surprises</span>
          <span class="cell-value">{{ surpriseEventCount }}</span>
        </div>
        <div class="history-cell">
          <span class="cell-label">Point value</span>
          <span class="cell-value">{{ pointValueDisplay }}</span>
        </div>
        <div class="history-cell">
          <span class="cell-label">Impact cumulé</span>
          <span class="cell-value">{{ formattedImpact }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  noiseRatioBefore: number
  noiseRatioDuring: number
  noiseRatioAfter: number
  volatilityIncreasePercent: number
  eventCount: number
  volatilityScoreBefore: number
  volatilityScoreAfter: number
  atrPercentBefore: number
  atrPercentAfter: number
  contextVolatilityPercent?: number
  avgDeviation?: number
  surpriseEventCount?: number
  pointValue?: number
}>(), {
  contextVolatilityPercent: 0,
  avgDeviation: 0,
  surpriseEventCount: 0,
  pointValue: 0
})

const noiseQualityBefore = computed(() => props.noiseRatioBefore < 1.5 ? 'clean' : props.noiseRatioBefore < 2.5 ? 'mixed' : 'choppy')
const noiseQualityDuring = computed(() => props.noiseRatioDuring < 1.5 ? 'clean' : props.noiseRatioDuring < 2.5 ? 'mixed' : 'choppy')
const noiseQualityAfter = computed(() => props.noiseRatioAfter < 1.5 ? 'clean' : props.noiseRatioAfter < 2.5 ? 'mixed' : 'choppy')

const toneForScore = (score: number) => {
  if (score >= 70) return 'score-strong'
  if (score >= 40) return 'score-neutral'
  return 'score-weak'
}

const scoreToneBefore = computed(() => toneForScore(props.volatilityScoreBefore))
const scoreToneAfter = computed(() => toneForScore(props.volatilityScoreAfter))

const verdict = computed(() => {
  const vol = props.volatilityIncreasePercent
  const noise = props.noiseRatioDuring

  if (vol < 10) {
    return { text: 'Impact faible', tone: 'verdict-calm' }
  }

  if (noise > 3.0) {
    return { text: 'Bruit excessif', tone: 'verdict-risk' }
  }

  if (noise > 2.0) {
    return { text: 'Signal fragile', tone: 'verdict-warning' }
  }

  return { text: 'Signal propre', tone: 'verdict-go' }
})

const verdictText = computed(() => verdict.value.text)
const verdictTone = computed(() => verdict.value.tone)
const formatPercent = (value: number) => `${value.toFixed(2)}%`
const formattedImpact = computed(() => `${props.volatilityIncreasePercent >= 0 ? '+' : ''}${props.volatilityIncreasePercent.toFixed(1)}%`)
const pointValueDisplay = computed(() => {
  if (!props.pointValue || props.pointValue <= 0) return 'n/a'
  return props.pointValue.toFixed(2)
})
</script>

<style scoped>
.graph-header-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}

.card {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 12px;
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card.signal-card { border-color: #312b81; box-shadow: 0 0 20px rgba(49, 43, 129, 0.35); }
.card.volatility-card { border-color: #173d5c; }
.card.history-card { border-color: #2f3a45; }

.card-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-title {
  font-size: 0.95em;
  font-weight: 600;
  color: #e6edf3;
  text-transform: uppercase;
  letter-spacing: 0.6px;
}

.verdict-pill {
  font-size: 0.8em;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 999px;
  text-transform: uppercase;
}

.verdict-calm { background: rgba(139, 148, 158, 0.2); color: #c9d1d9; }
.verdict-warning { background: rgba(210, 153, 34, 0.2); color: #f0b429; }
.verdict-risk { background: rgba(248, 81, 73, 0.2); color: #f85149; }
.verdict-go { background: rgba(63, 185, 80, 0.2); color: #3fb950; }

.score-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.score-block {
  background: rgba(21, 25, 34, 0.8);
  border: 1px solid #1f2a37;
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.block-label {
  font-size: 0.75em;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  color: #8b949e;
}

.block-value {
  font-size: 2.1em;
  font-weight: 700;
}

.block-value.score-strong { color: #3fb950; }
.block-value.score-neutral { color: #fbbf24; }
.block-value.score-weak { color: #f85149; }

.block-hint {
  font-size: 0.75em;
  color: #8b949e;
}

.signal-meta {
  display: flex;
  gap: 18px;
  flex-wrap: wrap;
}

.stat-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.stat-label {
  font-size: 0.7em;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  font-size: 1.05em;
  font-weight: 700;
  color: #e6edf3;
}

.stat-value.clean { color: #3fb950; }
.stat-value.mixed { color: #d29922; }
.stat-value.choppy { color: #f85149; }
.impact-value { color: #a371f7; }

.volatility-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.volatility-cell,
.history-cell {
  background: rgba(13, 17, 23, 0.9);
  border: 1px solid #1f2a37;
  border-radius: 10px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.cell-label {
  font-size: 0.7em;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #8b949e;
}

.cell-value {
  font-size: 1.2em;
  font-weight: 700;
  color: #e6edf3;
}

.noise-row {
  display: flex;
  gap: 18px;
  flex-wrap: wrap;
}

.history-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

@media (max-width: 768px) {
  .graph-header-metrics {
    grid-template-columns: 1fr;
  }

  .score-grid,
  .volatility-grid,
  .history-grid {
    grid-template-columns: 1fr;
  }
}
</style>
