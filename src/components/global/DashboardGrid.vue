<template>
  <div class="dashboard-grid">
    <!-- Colonne Gauche : Top Paires -->
    <div class="dashboard-column">
      <h3>🏆 Top Paires Performantes</h3>
      <div class="pairs-list">
        <div
          v-for="(pair, index) in result.best_pairs.slice(0, 5)"
          :key="pair.symbol"
          class="pair-item glass"
          :class="'rank-' + (index + 1)"
        >
          <div class="pair-rank">
            #{{ index + 1 }}
          </div>
          <div class="pair-info">
            <span class="pair-symbol">{{ pair.symbol }}</span>
            <span class="pair-details">{{ pair.analysis_count }} analyses</span>
          </div>
          <div class="pair-score">
            <span class="score-label">Score IA</span>
            <span class="score-value">{{ pair.score.toFixed(0) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Colonne Droite : Golden Hours -->
    <div class="dashboard-column">
      <h3>⏰ Golden Hours (Heures en Or)</h3>
      <div class="hours-chart">
        <div
          v-for="hour in sortedGoldenHours"
          :key="hour.hour"
          class="hour-bar-container"
        >
          <div class="hour-label">
            {{ hour.hour }}h
          </div>
          <div class="hour-bar-wrapper">
            <div
              class="hour-bar"
              :style="{ width: hour.reliability + '%' }"
              :class="getHourClass(hour.reliability)"
            />
          </div>
          <div class="hour-value">
            {{ hour.reliability.toFixed(0) }}%
          </div>
        </div>
      </div>
      <div class="insight-box glass">
        <h4>💡 L'Insight de l'IA</h4>
        <p>
          D'après l'analyse de {{ result.total_analyses }} sessions, votre créneau optimal est 
          <strong>{{ bestHour }}h</strong> avec une fiabilité de <strong>{{ bestHourReliability }}%</strong>.
          Privilégiez la paire <strong>{{ bestPair }}</strong> pour maximiser vos chances.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface BestPair {
  symbol: string
  analysis_count: number
  score: number
}

interface GoldenHour {
  hour: number
  reliability: number
}

interface DashboardResult {
  best_pairs: BestPair[]
  total_analyses: number
}

defineProps<{
  result: DashboardResult
  sortedGoldenHours: GoldenHour[]
  bestHour: number | string
  bestHourReliability: string
  bestPair: string
}>()

function getHourClass(reliability: number): string {
  if (reliability >= 80) return 'bar-excellent'
  if (reliability >= 60) return 'bar-good'
  if (reliability >= 40) return 'bar-average'
  return 'bar-poor'
}
</script>

<style scoped>
.dashboard-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 30px; margin-bottom: 40px; }
@media (max-width: 1000px) { .dashboard-grid { grid-template-columns: 1fr; } }
.dashboard-column h3 { font-size: 18px; margin-bottom: 20px; color: #e2e8f0; display: flex; align-items: center; gap: 10px; }
.pairs-list { display: flex; flex-direction: column; gap: 12px; }
.pair-item { display: flex; align-items: center; padding: 12px 16px; border-radius: 8px; border: 1px solid rgba(255, 255, 255, 0.05); transition: all 0.2s; }
.pair-item:hover { background: rgba(255, 255, 255, 0.08); transform: translateX(4px); }
.pair-rank { font-size: 16px; font-weight: 700; color: #718096; width: 40px; }
.rank-1 .pair-rank { color: #fbbf24; }
.rank-2 .pair-rank { color: #94a3b8; }
.rank-3 .pair-rank { color: #b45309; }
.pair-info { flex: 1; display: flex; flex-direction: column; }
.pair-symbol { font-weight: 700; color: #fff; font-size: 15px; }
.pair-details { font-size: 11px; color: #a0aec0; }
.pair-score { text-align: right; }
.score-label { display: block; font-size: 10px; color: #a0aec0; text-transform: uppercase; }
.score-value { font-size: 18px; font-weight: 700; color: #4ecdc4; }
.hours-chart { display: flex; flex-direction: column; gap: 12px; margin-bottom: 20px; }
.hour-bar-container { display: flex; align-items: center; gap: 12px; }
.hour-label { width: 40px; font-size: 13px; color: #a0aec0; text-align: right; }
.hour-bar-wrapper { flex: 1; height: 8px; background: rgba(255, 255, 255, 0.05); border-radius: 4px; overflow: hidden; }
.hour-bar { height: 100%; border-radius: 4px; }
.bar-excellent { background: linear-gradient(90deg, #4ecdc4, #2d9ca6); }
.bar-good { background: linear-gradient(90deg, #3b82f6, #2563eb); }
.bar-average { background: linear-gradient(90deg, #f59e0b, #d97706); }
.bar-poor { background: linear-gradient(90deg, #ef4444, #b91c1c); }
.hour-value { width: 40px; font-size: 13px; color: #fff; font-weight: 600; }
.insight-box { padding: 16px; border-radius: 8px; border-left: 4px solid #fbbf24; background: rgba(251, 191, 36, 0.1); }
.insight-box h4 { color: #fbbf24; margin: 0 0 8px 0; font-size: 14px; }
.insight-box p { margin: 0; font-size: 13px; color: #e2e8f0; line-height: 1.5; }
.glass { background: rgba(30, 30, 45, 0.6); backdrop-filter: blur(10px); }
</style>
