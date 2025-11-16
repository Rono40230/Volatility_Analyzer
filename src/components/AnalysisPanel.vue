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
          <div class="tooltip-section-text">Mesure 0-100 : probabilité de SUCCESS pour stratégie STRADDLE scalping. Score = synthèse de TOUS les métriques volatilité + signal-qualité.</div>
        </template>
        <template #usage>
          <div class="tooltip-section-title">📊 Facteurs (100 pts max)</div>
          <div class="tooltip-section-text">
            <strong>ATR (30 pts):</strong> Volatilité soutenue - >25 pips = 30 pts<br/>
            <strong>Body Range (25 pts):</strong> Directionnalité - >45% = 25 pts<br/>
            <strong>Volatilité (25 pts):</strong> Bonus mouvement - >30% = 25 pts<br/>
            <strong>Noise Ratio (10 pts):</strong> Pureté signal - <2.0 = 10 pts (CRUCIAL!)<br/>
            <strong>Breakout % (10 pts):</strong> Activité - >15% = 10 pts
          </div>
        </template>
        <template #scoring>
          <div class="tooltip-section-title">💡 Interprétation & Action</div>
          <div class="tooltip-section-text">
            <strong>⭐ 80-100</strong> ✅ EXCELLENT - Conditions PARFAITES scalpe agressif SANS limite<br/>
            <strong>🟢 65-80</strong> BON - Scalpe standard, appétit moyen, respect stop<br/>
            <strong>🟡 50-65</strong> PRUDENT - Scalpe PETIT volumes, stop serrés, position sizes réduits<br/>
            <strong>🟠 35-50</strong> RISKY - Breakouts SEULEMENT, ignorer scalp intra-candle<br/>
            <strong>❌ 0-35</strong> MAUVAIS - ATTENDRE, pas assez conditions, recherche autre paire
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
            <div class="tooltip-section-text">Average True Range (14 périodes) - Mesure la volatilité réelle en écartant les spikes isolés.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📈 Interprétation Forex M1</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;0.00025 (25+ pips):</strong> Excellent - Scalpe agressivement, bons mouvements constants<br>
              🟡 <strong>0.00015-0.00025:</strong> Bon - Scalpe normalement, volatilité fiable<br>
              🟠 <strong>0.00010-0.00015:</strong> Moyen - Stop serré obligatoire, mouvement limité<br>
              🔴 <strong>&lt;0.00010:</strong> Mauvais - Attendre, trop peu de volatilité
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Combine avec:</strong> Body Range &gt;40% → Signal confirmé<br>
              ⚠️ <strong>ATR faible MAIS BodyRange élevé:</strong> Piège possible (spikes isolés)<br>
              🎯 <strong>Action:</strong> ATR élevé + bruit bas = Scalpe agressif sans limite
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('atr', props.result.global_metrics.mean_atr)]">{{ props.result.global_metrics.mean_atr.toFixed(5) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Volatilité Globale">
          <h4>📈 Volatilité</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Écart-type des rendements (%) - Mesure la variation moyenne des prix sur la période.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation Scalping M1</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;30%:</strong> Excellent - Crypto-like volatility, mouvements constants<br>
              🟡 <strong>15-30%:</strong> Bon - Volatilité normale forex, scalpe standard<br>
              🟠 <strong>5-15%:</strong> Moyen - Scalpe très serré, patience requise<br>
              🔴 <strong>&lt;5%:</strong> Mauvais - Trop peu de mouvement, attendre
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Scalping actif si:</strong> Vol &gt;15% + ATR élevé + Bruit faible<br>
              ⚠️ <strong>Différent d'ATR:</strong> Vol mesure variation %, ATR mesure pips<br>
              🎯 <strong>Action:</strong> Combine avec Breakout% pour stratégie
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('volatility', props.result.global_metrics.mean_volatility)]">{{ (props.result.global_metrics.mean_volatility * 100).toFixed(2) }}%</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Body Range">
          <h4>📦 Body Range</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">% du corps de bougie (open-close) vs amplitude totale (high-low) - Mesure la directionnalité.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation Scalping</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;45%:</strong> Excellent - Bougies directionnelles, signal/bruit élevé<br>
              🟡 <strong>35-45%:</strong> Bon - Assez directif pour scalpe normal<br>
              🟠 <strong>15-35%:</strong> Moyen - Bougies indécises, bruit modéré<br>
              🔴 <strong>&lt;15%:</strong> Mauvais - Bougies indécises avec longues mèches
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>BodyRange élevé = signaux clairs</strong> (moins de faux mouvements)<br>
              ⚠️ <strong>BodyRange faible MAIS ATR élevé:</strong> Spikes = à éviter<br>
              🎯 <strong>Meilleur combo:</strong> BodyRange &gt;40% + NoiseRatio &lt;2.0 = Scalpe sûr
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('bodyrange', props.result.global_metrics.mean_body_range)]">{{ props.result.global_metrics.mean_body_range.toFixed(1) }}%</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Tick Quality">
          <h4>✨ Tick Quality</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Taille moyenne des mouvements de prix unitaires = liquidity quality. Mesure l'existence de market makers et la granularité des données tick.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation (Scalping M1)</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;0.001 (10+ pips):</strong> Excellent - Très liquide, spreads serrés, scalpe sûr<br>
              🟡 <strong>0.0005-0.001:</strong> Bon - Liquide, spreads acceptables<br>
              🟠 <strong>0.0001-0.0005:</strong> Moyen - Spreads plus larges, frais élevés<br>
              🔴 <strong>&lt;0.0001:</strong> Mauvais - Très peu liquide, spreads prohibitifs
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Tick Quality élevé = meilleure rentabilité scalping</strong> (spreads faibles)<br>
              ⚠️ <strong>Si &lt;0.0001:</strong> Les frais de trading dévorent les profits<br>
              🎯 <strong>Excellent combo:</strong> Tick Quality &gt;0.001 + NoiseRatio &lt;2.0 = Scalpe hyper-rentable
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('tickquality', props.result.global_metrics.mean_tick_quality)]">{{ props.result.global_metrics.mean_tick_quality.toFixed(5) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Noise Ratio">
          <h4>🔊 Noise Ratio</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Ratio bruit/signal = (Intra-candle range) / (Net directional move) - Plus bas = plus propre.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation (CRITIQUE pour Scalping)</div>
            <div class="tooltip-section-text">
              🟢 <strong>&lt;2.0:</strong> Excellent - Signal très propre, tendance claire = SCALPE!<br>
              🟡 <strong>2.0-3.0:</strong> Bon - Signal acceptable mais attention aux whipsaws<br>
              🟠 <strong>3.0-4.0:</strong> Moyen - Bruit modéré, beaucoup de faux signaux<br>
              🔴 <strong>&gt;4.0:</strong> Mauvais - Chaos total, NE PAS TRADER
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading (CRUCIAL!)</div>
            <div class="tooltip-section-text">
              ✅ <strong>Si NoiseRatio &lt;2.0:</strong> C'est TON meilleur moment de scalpe<br>
              ⚠️ <strong>Si &gt;3.0:</strong> Ignore ATR/Vol, c'est du bruit, ATTENDS<br>
              🎯 <strong>Golden combo:</strong> NoiseRatio &lt;2.0 + BodyRange &gt;40% + ATR élevé = JACKPOT
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('noiseratio', props.result.global_metrics.mean_noise_ratio)]">{{ props.result.global_metrics.mean_noise_ratio.toFixed(2) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Volume Imbalance">
          <h4>⚖️ Volume Imbalance</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Déséquilibre acheteurs vs vendeurs = Ratio (volume acheteur) / (volume vendeur). Mesure qui contrôle le marché.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation (Scalping & Tendance)</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;2.0 ou &lt;0.5:</strong> Excellent - Déséquilibre FORT = Tendance claire, scalpe facile<br>
              🟡 <strong>1.5-2.0 ou 0.5-0.67:</strong> Bon - Déséquilibre modéré, tendance visible<br>
              🟠 <strong>1.0-1.5 ou 0.67-1.0:</strong> Moyen - Équilibre, indécision, beaucoup de whipsaws<br>
              🔴 <strong>≈1.0:</strong> Mauvais - Marché indécis, NE PAS SCALPER
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Si &gt;1.5 ou &lt;0.67:</strong> Trading directionnel facile, follow the trend<br>
              ⚠️ <strong>Si ≈1.0:</strong> ATTENDS, pas d'avantage directionnel<br>
              🎯 <strong>Best combo:</strong> Imbalance &gt;2.0 + NoiseRatio &lt;2.0 = Tendance très claire
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('volumeimbalance', props.result.global_metrics.mean_volume_imbalance)]">{{ props.result.global_metrics.mean_volume_imbalance.toFixed(4) }}</div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Breakout %">
          <h4>🚀 Breakout %</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">% de bougies sortant du range (cassure de support/résistance) - Mesure l'activité directionelle.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation Stratégie</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;20%:</strong> Excellent - Marché actif, beaucoup de cassures = TREND/BREAKOUT<br>
              🟡 <strong>10-20%:</strong> Bon - Activité modérée, scalpe avec cassures<br>
              🟠 <strong>5-10%:</strong> Moyen - Peu de cassures = RANGE TRADING préféré<br>
              🔴 <strong>&lt;5%:</strong> Mauvais - Très peu de mouvement, attendre
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Si &gt;15%:</strong> Trade les breakouts, IGNORÉ les ranges<br>
              ⚠️ <strong>Si &lt;10%:</strong> Range trading UNIQUEMENT, pas de breakout<br>
              🎯 <strong>Combine avec:</strong> Volatility % pour style adapté
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('breakout', props.result.global_metrics.mean_breakout_percentage)]">{{ props.result.global_metrics.mean_breakout_percentage.toFixed(1) }}%</div>
      </div>
    </div>

    <div class="color-legend-container">
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
      <button class="analysis-btn" @click="openAnalysisModal" title="Ouvrir l'analyse détaillée des métriques">
        📊 Analyse des métriques
      </button>
    </div>
  </div>
  <div v-else class="loading">
    <p>Sélectionnez une paire pour analyser...</p>
  </div>

  <!-- Analysis Modal -->
  <MetricsAnalysisModal
    :is-open="isAnalysisModalOpen"
    :analysis-result="(props.result as any)"
    @close="isAnalysisModalOpen = false"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch, onBeforeUnmount } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import MetricTooltip from './MetricTooltip.vue'
import MetricsAnalysisModal from './MetricsAnalysisModal.vue'

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
const isAnalysisModalOpen = ref(false)
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

// Fonction pour ouvrir la modal d'analyse
function openAnalysisModal() {
  isAnalysisModalOpen.value = true
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

/* Légende des couleurs et bouton d'analyse */
.color-legend-container {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
  margin-top: 30px;
}

.color-legend {
  flex: 1;
  background: #1a202c;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #30363d;
}

.analysis-btn {
  background: linear-gradient(135deg, #64c8ff 0%, #3b82f6 100%);
  color: #1a1a2e;
  border: none;
  padding: 12px 20px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  white-space: nowrap;
  box-shadow: 0 4px 12px rgba(100, 200, 255, 0.3);
  font-size: 14px;
}

.analysis-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(100, 200, 255, 0.4);
  background: linear-gradient(135deg, #4db8ff 0%, #2972e1 100%);
}

.analysis-btn:active {
  transform: translateY(0px);
  box-shadow: 0 2px 8px rgba(100, 200, 255, 0.3);
}

.legend-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; }
.legend-item { display: flex; gap: 10px; align-items: center; }
.legend-color { width: 16px; height: 16px; border-radius: 3px; flex-shrink: 0; }
.legend-color.excellent { background: linear-gradient(135deg, #10b981 0%, #059669 100%); box-shadow: 0 0 8px rgba(16, 185, 129, 0.3); }
.legend-color.good { background: linear-gradient(135deg, #3b82f6 0%, #1d4ed8 100%); box-shadow: 0 0 8px rgba(59, 130, 246, 0.3); }
.legend-color.acceptable { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); box-shadow: 0 0 8px rgba(245, 158, 11, 0.3); }
.legend-color.poor { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); box-shadow: 0 0 8px rgba(239, 68, 68, 0.3); }
.legend-text { flex: 1; }
.legend-text span { color: #a0aec0; font-size: 0.85em; display: block; }

@media (max-width: 768px) {
  .color-legend-container {
    flex-direction: column;
  }

  .analysis-btn {
    width: 100%;
  }
}

.loading { text-align: center; padding: 40px; color: #a0aec0; }
</style>
