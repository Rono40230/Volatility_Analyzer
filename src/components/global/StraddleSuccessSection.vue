<template>
  <div v-if="result.pair_straddle_rates?.length > 0" class="straddle-section">
    <h3>📊 Taux de Réussite du Straddle par Paire</h3>
    <div class="straddle-grid">
      <div v-for="(rate, index) in result.pair_straddle_rates.slice(0, 6)" :key="rate.pair" class="straddle-card glass" :class="'straddle-rank-' + (index + 1)">
        <div class="straddle-header">
          <div class="straddle-rank">#{{ index + 1 }}</div>
          <div class="straddle-pair">{{ rate.pair }}</div>
          <div class="straddle-main-score" :class="getScoreClass(rate.straddle_score)">{{ rate.straddle_score.toFixed(0) }}</div>
        </div>
        <div class="straddle-metrics">
          <div class="straddle-metric">
            <span class="metric-label">Directional Move</span>
            <div class="metric-bar-container">
              <div class="metric-bar directional" :style="{ width: rate.directional_move_rate + '%' }" />
              <span class="metric-bar-value">{{ rate.directional_move_rate.toFixed(0) }}%</span>
            </div>
          </div>
          <div class="straddle-metric">
            <span class="metric-label">Whipsaw Rate</span>
            <div class="metric-bar-container">
              <div class="metric-bar whipsaw" :style="{ width: rate.whipsaw_rate + '%' }" />
              <span class="metric-bar-value">{{ rate.whipsaw_rate.toFixed(0) }}%</span>
            </div>
          </div>
          <div class="straddle-metric"><span class="metric-label">Volatilité Moyenne</span><span class="metric-value">{{ (rate.avg_volatility * 100).toFixed(2) }}%</span></div>
          <div class="straddle-metric"><span class="metric-label">Événements Analysés</span><span class="metric-value">{{ rate.total_events }}</span></div>
        </div>
        <div class="straddle-events">
          <span class="events-label">Top événements:</span>
          <ul class="events-list"><li v-for="event in rate.top_events" :key="event">{{ event }}</li></ul>
        </div>
      </div>
    </div>
    <div class="insight-box glass">
      <h4>💡 Comment Interpréter</h4>
      <p>Le <strong>Score Straddle</strong> = Directional Move Rate - Whipsaw Rate. Un score élevé indique que la paire génère des mouvements directionnels clairs (bon pour straddle) avec peu de whipsaws (allers-retours qui tuent les positions). Privilégiez les paires avec un score > 40 et au moins 10 événements analysés.</p>
    </div>
  </div>
  <div v-else class="straddle-placeholder glass">
    <div class="placeholder-icon">📊</div>
    <h4>Taux de Réussite du Straddle par Paire</h4>
    <p>Cette analyse nécessite des archives de type "Corrélation paire/événement".</p>
    <p class="hint">Créez des analyses de corrélation pour débloquer cette fonctionnalité.</p>
  </div>
</template>

<script setup lang="ts">
interface PairStraddleRate {
  pair: string
  straddle_score: number
  directional_move_rate: number
  whipsaw_rate: number
  avg_volatility: number
  total_events: number
  top_events: string[]
}

interface StraddleSuccessResult {
  pair_straddle_rates?: PairStraddleRate[]
}

defineProps<{
  result: StraddleSuccessResult
}>()

function getScoreClass(score: number): string {
  if (score >= 75) return 'score-excellent'
  if (score >= 50) return 'score-good'
  if (score >= 25) return 'score-average'
  return 'score-poor'
}
</script>

<style scoped>
.straddle-section { margin-bottom: 20px; }
.straddle-section h3 { font-size: 18px; margin-bottom: 20px; color: #e2e8f0; }

.straddle-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; margin-bottom: 20px; }

.straddle-card { padding: 16px; border-radius: 12px; border: 1px solid rgba(255, 255, 255, 0.05); transition: all 0.2s; }
.straddle-card:hover { transform: translateY(-2px); border-color: rgba(78, 205, 196, 0.3); }

.straddle-header { display: flex; align-items: center; gap: 12px; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); }

.straddle-rank { font-size: 14px; font-weight: 700; color: #718096; background: rgba(255, 255, 255, 0.05); padding: 2px 8px; border-radius: 4px; }

.straddle-rank-1 .straddle-rank { color: #fbbf24; background: rgba(251, 191, 36, 0.1); }

.straddle-pair { flex: 1; font-weight: 700; color: #fff; font-size: 16px; }
.straddle-main-score { font-size: 20px; font-weight: 800; }

.score-excellent { color: #4ecdc4; }
.score-good { color: #3b82f6; }
.score-average { color: #f59e0b; }
.score-poor { color: #ef4444; }

.straddle-metrics { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px; }
.straddle-metric { display: flex; flex-direction: column; gap: 4px; }

.metric-label { font-size: 10px; color: #a0aec0; text-transform: uppercase; }
.metric-value { font-size: 14px; font-weight: 700; color: #fff; }

.metric-bar-container { display: flex; align-items: center; gap: 8px; height: 16px; }
.metric-bar { height: 6px; border-radius: 3px; background: rgba(255, 255, 255, 0.1); }

.metric-bar.directional { background: #4ecdc4; }
.metric-bar.whipsaw { background: #ef4444; }

.metric-bar-value { font-size: 12px; font-weight: 600; color: #fff; min-width: 30px; text-align: right; }

.straddle-events { font-size: 12px; }
.events-label { color: #a0aec0; display: block; margin-bottom: 4px; }

.events-list { list-style: none; padding: 0; margin: 0; display: flex; flex-wrap: wrap; gap: 6px; }
.events-list li { background: rgba(255, 255, 255, 0.05); padding: 2px 8px; border-radius: 10px; color: #e2e8f0; font-size: 11px; }

.insight-box { padding: 16px; border-radius: 8px; border-left: 4px solid #fbbf24; background: rgba(251, 191, 36, 0.1); }
.insight-box h4 { color: #fbbf24; margin: 0 0 8px 0; font-size: 14px; }
.insight-box p { margin: 0; font-size: 13px; color: #e2e8f0; line-height: 1.5; }

.straddle-placeholder { padding: 40px; text-align: center; border-radius: 12px; border: 1px dashed rgba(255, 255, 255, 0.1); margin-bottom: 40px; }
.placeholder-icon { font-size: 40px; margin-bottom: 16px; opacity: 0.5; }
.straddle-placeholder h4 { color: #e2e8f0; margin-bottom: 8px; }
.straddle-placeholder p { color: #a0aec0; font-size: 13px; margin: 0; }
.straddle-placeholder .hint { color: #4ecdc4; margin-top: 8px; font-size: 12px; }
.glass { background: rgba(30, 30, 45, 0.6); backdrop-filter: blur(10px); }
</style>
