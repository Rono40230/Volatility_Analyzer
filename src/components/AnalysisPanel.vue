<template>
  <div v-if="props.result" class="analysis-panel">
    <!-- DEBUG: Vérifier que result est chargé -->
    <div v-if="props.result" style="display: none;">{{ console.log('AnalysisPanel result:', props.result) }}</div>
    
    <div class="panel-header">
      <div class="header-title">
        <h2>🎯 Analyse: {{ props.result.symbol }}</h2>
        <select :value="currentSymbol" @change="(e) => onSymbolChange((e.target as HTMLSelectElement).value)" class="symbol-select">
          <option v-for="s in symbols" :key="s.symbol" :value="s.symbol">{{ s.symbol }}</option>
        </select>
      </div>
      <div class="badges">
        <span 
          :class="['badge', 'recommendation', recommendationClass]"
          :title="getRecommendationTooltip(props.result.recommendation)"
        >
          {{ formatRecommendation(props.result.recommendation) }}
        </span>
        <span 
          :class="['badge', 'risk', getRiskClass(props.result.risk_level)]"
          :title="getRiskTooltip(props.result.risk_level)"
        >
          {{ formatRisk(props.result.risk_level) }}
        </span>
      </div>
    </div>

    <div class="confidence-section">
      <MetricTooltip title="Score de Confiance">
        <h3>Score de Confiance 🎯</h3>
        <template #definition>
          <div class="tooltip-section-title">Définition</div>
          <div class="tooltip-section-text">Mesure 0-100 : à quel point tu peux CONFIER la stratégie STRADDLE scalping à cette paire pendant cette période. Plus le score est élevé, plus les conditions sont stables et prévisibles.</div>
        </template>
        <template #usage>
          <div class="tooltip-section-title">Formule (max 100 points)</div>
          <div class="tooltip-section-text">
            <strong>ATR (30 pts)</strong> - Volatilité soutenue : >25 pips = 30 pts | 15-25 pips = 25 pts<br/>
            <strong>Body Range (25 pts)</strong> - Directionnalité : >45% = 25 pts | 35-45% = 20 pts<br/>
            <strong>Volatilité (25 pts)</strong> - Bonus mouvement : >30% = 25 pts | 20-30% = 20 pts<br/>
            <strong>Noise Ratio (10 pts)</strong> - Signal/bruit : <2.0 = 10 pts | <3.0 = 7 pts<br/>
            <strong>Breakout % (10 pts)</strong> - Cassures : >15% = 10 pts | >10% = 7 pts<br/>
            <strong>Bonus Données (5 pts)</strong> - Si >100k candles = 5 pts
          </div>
        </template>
        <template #scoring>
          <div class="tooltip-section-title">Interprétation</div>
          <div class="tooltip-section-text">
            <strong>80-100</strong> ✅ EXCELLENT - Scalpe agressivement<br/>
            <strong>65-80</strong> 🟢 BON - Scalpe normalement<br/>
            <strong>50-65</strong> 🟡 PRUDENT - Scalpe avec stop serrés<br/>
            <strong>35-50</strong> 🟠 RISKY - Très prudent, breakouts only<br/>
            <strong>0-35</strong> ❌ MAUVAIS - Ne pas trader
          </div>
        </template>
      </MetricTooltip>
      <div class="confidence-bar" :style="{ width: props.result.confidence_score + '%' }"></div>
      <span class="confidence-text">{{ props.result.confidence_score.toFixed(0) }}/100</span>
    </div>

    <div class="metrics-grid">
      <div class="metric-card">
        <MetricTooltip title="Nombre de Bougies">
          <h4>🎯 Bougies</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Quantité totale de bougies analysées sur la période.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Plus de données signifie une analyse plus fiable. Minimum recommandé: 100 bougies pour obtenir des résultats statistiquement significatifs.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&gt;500 = Excellent | &gt;200 = Bon | &gt;100 = Acceptable | &lt;100 = Données insuffisantes</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('bougies', props.result.global_metrics.total_candles)]">{{ props.result.global_metrics.total_candles }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="ATR Moyen">
          <h4>📊 ATR Moyen</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Average True Range - Mesure la volatilité moyenne sur 14 périodes. Représente l'amplitude moyenne des mouvements.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Aide à définir les stops loss et take profit. Un ATR élevé = grands mouvements possibles = risque plus important. Utile pour ajuster la taille des positions.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&gt;0.001 = Excellent | &gt;0.0005 = Bon | &gt;0.0001 = Acceptable | &lt;0.0001 = Très faible volatilité</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('atr', props.result.global_metrics.mean_atr)]">{{ props.result.global_metrics.mean_atr.toFixed(5) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Volatilité Globale">
          <h4>📈 Volatilité</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Pourcentage moyen de variation des prix (écart-type des rendements).</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Volatilité élevée = plus d'opportunités mais plus de risques. Volatilité basse = tendances plus stables mais moins de mouvements. Adapter le style de trading à la volatilité.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&lt;5% = Très basse (scalping) | 5-15% = Normale (swing) | &gt;15% = Élevée (prudence)</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('volatility', props.result.global_metrics.mean_volatility)]">{{ (props.result.global_metrics.mean_volatility * 100).toFixed(2) }}%</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Body Range">
          <h4>📦 Body Range</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Pourcentage du corps de la bougie (distance open-close) par rapport à l'amplitude totale (high-low).</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Body Range élevé = bougies plus directionnelles et décisives. Body Range faible = bougies indécises avec beaucoup de mèches. Préférer les corps forts pour les signaux clairs.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&gt;50% = Excellent (très directif) | &gt;30% = Bon | &gt;10% = Acceptable | &lt;10% = Très indécis</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('bodyrange', props.result.global_metrics.mean_body_range)]">{{ props.result.global_metrics.mean_body_range.toFixed(1) }}%</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Tick Quality">
          <h4>✨ Tick Quality</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Qualité des ticks - taille moyenne des mouvements de prix unitaires. Mesure la liquidité et la granularité des données.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Tick Quality élevé = meilleure liquidité et données plus fiables. Important pour le scalping et les stratégies haute fréquence. Indique l'existence de market makers actifs.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&gt;0.001 = Excellent | &gt;0.0005 = Bon | &gt;0.0001 = Acceptable | Plus élevé = Meilleur</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('tickquality', props.result.global_metrics.mean_tick_quality)]">{{ props.result.global_metrics.mean_tick_quality.toFixed(5) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Noise Ratio">
          <h4>🔊 Noise Ratio</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Ratio bruit/signal - compare les mouvements intra-bougie au mouvement directionnel net. Plus bas = moins de bruit.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Bruit élevé = plus de faux signaux et de whipsaws. Bruit bas = tendances plus nettes. Critère essentiel pour éviter de trader dans du bruit chaotique.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&lt;2.0 = Excellent | &lt;3.0 = Bon | &gt;3.0 = Élevé (à éviter) | &gt;5.0 = Très chaotique</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('noiseratio', props.result.global_metrics.mean_noise_ratio)]">{{ props.result.global_metrics.mean_noise_ratio.toFixed(2) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Volume Imbalance">
          <h4>⚖️ Volume Imbalance</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Déséquilibre entre les volumes d'achat et de vente. Mesure la domination d'un côté du marché.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Imbalance élevé = forces acheteuses ou vendeuses dominantes = signaux de tendance forts. Imbalance équilibré (proche de 1.0) = marché dans l'indécision.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">≈1.0 = Parfaitement équilibré | &gt;1.5 = Déséquilibre marqué | &lt;0.5 = Forte dominance vendeur</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('volumeimbalance', props.result.global_metrics.mean_volume_imbalance)]">{{ props.result.global_metrics.mean_volume_imbalance.toFixed(4) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Breakout %">
          <h4>🚀 Breakout %</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Pourcentage de bougies qui sortent des niveaux de support/résistance (breakouts).</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Utilité pour le Trading</div>
            <div class="tooltip-section-text">Breakout % élevé = marché actif avec beaucoup de cassures. Breakout % bas = marché consolidé. Détermine le style: range trading ou breakout trading.</div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Interprétation</div>
            <div class="tooltip-section-text">&lt;10% = Peu de breakouts (range trading) | 10-30% = Modéré (swing) | &gt;30% = Très actif (trendy)</div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('breakout', props.result.global_metrics.mean_breakout_percentage)]">{{ props.result.global_metrics.mean_breakout_percentage.toFixed(1) }}%</div>
      </div>
    </div>

    <div class="color-legend">
      <div class="legend-grid">
        <div class="legend-item">
          <div class="legend-color excellent"></div>
          <div class="legend-text">
            <span>Métrique très bonne, conditions optimales</span>
          </div>
        </div>
        <div class="legend-item">
          <div class="legend-color good"></div>
          <div class="legend-text">
            <span>Métrique satisfaisante, conditions acceptables</span>
          </div>
        </div>
        <div class="legend-item">
          <div class="legend-color acceptable"></div>
          <div class="legend-text">
            <span>Métrique à la limite, à surveiller</span>
          </div>
        </div>
        <div class="legend-item">
          <div class="legend-color poor"></div>
          <div class="legend-text">
            <span>Métrique insuffisante, déconseillé</span>
          </div>
        </div>
      </div>
    </div>
  </div>
  <div v-else class="loading">
    <p>Sélectionnez une paire pour analyser...</p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import MetricTooltip from './MetricTooltip.vue'

interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_tick_quality: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  total_candles: number
}

interface HourlyStats {
  hour: number
  candle_count: number
  atr_mean: number
}

interface AnalysisResult {
  symbol: string
  period_start: string
  period_end: string
  timeframe: string
  recommendation: string
  risk_level: string
  confidence_score: number
  global_metrics: GlobalMetrics
  hourly_stats: HourlyStats[]
  best_hours: number[]
}

const props = defineProps<{
  result: AnalysisResult | null
  symbols: Array<{ symbol: string; file_path: string }>
}>()

const emit = defineEmits<{
  symbolSelected: [symbol: string]
}>()

const store = useVolatilityStore()
const currentSymbol = computed(() => props.result?.symbol || '')
const symbols = ref<Array<{ symbol: string; file_path: string }>>([])
const { onPairDataRefresh } = useDataRefresh()

const unsubscribe = onPairDataRefresh(() => {
  store.loadSymbols()
})

onMounted(async () => {
  try {
    symbols.value = props.symbols || await invoke('load_symbols')
  } catch (err) {
    console.error('Erreur:', err)
  }
})

onBeforeUnmount(() => {
  unsubscribe()
})

// Écouter les changements du store
watch(() => store.symbols, (newSymbols) => {
  symbols.value = newSymbols
}, { deep: true })

function onSymbolChange(newSymbol: string) {
  if (newSymbol && newSymbol !== props.result?.symbol) {
    emit('symbolSelected', newSymbol)
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

function getRecommendationTooltip(rec: string): string {
  const tooltips: { [key: string]: string } = {
    'BUY': 'ACHETER - Le marché offre une configuration favorable pour un achat',
    'SELL': 'VENDRE - Le marché offre une configuration favorable pour une vente',
    'HOLD': 'ATTENDRE - Le marché n\'offre pas de configuration clairement favorable'
  }
  return tooltips[rec] || rec
}

function getRiskClass(risk: string): string {
  const map: { [key: string]: string } = {
    'HIGH': 'high',
    'MEDIUM': 'medium',
    'LOW': 'low'
  }
  return map[risk] || ''
}

function getRiskTooltip(risk: string): string {
  const tooltips: { [key: string]: string } = {
    'HIGH': 'Risque ÉLEVÉ - Volatilité > 15% ou conditions instables. À approcher avec prudence.',
    'MEDIUM': 'Risque MOYEN - Volatilité 5-15% avec conditions acceptables. Gestion stricte du risque recommandée.',
    'LOW': 'Risque BAS - Volatilité < 5% avec conditions stables. Favorise les positions plus agressives.'
  }
  return tooltips[risk] || risk
}

const recommendationClass = computed(() => {
  if (props.result?.recommendation === 'BUY') return 'buy'
  if (props.result?.recommendation === 'SELL') return 'sell'
  return 'hold'
})

// Fonctions de scoring pour les métriques
function getMetricQuality(metric: string, value: number): string {
  if (!props.result) return 'neutral'
  
  switch (metric) {
    case 'bougies':
      if (value > 500) return 'excellent'
      if (value > 200) return 'good'
      if (value > 100) return 'acceptable'
      return 'poor'
    
    case 'atr':
      if (value > 0.001) return 'excellent'
      if (value > 0.0005) return 'good'
      if (value > 0.0001) return 'acceptable'
      return 'poor'
    
    case 'volatility':
      if (value >= 0.05 && value <= 0.15) return 'excellent'
      if ((value >= 0.03 && value < 0.05) || (value > 0.15 && value <= 0.25)) return 'good'
      if ((value >= 0.01 && value < 0.03) || (value > 0.25 && value <= 0.35)) return 'acceptable'
      return 'poor'
    
    case 'bodyrange':
      if (value > 50) return 'excellent'
      if (value > 30) return 'good'
      if (value > 10) return 'acceptable'
      return 'poor'
    
    case 'tickquality':
      if (value > 0.001) return 'excellent'
      if (value > 0.0005) return 'good'
      if (value > 0.0001) return 'acceptable'
      return 'poor'
    
    case 'noiseratio':
      if (value < 2.0) return 'excellent'
      if (value < 3.0) return 'good'
      if (value < 5.0) return 'acceptable'
      return 'poor'
    
    case 'volumeimbalance':
      const imbalanceScore = Math.abs(value - 1.0)
      if (imbalanceScore < 0.2) return 'excellent'
      if (imbalanceScore < 0.5) return 'good'
      if (imbalanceScore < 1.5) return 'acceptable'
      return 'poor'
    
    case 'breakout':
      if (value < 10) return 'good' // Range trading favorable
      if (value >= 10 && value <= 30) return 'excellent'
      if (value > 30 && value <= 50) return 'good'
      return 'acceptable'
    
    default:
      return 'neutral'
  }
}

function getColorClass(metric: string, value: number): string {
  return `metric-${getMetricQuality(metric, value)}`
}
</script>

<style scoped>
.analysis-panel { background: #161b22; padding: 30px; border-radius: 12px; border: 1px solid #30363d; }
.panel-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 30px; }
.header-title { display: flex; align-items: center; gap: 15px; }
.header-title h2 { margin: 0; }
.symbol-select { padding: 8px 12px; border: 2px solid #30363d; background: #1a202c; color: #000000; border-radius: 6px; cursor: pointer; font-weight: 600; }
.symbol-select option { background: #ffffff; color: #000000; }
.badges { display: flex; gap: 10px; }
.badge { padding: 8px 16px; border-radius: 6px; font-weight: 600; font-size: 0.9em; color: white; cursor: help; transition: all 0.2s; border: 2px solid transparent; }
.badge:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
.recommendation.buy { background: linear-gradient(135deg, #10b981 0%, #059669 100%); border-color: #047857; }
.recommendation.sell { background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%); border-color: #991b1b; }
.recommendation.hold { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); border-color: #b45309; }
.badge.risk.low { background: linear-gradient(135deg, #22c55e 0%, #16a34a 100%); border-color: #15803d; }
.badge.risk.medium { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); border-color: #b45309; }
.badge.risk.high { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); border-color: #b91c1c; }
.confidence-section { background: #1a202c; padding: 20px; border-radius: 8px; margin-bottom: 20px; }
.confidence-section h3 { margin: 0 0 15px 0; }
.confidence-bar { height: 8px; background: #667eea; border-radius: 4px; margin-bottom: 8px; }
.confidence-text { color: #cbd5e0; font-size: 0.9em; }
.metrics-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 30px; }
.metric-card { background: #1a202c; padding: 15px; border-radius: 8px; border-left: 3px solid #667eea; }
.metric-card h4 { margin: 0 0 10px 0; color: #e2e8f0; }
.metric-value { font-size: 1.5em; font-weight: bold; transition: color 0.3s ease; }

/* Couleurs dynamiques pour les métriques */
.metric-value.metric-excellent { color: #10b981; text-shadow: 0 0 8px rgba(16, 185, 129, 0.3); }
.metric-value.metric-good { color: #3b82f6; text-shadow: 0 0 8px rgba(59, 130, 246, 0.3); }
.metric-value.metric-acceptable { color: #f59e0b; text-shadow: 0 0 8px rgba(245, 158, 11, 0.3); }
.metric-value.metric-poor { color: #ef4444; text-shadow: 0 0 8px rgba(239, 68, 68, 0.3); }
.metric-value.metric-neutral { color: #667eea; text-shadow: 0 0 8px rgba(102, 126, 234, 0.3); }

/* Légende des couleurs */
.metric-card:has(.metric-excellent) {
  border-left-color: #10b981;
}
.metric-card:has(.metric-good) {
  border-left-color: #3b82f6;
}
.metric-card:has(.metric-acceptable) {
  border-left-color: #f59e0b;
}
.metric-card:has(.metric-poor) {
  border-left-color: #ef4444;
}

/* Légende des couleurs */
.color-legend { background: #1a202c; padding: 20px; border-radius: 8px; border: 1px solid #30363d; margin-top: 30px; }
.legend-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; }
.legend-item { display: flex; gap: 10px; align-items: center; }
.legend-color { width: 16px; height: 16px; border-radius: 3px; flex-shrink: 0; }
.legend-color.excellent { background: linear-gradient(135deg, #10b981 0%, #059669 100%); box-shadow: 0 0 8px rgba(16, 185, 129, 0.3); }
.legend-color.good { background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%); box-shadow: 0 0 8px rgba(59, 130, 246, 0.3); }
.legend-color.acceptable { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); box-shadow: 0 0 8px rgba(245, 158, 11, 0.3); }
.legend-color.poor { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); box-shadow: 0 0 8px rgba(239, 68, 68, 0.3); }
.legend-text { flex: 1; }
.legend-text span { color: #a0aec0; font-size: 0.85em; display: block; }

.loading { text-align: center; padding: 40px; color: #a0aec0; }
</style>
