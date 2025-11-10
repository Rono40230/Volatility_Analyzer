<template>
  <div v-if="result" class="analysis-panel">
    <div class="panel-header">
      <div class="header-title">
        <h2>🎯 Analyse :</h2>
        <select v-model="currentSymbol" @change="onSymbolChange" class="symbol-select">
          <option v-for="s in symbols" :key="s.symbol" :value="s.symbol">{{ s.symbol }}</option>
        </select>
      </div>
      <div class="badges">
        <span :class="['badge', 'recommendation', recommendationClass]">{{ formatRecommendation(result.recommendation) }}</span>
        <span :class="['badge', 'risk']">Risque: {{ formatRisk(result.risk_level) }}</span>
      </div>
    </div>

    <div class="confidence-section">
      <h3>Score de Confiance</h3>
      <div class="confidence-bar" :style="{ width: result.confidence_score + '%' }"></div>
      <span class="confidence-text">{{ result.confidence_score }}%</span>
    </div>

    <div class="metrics-grid">
      <div class="metric-card">
        <h4>📊 Volatilité Globale</h4>
        <div class="metric-value">{{ result.global_metrics.avg_volatility.toFixed(2) }} pips</div>
        <small>moyenne</small>
      </div>
      <div class="metric-card">
        <h4>📈 Trend</h4>
        <div class="metric-value">{{ result.global_metrics.trend_strength }}</div>
        <small>force</small>
      </div>
      <div class="metric-card">
        <h4>🎯 Win Rate</h4>
        <div class="metric-value">{{ (result.global_metrics.win_rate * 100).toFixed(1) }}%</div>
        <small>sessions</small>
      </div>
    </div>

    <div class="volatility-section">
      <h3>📊 Volatilité par Session</h3>
      <div class="session-stats">
        <div v-for="session in result.sessions" :key="session.session_name" class="session-stat">
          <div class="session-name">{{ session.session_name }}</div>
          <div class="stat-value">{{ session.volatility.toFixed(2) }} pips</div>
        </div>
      </div>
    </div>

    <div class="recommendations-section">
      <h3>💡 Recommandations</h3>
      <ul>
        <li v-for="(rec, idx) in result.recommendations" :key="idx">{{ rec }}</li>
      </ul>
    </div>
  </div>
  <div v-else class="loading">
    <p>Sélectionnez une paire pour analyser...</p>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface AnalysisResult {
  symbol: string
  period_start: string
  period_end: string
  recommendation: string
  risk_level: string
  confidence_score: number
  global_metrics: { avg_volatility: number; trend_strength: string; win_rate: number; total_candles: number }
  sessions: Array<{ session_name: string; volatility: number }>
  recommendations: string[]
}

const currentSymbol = ref<string>('')
const result = ref<AnalysisResult | null>(null)
const symbols = ref<Array<{ symbol: string; file_path: string }>>([])

onMounted(async () => {
  try {
    symbols.value = await invoke('load_symbols')
  } catch (err) {
    console.error('Erreur:', err)
  }
})

async function onSymbolChange() {
  if (!currentSymbol.value) {
    result.value = null
    return
  }
  try {
    result.value = await invoke('analyze_volatility', { symbol: currentSymbol.value })
  } catch (err) {
    console.error('Erreur analyse:', err)
  }
}

function formatRecommendation(rec: string): string {
  const map: { [key: string]: string } = {
    'BUY': '✅ ACHETER',
    'SELL': '⛔ VENDRE',
    'HOLD': '⏸️ ATTENDRE'
  }
  return map[rec] || rec
}

function formatRisk(risk: string): string {
  const map: { [key: string]: string } = {
    'HIGH': '🔴 ÉLEVÉ',
    'MEDIUM': '🟡 MOYEN',
    'LOW': '🟢 BAS'
  }
  return map[risk] || risk
}

const recommendationClass = (() => {
  if (result.value?.recommendation === 'BUY') return 'buy'
  if (result.value?.recommendation === 'SELL') return 'sell'
  return 'hold'
})()
</script>

<style scoped>
.analysis-panel { background: #161b22; padding: 30px; border-radius: 12px; border: 1px solid #30363d; }
.panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; }
.header-title { display: flex; align-items: center; gap: 15px; }
.header-title h2 { margin: 0; }
.symbol-select { padding: 8px 12px; border: 2px solid #30363d; background: #1a202c; color: #e2e8f0; border-radius: 6px; cursor: pointer; }
.badges { display: flex; gap: 10px; }
.badge { padding: 6px 12px; border-radius: 6px; font-weight: 600; font-size: 0.9em; color: white; }
.recommendation.buy { background: #10b981; }
.recommendation.sell { background: #dc2626; }
.recommendation.hold { background: #f59e0b; }
.badge.risk { background: #6b7280; }
.confidence-section { background: #1a202c; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
.confidence-bar { height: 8px; background: #667eea; border-radius: 4px; }
.confidence-text { color: #cbd5e0; font-size: 0.9em; }
.metrics-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 30px; }
.metric-card { background: #1a202c; padding: 15px; border-radius: 8px; border-left: 3px solid #667eea; }
.metric-card h4 { margin: 0 0 10px 0; color: #e2e8f0; }
.metric-value { font-size: 1.5em; font-weight: bold; color: #667eea; }
.metric-card small { color: #a0aec0; }
.volatility-section { background: #1a202c; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
.session-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 10px; margin-top: 15px; }
.session-stat { background: #2d3748; padding: 10px; border-radius: 6px; text-align: center; }
.session-name { color: #cbd5e0; font-size: 0.85em; }
.stat-value { color: #667eea; font-weight: bold; }
.recommendations-section { background: #1a202c; padding: 20px; border-radius: 8px; }
.recommendations-section h3 { color: #e2e8f0; margin-top: 0; }
.recommendations-section ul { list-style: none; padding: 0; margin: 0; }
.recommendations-section li { padding: 8px; background: #2d3748; border-left: 3px solid #667eea; color: #e2e8f0; margin-bottom: 8px; border-radius: 4px; }
.loading { text-align: center; padding: 40px; color: #a0aec0; }
</style>
