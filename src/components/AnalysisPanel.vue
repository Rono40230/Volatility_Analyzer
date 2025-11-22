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
        <MetricTooltip title="Qualité du Setup Straddle">
          <span 
            :class="['badge', 'recommendation', recommendationClass]"
          >
            {{ formatRecommendation(props.result.recommendation) }}
          </span>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">
              Évalue la qualité des conditions de marché pour exécuter un <strong>Straddle</strong> (placement d'ordres Buy Stop et Sell Stop de part et d'autre du prix avant une annonce économique).
            </div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Codes Couleurs & Signification</div>
            <div class="tooltip-section-text">
              <strong style="color: #10b981;">✅ SETUP OPTIMAL (Vert)</strong><br/>
              Score 80-100 : Conditions idéales. Offset standard <strong>10-15 pips</strong>. Forte probabilité de breakout directionnel franc.<br/><br/>
              
              <strong style="color: #3b82f6;">🟢 SETUP CORRECT (Bleu)</strong><br/>
              Score 65-80 : Bon setup. Élargir légèrement l'offset à <strong>15-20 pips</strong> pour éviter les fausses mèches.<br/><br/>
              
              <strong style="color: #f59e0b;">🔵 SETUP ACCEPTABLE (Orange)</strong><br/>
              Score 50-65 : Setup moyen. Offset large recommandé <strong>20-30 pips</strong>. Surveillance accrue du bruit.<br/><br/>
              
              <strong style="color: #f97316;">🟠 SETUP RISQUÉ (Orange foncé)</strong><br/>
              Score 35-50 : Conditions médiocres. Envisager de passer l'événement ou réduire drastiquement la taille.<br/><br/>
              
              <strong style="color: #ef4444;">❌ NE PAS TRADER (Rouge)</strong><br/>
              Score &lt;35 : Conditions inadaptées (volatilité insuffisante ou trop de bruit).
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Action Recommandée</div>
            <div class="tooltip-section-text">
              Le badge indique la <strong>distance d'offset optimale</strong> à utiliser pour vos ordres Stop.<br/>
              Plus le setup est bon (vert), plus vous pouvez serrer vos ordres près du prix actuel.
            </div>
          </template>
        </MetricTooltip>

        <MetricTooltip title="Qualité du Mouvement">
          <span 
            :class="['badge', 'risk', getRiskClass(props.result.risk_level)]"
          >
            {{ formatRisk(props.result.risk_level) }}
          </span>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">
              Caractérise le <strong>type de mouvement</strong> attendu après l'annonce, basé sur la volatilité historique et le niveau de bruit.
            </div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Codes Couleurs & Signification</div>
            <div class="tooltip-section-text">
              <strong style="color: #22c55e;">🟢 DIRECTIONNEL (Vert)</strong><br/>
              Volatilité 15-30% avec faible bruit. <strong>Idéal pour Straddle</strong> : mouvement franc, prévisible, peu de faux breakouts.<br/><br/>
              
              <strong style="color: #f59e0b;">🔵 MODÉRÉ (Orange)</strong><br/>
              Volatilité 5-15% avec bruit acceptable. Straddle possible mais offset à élargir. Mouvement moins explosif.<br/><br/>
              
              <strong style="color: #ef4444;">🔴 ERRATIQUE (Rouge)</strong><br/>
              Soit trop calme (&lt;5% volatilité, pas de breakout) soit trop chaotique (&gt;30%, faux breakouts multiples). <strong>Risque élevé</strong>.
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Interprétation</div>
            <div class="tooltip-section-text">
              <strong>Vert</strong> : Le mouvement sera probablement unidirectionnel et propre.<br/>
              <strong>Orange</strong> : Mouvement modéré, ajustez vos attentes de gain.<br/>
              <strong>Rouge</strong> : Mouvement imprévisible ou absent, évitez ou réduisez la taille.
            </div>
          </template>
        </MetricTooltip>
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
            <strong>🔵 50-65</strong> PRUDENT - Scalpe PETIT volumes, stop serrés, position sizes réduits<br/>
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
            <div class="tooltip-section-title">📈 Interprétation (Tous actifs)</div>
            <div class="tooltip-section-text">
              🟢 <strong>>2.5%:</strong> Excellent - Forte volatilité, mouvements constants<br>
              🔵 <strong>1.5-2.5%:</strong> Bon - Volatilité fiable pour scalping<br>
              🟠 <strong>1.0-1.5%:</strong> Moyen - Stop serré obligatoire<br>
              🔴 <strong><1.0%:</strong> Mauvais - Attendre, trop peu de volatilité
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
        <div :class="['metric-value', getColorClass('atr', props.result.global_metrics.mean_atr)]">
          {{ formatATR(props.result.global_metrics.mean_atr) }}
        </div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="True Range">
          <h4>📏 True Range</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">True Range = max(Haut-Bas, |Haut-Clôture[t-1]|, |Bas-Clôture[t-1]|). Capture les gaps et fermetures précédentes. Plus précis que simple H-L pour détecter les vrais mouvements.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation</div>
            <div class="tooltip-section-text">
              🟢 <strong>>2.5%:</strong> Excellent - Grande amplitude, idéal scalping<br>
              🔵 <strong>1.5-2.5%:</strong> Bon - Amplitude correcte<br>
              🟠 <strong>1.0-1.5%:</strong> Moyen - Mouvements limités<br>
              🔴 <strong><1.0%:</strong> Mauvais - Bougies minuscules
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">💡 Décision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Si TR >2.5%:</strong> Espace suffisant pour TP rapide<br>
              ⚠️ <strong>Si TR >> ATR:</strong> Risque gap important overnight<br>
              🎯 <strong>Action:</strong> Utiliser pour détecter breakouts après gap, combiné avec ATR
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('range', props.result.global_metrics.mean_range)]">
          {{ formatATR(props.result.global_metrics.mean_range) }}
        </div>
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
              🔵 <strong>15-30%:</strong> Bon - Volatilite normale forex, scalpe standard<br>
              🟠 <strong>5-15%:</strong> Moyen - Scalpe tres serré, patience requise<br>
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
              🔵 <strong>35-45%:</strong> Bon - Assez directif pour scalpe normal<br>
              🟠 <strong>15-35%:</strong> Moyen - Bougies indécises, bruit modéré<br>
              🔴 <strong>≤15%:</strong> Mauvais - Bougies indécises avec longues mèches
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
        <div :class="['metric-value', getColorClass('bodyrange', Math.abs(props.result.global_metrics.mean_body_range))]">
          {{ Math.abs(props.result.global_metrics.mean_body_range).toFixed(2) }}%
          <span style="font-size: 0.8em; opacity: 0.7;">{{ props.result.global_metrics.mean_body_range >= 0 ? '↗' : '↘' }}</span>
        </div>
      </div>
      <div class="metric-card">
        <MetricTooltip title="Tick Quality">
          <h4>✨ Tick Quality</h4>
          <template #definition>
            <div class="tooltip-section-title">Définition</div>
            <div class="tooltip-section-text">Taille moyenne des mouvements de prix unitaires = liquidity quality. Mesure l'existence de market makers et la granularité des données tick.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">📊 Interprétation (Tous actifs)</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;1.0%:</strong> Excellent - Très liquide, spreads serrés<br>
              🔵 <strong>0.5-1.0%:</strong> Bon - Liquide, spreads acceptables<br>
              🟠 <strong>0.1-0.5%:</strong> Moyen - Spreads plus larges<br>
              🔴 <strong>≤0.1%:</strong> Mauvais - Peu liquide, spreads prohibitifs
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
        <div :class="['metric-value', getColorClass('tickquality', props.result.global_metrics.mean_tick_quality)]">
          {{ formatTickQuality(props.result.global_metrics.mean_tick_quality) }}
        </div>
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
              🔵 <strong>2.0-3.0:</strong> Bon - Signal acceptable mais attention aux whipsaws<br>
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
        <MetricTooltip title="Direction Strength">
          <h4>💪 Direction Strength</h4>
          <template #definition>
            <div class="tooltip-section-title">Definition</div>
            <div class="tooltip-section-text">Force directionnelle = (|Body Range %| × Breakout %) / 100. Mesure puissance mouvement directionnel combine aux cassures identifiees.</div>
          </template>
          <template #usage>
            <div class="tooltip-section-title">Interpretation - Seuils de Couleur</div>
            <div class="tooltip-section-text">
              🟢 <strong>&gt;20%:</strong> Excellent - Mouvement fort directionnel ideal STRADDLE<br>
              🔵 <strong>10-20%:</strong> Bon - Direction claire avec cassures<br>
              🟠 <strong>5-10%:</strong> Moyen - Mouvement faible ou indecis<br>
              🔴 <strong>&lt;5%:</strong> Mauvais - Peu de direction indecis
            </div>
          </template>
          <template #scoring>
            <div class="tooltip-section-title">Decision Trading</div>
            <div class="tooltip-section-text">
              ✅ <strong>Si &gt;20%:</strong> Scalpe directionnel optimal COMBO DIRECTIONNEL FORT<br>
              ⚠️ <strong>Si &lt;5%:</strong> Evite trading agressif<br>
              🎯 <strong>Best combo:</strong> Direction Strength &gt;20% + Body Range &gt;45% + Noise &lt;2.0 = JACKPOT
            </div>
          </template>
        </MetricTooltip>
        <div :class="['metric-value', getColorClass('directionstrength', props.result.global_metrics.mean_volume_imbalance)]">
          {{ (props.result.global_metrics.mean_volume_imbalance * 100).toFixed(2) }}%
        </div>
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
              🔵 <strong>10-20%:</strong> Bon - Activité modérée, scalpe avec cassures<br>
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
        <div :class="['metric-value', getColorClass('breakout', props.result.global_metrics.mean_breakout_percentage)]">{{ props.result.global_metrics.mean_breakout_percentage.toFixed(2) }}%</div>
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
  mean_range: number
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
  stats_15min?: any[]
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
    'StraddleOptimal': '✅ SETUP OPTIMAL',
    'StraddleGood': '🟢 SETUP CORRECT',
    'StraddleCautious': '🔵 SETUP ACCEPTABLE',
    'StraddleRisky': '🟠 SETUP RISQUÉ',
    'NoTrade': '❌ NE PAS TRADER'
  }
  return map[rec] || rec
}

function formatRisk(risk: string): string {
  const map: { [key: string]: string } = {
    'Low': '🟢 DIRECTIONNEL',
    'Medium': '🔵 MODÉRÉ',
    'High': '🔴 ERRATIQUE'
  }
  return map[risk] || risk
}

function getRecommendationTooltip(rec: string): string {
  const tooltips: { [key: string]: string } = {
    'StraddleOptimal': 'SETUP OPTIMAL - Conditions idéales pour Straddle. Offset standard (10-15 pips), forte probabilité de breakout directionnel.',
    'StraddleGood': 'SETUP CORRECT - Bon setup Straddle. Élargir légèrement l\'offset (15-20 pips) pour éviter les fausses mèches.',
    'StraddleCautious': 'SETUP ACCEPTABLE - Setup moyen. Offset large recommandé (20-30 pips), surveillance accrue du bruit.',
    'StraddleRisky': 'SETUP RISQUÉ - Conditions médiocres. Envisager de passer cet événement ou réduire drastiquement la taille.',
    'NoTrade': 'NE PAS TRADER - Conditions inadaptées au Straddle (volatilité insuffisante ou trop de bruit).'
  }
  return tooltips[rec] || rec
}

function getRiskClass(risk: string): string {
  const map: { [key: string]: string } = {
    'Low': 'low',
    'Medium': 'medium',
    'High': 'high'
  }
  return map[risk] || ''
}

function getRiskTooltip(risk: string): string {
  const tooltips: { [key: string]: string } = {
    'Low': 'MOUVEMENT DIRECTIONNEL - Volatilité 15-30% avec faible bruit. Idéal pour Straddle : mouvement franc et prévisible.',
    'Medium': 'MOUVEMENT MODÉRÉ - Volatilité 5-15% avec bruit acceptable. Straddle possible mais offset à élargir.',
    'High': 'MOUVEMENT ERRATIQUE - Soit trop calme (<5%) soit trop chaotique (>30%). Risque élevé de faux breakouts ou d\'absence de mouvement.'
  }
  return tooltips[risk] || risk
}

// Fonction pour ouvrir la modal d'analyse
function openAnalysisModal() {
  isAnalysisModalOpen.value = true
}

const recommendationClass = computed(() => {
  const rec = props.result?.recommendation
  if (rec === 'StraddleOptimal') return 'optimal'
  if (rec === 'StraddleGood') return 'good'
  if (rec === 'StraddleCautious') return 'cautious'
  if (rec === 'StraddleRisky') return 'risky'
  if (rec === 'NoTrade') return 'notrade'
  return 'hold' // fallback
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
    case 'range':
      // ATR/Range normalisé en % du prix
      const atrPercent = value < 1 ? value * 100 : (value / getEstimatedPrice()) * 100
      if (atrPercent > 2.5) return 'excellent' // >2.5%
      if (atrPercent > 1.5) return 'good'      // 1.5-2.5%
      if (atrPercent > 1.0) return 'acceptable' // 1.0-1.5%
      return 'poor'
    
    case 'volatility':
      // Volatilité déjà en décimal (0.19 = 19%)
      if (value >= 0.30) return 'excellent' // >30%
      if (value >= 0.15) return 'good'      // 15-30%
      if (value >= 0.05) return 'acceptable' // 5-15%
      return 'poor'
    
    case 'bodyrange':
      // Body Range normalisé en % (backend envoie en % 0-100, pas décimal)
      if (value > 45) return 'excellent'
      if (value > 35) return 'good'
      if (value > 15) return 'acceptable'
      return 'poor'
    
    case 'tickquality':
      // Tick Quality normalisé en % du prix
      const tickPercent = value < 1 ? value * 100 : (value / getEstimatedPrice()) * 100
      if (tickPercent > 1.0) return 'excellent' // >1.0%
      if (tickPercent > 0.5) return 'good'      // 0.5-1.0%
      if (tickPercent > 0.1) return 'acceptable' // 0.1-0.5%
      return 'poor'
    
    case 'noiseratio':
      if (value < 2.0) return 'excellent'
      if (value < 3.0) return 'good'
      if (value < 4.0) return 'acceptable'
      return 'poor'
    
    case 'directionstrength':
      // Direction Strength en % (body_range_mean * breakout_percentage / 100)
      // Thresholds: >20% = excellent, 10-20% = good, 5-10% = acceptable, <5% = poor
      if (value >= 0.20) return 'excellent'
      if (value >= 0.10) return 'good'
      if (value >= 0.05) return 'acceptable'
      return 'poor'
    
    case 'volumeimbalance':
      // Volume Imbalance : moyenne des écarts absolus (0 à +∞)
      // 0.5 = 50% de variation moyenne
      if (value > 0.5) return 'excellent'
      if (value > 0.3) return 'good'
      if (value > 0.1) return 'acceptable'
      return 'poor'
    
    case 'breakout':
      if (value >= 20) return 'excellent' // >20% breakouts
      if (value >= 10) return 'good'
      if (value >= 5) return 'acceptable'
      return 'poor'
    
    default:
      return 'neutral'
  }
}

// Fonction helper pour estimer le prix moyen (pour normalisation)
function getEstimatedPrice(): number {
  if (!props.result?.hourly_stats || props.result.hourly_stats.length === 0) {
    return 100000 // Valeur par défaut pour BTCUSD
  }
  // Utilise l'ATR moyen pour estimer l'ordre de grandeur du prix
  const atr = props.result.global_metrics.mean_atr
  if (atr > 1000) return 100000 // Crypto (BTCUSD ~100k)
  if (atr > 10) return 10000    // Indices (SPX ~10k)
  return 1.0                     // Forex (EURUSD ~1.0)
}

// Formatte l'ATR en % du prix
function formatATR(atr: number): string {
  const price = getEstimatedPrice()
  const atrPercent = (atr / price) * 100
  return `${atrPercent.toFixed(2)}%`
}

// Formatte le Tick Quality en % du prix
function formatTickQuality(tick: number): string {
  const price = getEstimatedPrice()
  const tickPercent = (tick / price) * 100
  return `${tickPercent.toFixed(2)}%`
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

/* Styles pour les recommandations Straddle */
.recommendation.optimal { background: linear-gradient(135deg, #10b981 0%, #059669 100%); border-color: #047857; }
.recommendation.good { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); border-color: #1d4ed8; }
.recommendation.cautious { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); border-color: #b45309; }
.recommendation.risky { background: linear-gradient(135deg, #f97316 0%, #ea580c 100%); border-color: #c2410c; }
.recommendation.notrade { background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%); border-color: #b91c1c; }

/* Styles pour le niveau de risque (qualité du mouvement) */
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

/* Couleurs dynamiques pour les metriques */
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
