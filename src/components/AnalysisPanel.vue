<template>
  <div v-if="result" class="analysis-panel">
    <div class="panel-header">
      <div class="header-title-section">
        <div class="title-with-selector">
          <h2>🎯 Analyse :</h2>
          <select 
            v-model="currentSymbol" 
            @change="onSymbolChange"
            class="symbol-select-inline"
          >
            <option 
              v-for="symbol in symbols" 
              :key="symbol.symbol" 
              :value="symbol.symbol"
            >
              {{ symbol.symbol }}
            </option>
          </select>
        </div>
        <div class="analysis-metadata">
          <span class="metadata-item">
            <span class="metadata-icon">📅</span>
            <span class="metadata-label">Période:</span>
            <span class="metadata-value">{{ result.period_start }} → {{ result.period_end }}</span>
          </span>
          <span class="metadata-item">
            <span class="metadata-icon">📊</span>
            <span class="metadata-label">Bougies:</span>
            <span class="metadata-value">{{ result.global_metrics.total_candles.toLocaleString() }}</span>
          </span>
          <span class="metadata-item">
            <span class="metadata-icon">⏱️</span>
            <span class="metadata-label">Timeframe:</span>
            <span class="metadata-value">{{ result.timeframe }}</span>
          </span>
        </div>
      </div>
      <div class="header-badges">
        <span :class="['badge', 'recommendation', recommendationClass]">
          {{ formatRecommendation(result.recommendation) }}
        </span>
        <span :class="['badge', 'risk', riskClass]">
          Risque: {{ formatRisk(result.risk_level) }}
        </span>
      </div>
    </div>

    <div class="confidence-section">
      <h3>Score de Confiance</h3>
      <div class="confidence-bar-container" :title="tooltips.confidenceScore">
        <div 
          class="confidence-bar" 
          :style="{ width: `${result.confidence_score}%` }"
          :class="confidenceClass"
        ></div>
        <span class="confidence-value">{{ result.confidence_score.toFixed(1) }}%</span>
      </div>
    </div>

    <div class="metrics-grid">
      <div class="metric-card" :title="tooltips.atr">
        <div class="metric-icon">📈</div>
        <div class="metric-label">
          ATR Moyen
          <span class="info-icon">ℹ️</span>
        </div>
        <div class="metric-value">{{ formatNumber(result.global_metrics.mean_atr, 5) }}</div>
      </div>
      
      <div class="metric-card" :title="tooltips.volatility">
        <div class="metric-icon">📊</div>
        <div class="metric-label">
          Volatilité
          <span class="info-icon">ℹ️</span>
        </div>
        <div class="metric-value">{{ (result.global_metrics.mean_volatility * 100).toFixed(2) }}%</div>
      </div>
      
      <div class="metric-card" :title="tooltips.bodyRange">
        <div class="metric-icon">🎯</div>
        <div class="metric-label">
          Body Range
          <span class="info-icon">ℹ️</span>
        </div>
        <div class="metric-value">{{ result.global_metrics.mean_body_range.toFixed(1) }}%</div>
      </div>
      
      <div class="metric-card" :title="tooltips.tickQuality">
        <div class="metric-icon">⚡</div>
        <div class="metric-label">
          Tick Quality
          <span class="info-icon">ℹ️</span>
        </div>
        <div class="metric-value">{{ formatNumber(result.global_metrics.mean_tick_quality, 5) }}</div>
      </div>
      
      <div class="metric-card" :title="tooltips.noiseRatio">
        <div class="metric-icon">🔊</div>
        <div class="metric-label">
          Noise Ratio
          <span class="info-icon">ℹ️</span>
        </div>
        <div class="metric-value">{{ result.global_metrics.mean_noise_ratio.toFixed(2) }}</div>
      </div>
    </div>

    <div class="best-hours-section">
      <h3>⭐ Top 3 Meilleures Heures de Trading (UTC)</h3>
      <div class="hours-badges">
        <span 
          v-for="hour in result.best_hours" 
          :key="hour"
          class="hour-badge"
        >
          {{ formatHour(hour) }}
        </span>
      </div>
    </div>

    <!-- Section Événements Économiques Corrélés -->
    <div v-if="result.correlated_events && result.correlated_events.length > 0" class="correlated-events-section">
      <h3>📅 Événements Économiques Corrélés</h3>
      <p class="section-subtitle">
        Événements détectés pendant les périodes de haute volatilité
      </p>
      <div class="events-list">
        <div 
          v-for="(corr, index) in result.correlated_events" 
          :key="index"
          class="event-card"
          :class="impactClass(corr.event.impact)"
        >
          <div class="event-header">
            <span class="event-icon">📅</span>
            <span class="event-time">{{ formatEventTime(corr.event.event_time) }}</span>
            <span :class="['event-impact', impactClass(corr.event.impact)]">
              {{ corr.event.impact }}
            </span>
          </div>
          <div class="event-title">{{ corr.event.description }}</div>
          <div class="event-metrics">
            <div class="event-metric">
              <span class="metric-label">Heure de volatilité:</span>
              <span class="metric-value">{{ formatHour(corr.volatility_hour) }}</span>
            </div>
            <div class="event-metric">
              <span class="metric-label">Augmentation:</span>
              <span class="metric-value volatility-increase">+{{ corr.volatility_increase.toFixed(1) }}%</span>
            </div>
            <div class="event-metric">
              <span class="metric-label">Score corrélation:</span>
              <span class="metric-value">{{ corr.correlation_score.toFixed(1) }}</span>
            </div>
          </div>
          <div v-if="hasEconomicData(corr.event)" class="event-data">
            <span v-if="corr.event.actual !== null" class="data-item">
              Réel: <strong>{{ corr.event.actual }}</strong>
            </span>
            <span v-if="corr.event.forecast !== null" class="data-item">
              Prévu: {{ corr.event.forecast }}
            </span>
            <span v-if="corr.event.previous !== null" class="data-item">
              Précédent: {{ corr.event.previous }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { AnalysisResult, SymbolInfo } from '../stores/volatility'

const props = defineProps<{
  result: AnalysisResult | null
  symbols: SymbolInfo[]
}>()

const emit = defineEmits<{
  symbolSelected: [symbol: string]
}>()

const currentSymbol = ref(props.result?.symbol || '')

// Mettre à jour currentSymbol quand result change
watch(() => props.result?.symbol, (newSymbol) => {
  if (newSymbol) {
    currentSymbol.value = newSymbol
  }
})

function onSymbolChange() {
  if (currentSymbol.value) {
    emit('symbolSelected', currentSymbol.value)
  }
}

// Tooltips explicatifs pour chaque métrique
const tooltips = {
  atr: `ATR (Average True Range) - Moyenne de l'amplitude des mouvements
  
📊 UTILITÉ STRADDLE:
• Base de calcul pour votre Stop Loss
• Exemple: Si ATR = 0.00121, SL = 2.0 × ATR = 0.00242
• Plus l'ATR est élevé, plus les stops doivent être larges

✅ BON: > 0.001 (mouvements significatifs)
⚠️ MOYEN: 0.0005 - 0.001
❌ FAIBLE: < 0.0005 (peu de mouvement)`,

  volatility: `Volatilité % - Amplitude moyenne des variations de prix

📊 UTILITÉ STRADDLE:
• Mesure la "nervosité" de la paire
• Plus c'est élevé, plus le Straddle est profitable
• Indique l'amplitude des mouvements attendus

✅ EXCELLENT: > 30% (très volatil, idéal Straddle)
🟢 BON: 15-30% (volatilité correcte)
⚠️ MOYEN: 5-15% (peu de mouvement)
❌ FAIBLE: < 5% (éviter le Straddle)`,

  bodyRange: `Body Range % - Taille du corps des bougies vs mèches

📊 UTILITÉ STRADDLE:
• Mesure la force directionnelle des mouvements
• Corps large = mouvement franc (bon pour Straddle)
• Corps petit = indécision/whipsaw (danger!)

✅ EXCELLENT: > 50% (mouvements directionnels clairs)
🟢 BON: 30-50% (acceptable)
⚠️ ATTENTION: 10-30% (beaucoup de fausses cassures)
❌ DANGER: < 10% (ne PAS trader, trop de whipsaw)`,

  tickQuality: `Tick Quality - Qualité et complétude des données

📊 UTILITÉ STRADDLE:
• Vérifie la fiabilité des données historiques
• Détecte les gaps ou données manquantes
• Impact sur la fiabilité de l'analyse

✅ EXCELLENT: > 0.001 (données complètes)
🟢 BON: 0.0005 - 0.001
⚠️ MOYEN: 0.0001 - 0.0005 (vérifier source)
❌ FAIBLE: < 0.0001 (données peu fiables)`,

  noiseRatio: `Noise Ratio - Rapport Bruit/Signal

📊 UTILITÉ STRADDLE:
• Mesure les fausses cassures vs vrais mouvements
• Ratio faible = signal propre (bon!)
• Ratio élevé = beaucoup de bruit (danger!)

✅ EXCELLENT: < 2.0 (signal clair, peu de whipsaw)
🟢 BON: 2.0 - 3.0 (acceptable)
⚠️ ATTENTION: 3.0 - 5.0 (beaucoup de faux signaux)
❌ DANGER: > 5.0 (trop de bruit, éviter)`,

  totalCandles: `Total Bougies - Nombre de données analysées

📊 UTILITÉ STRADDLE:
• Plus il y a de données, plus l'analyse est fiable
• Minimum recommandé: 100,000 bougies
• Idéal: > 500,000 pour statistiques robustes

✅ EXCELLENT: > 500,000 (très fiable)
🟢 BON: 100,000 - 500,000
⚠️ MOYEN: 10,000 - 100,000
❌ INSUFFISANT: < 10,000 (peu fiable)`,

  confidenceScore: `Score de Confiance - Fiabilité globale de l'analyse (0-100%)

📊 COMMENT C'EST CALCULÉ:
• 25 pts : ATR significatif (>0.001)
• 25 pts : Body Range élevé (>50%)
• 20 pts : Tick Quality excellente (>0.001)
• 20 pts : Noise Ratio faible (<2.0)
• 10 pts : Volatilité raisonnable (<15%)

💡 INTERPRÉTATION:
✅ EXCELLENT (80-100%) : Conditions idéales pour trader
🟢 BON (60-80%) : Bonnes conditions, tradable
🟡 MOYEN (40-60%) : Conditions acceptables, prudence
🔴 FAIBLE (<40%) : Éviter de trader, trop risqué

📌 UTILITÉ STRADDLE:
Plus le score est élevé, plus votre stratégie Straddle a de chances de réussir sur cette paire et cette période horaire.`
}

const recommendationClass = computed(() => {
  if (!props.result) return ''
  const rec = props.result.recommendation
  if (rec.includes('Aggressive')) return 'aggressive'
  if (rec.includes('Normal')) return 'normal'
  if (rec.includes('Cautious')) return 'cautious'
  if (rec.includes('NoTrade')) return 'no-trade'
  return ''
})

const riskClass = computed(() => {
  if (!props.result) return ''
  const risk = props.result.risk_level.toLowerCase()
  if (risk.includes('high')) return 'high'
  if (risk.includes('medium')) return 'medium'
  return 'low'
})

const confidenceClass = computed(() => {
  if (!props.result) return ''
  const score = props.result.confidence_score
  if (score >= 80) return 'excellent'
  if (score >= 60) return 'good'
  if (score >= 40) return 'fair'
  return 'poor'
})

function formatRecommendation(rec: string): string {
  if (rec === 'ScalpAggressive') return 'Scalp Agressif'
  if (rec === 'ScalpNormal') return 'Scalp Normal'
  if (rec === 'ScalpCautious') return 'Scalp Prudent'
  if (rec === 'VeryCautious') return 'Très Prudent'
  if (rec === 'NoTrade') return 'Pas de Trading'
  return rec
}

function formatRisk(risk: string): string {
  if (risk === 'High') return 'Élevé'
  if (risk === 'Medium') return 'Moyen'
  if (risk === 'Low') return 'Faible'
  return risk
}

function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, '0')}:00`
}

function formatNumber(num: number, decimals: number): string {
  return num.toFixed(decimals)
}

function formatEventTime(dateTimeStr: string): string {
  const date = new Date(dateTimeStr)
  return date.toLocaleString('fr-FR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function impactClass(impact: string): string {
  const upper = impact.toUpperCase()
  if (upper === 'HIGH') return 'impact-high'
  if (upper === 'MEDIUM') return 'impact-medium'
  return 'impact-low'
}

function hasEconomicData(event: any): boolean {
  return event.actual !== null || event.forecast !== null || event.previous !== null
}
</script>

<style scoped>
.analysis-panel {
  background: #161b22;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
  border: 1px solid #30363d;
  margin-bottom: 2rem;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 2rem;
  border-bottom: 2px solid #30363d;
  padding-bottom: 1rem;
}

.header-title-section {
  flex: 1;
}

.title-with-selector {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.panel-header h2 {
  margin: 0;
  color: #e6edf3;
}

.symbol-select-inline {
  padding: 0.5rem 1rem;
  font-size: 1.1rem;
  font-weight: bold;
  border: 2px solid #4a5568;
  border-radius: 8px;
  background: #2d3748;
  color: #3b82f6;
  cursor: pointer;
  transition: all 0.3s;
  min-width: 150px;
}

.symbol-select-inline:hover {
  border-color: #3b82f6;
  background: #374151;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.symbol-select-inline:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.3);
}

.analysis-metadata {
  display: flex;
  gap: 1.5rem;
  flex-wrap: wrap;
  margin-top: 0.75rem;
}

.metadata-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: #0d1117;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  border: 1px solid #30363d;
  font-size: 0.9rem;
}

.metadata-icon {
  font-size: 1.1rem;
}

.metadata-label {
  color: #8b949e;
  font-weight: 500;
}

.metadata-value {
  color: #e6edf3;
  font-weight: bold;
}

.header-badges {
  display: flex;
  gap: 0.5rem;
  flex-shrink: 0;
}

.badge {
  padding: 0.5rem 1rem;
  border-radius: 20px;
  font-weight: bold;
  font-size: 0.9rem;
}

.recommendation.aggressive {
  background: #22c55e;
  color: white;
}

.recommendation.normal {
  background: #3b82f6;
  color: white;
}

.recommendation.cautious {
  background: #f59e0b;
  color: white;
}

.recommendation.no-trade {
  background: #ef4444;
  color: white;
}

.risk.high {
  background: #2d1215;
  color: #f97583;
  border: 1px solid #dc3545;
}

.risk.medium {
  background: #2d2715;
  color: #f9c513;
  border: 1px solid #ffc107;
}

.risk.low {
  background: #1a2d1f;
  color: #3fb950;
  border: 1px solid #22c55e;
}

.confidence-section {
  margin-bottom: 2rem;
}

.confidence-section h3 {
  margin: 0 0 1rem 0;
  color: #8b949e;
}

.confidence-bar-container {
  position: relative;
  width: 100%;
  height: 40px;
  background: #0d1117;
  border-radius: 20px;
  overflow: hidden;
  border: 1px solid #30363d;
}

.confidence-bar {
  height: 100%;
  transition: width 0.5s ease;
  border-radius: 20px;
}

.confidence-bar.excellent {
  background: linear-gradient(90deg, #22c55e, #10b981);
}

.confidence-bar.good {
  background: linear-gradient(90deg, #3b82f6, #2563eb);
}

.confidence-bar.fair {
  background: linear-gradient(90deg, #f59e0b, #d97706);
}

.confidence-bar.poor {
  background: linear-gradient(90deg, #ef4444, #dc2626);
}

.confidence-value {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-weight: bold;
  color: #e6edf3;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.5);
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.metric-card {
  background: linear-gradient(135deg, #0d1117 0%, #161b22 100%);
  padding: 1.5rem;
  border-radius: 12px;
  text-align: center;
  border: 1px solid #30363d;
  cursor: help;
  transition: all 0.3s ease;
  position: relative;
}

.metric-card:hover {
  border-color: #3b82f6;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
  transform: translateY(-2px);
}

.metric-icon {
  font-size: 2rem;
  margin-bottom: 0.5rem;
}

.metric-label {
  font-size: 0.9rem;
  color: #8b949e;
  margin-bottom: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.3rem;
}

.info-icon {
  font-size: 0.75rem;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.metric-card:hover .info-icon {
  opacity: 1;
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.1); }
}

.metric-value {
  font-size: 1.5rem;
  font-weight: bold;
  color: #e6edf3;
}

.best-hours-section h3 {
  margin: 0 0 1rem 0;
  color: #8b949e;
}

.hours-badges {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.hour-badge {
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  color: white;
  padding: 0.75rem 1.5rem;
  border-radius: 25px;
  font-weight: bold;
  font-size: 1.1rem;
}

/* Événements Économiques Corrélés */
.correlated-events-section {
  margin-top: 2rem;
  padding-top: 2rem;
  border-top: 2px solid #30363d;
}

.correlated-events-section h3 {
  margin: 0 0 0.5rem 0;
  color: #e6edf3;
  font-size: 1.5rem;
}

.section-subtitle {
  color: #8b949e;
  font-size: 0.9rem;
  margin-bottom: 1.5rem;
}

.events-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.event-card {
  background: #0d1117;
  border-left: 4px solid #3b82f6;
  border-radius: 8px;
  padding: 1.5rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  transition: transform 0.2s, box-shadow 0.2s;
  border: 1px solid #30363d;
}

.event-card:hover {
  transform: translateX(4px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.5);
}

.event-card.impact-high {
  border-left-color: #dc2626;
  background: linear-gradient(to right, #2d1215 0%, #0d1117 10%);
}

.event-card.impact-medium {
  border-left-color: #f59e0b;
  background: linear-gradient(to right, #2d2715 0%, #0d1117 10%);
}

.event-card.impact-low {
  border-left-color: #10b981;
  background: linear-gradient(to right, #1a2d1f 0%, #0d1117 10%);
}

.event-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.75rem;
}

.event-icon {
  font-size: 1.5rem;
}

.event-time {
  font-size: 0.9rem;
  color: #8b949e;
  font-weight: 500;
}

.event-impact {
  padding: 0.25rem 0.75rem;
  border-radius: 12px;
  font-size: 0.75rem;
  font-weight: bold;
  text-transform: uppercase;
  margin-left: auto;
}

.event-impact.impact-high {
  background: #dc2626;
  color: white;
}

.event-impact.impact-medium {
  background: #f59e0b;
  color: white;
}

.event-impact.impact-low {
  background: #10b981;
  color: white;
}

.event-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: #e6edf3;
  margin-bottom: 1rem;
}

.event-metrics {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 0.75rem;
  margin-bottom: 1rem;
}

.event-metric {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.event-metric .metric-label {
  font-size: 0.8rem;
  color: #8b949e;
}

.event-metric .metric-value {
  font-size: 1rem;
  font-weight: 600;
  color: #e6edf3;
}

.event-metric .volatility-increase {
  color: #f97583;
  font-weight: bold;
}

.event-data {
  display: flex;
  gap: 1.5rem;
  padding-top: 1rem;
  border-top: 1px solid #30363d;
  font-size: 0.9rem;
}

.data-item {
  color: #8b949e;
}

.data-item strong {
  color: #e6edf3;
  font-weight: 700;
}
</style>
