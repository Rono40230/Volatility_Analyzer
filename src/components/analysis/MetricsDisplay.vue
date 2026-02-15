<template>
  <div class="metrics-grid">
    <MetricTooltip
      v-for="(metric, index) in displayedMetrics"
      :key="index"
      :title="metric.label"
    >
      <div :class="['metric-card', isSpecialMetric(metric.key) && `metric-special-${getColorClass(metric.key, metric.value)}`]">
        <h4>{{ metric.label }}</h4>
        <div
          :class="['metric-value', getColorClass(metric.key, metric.value)]"
        >
          <span v-if="metric.unit">{{ formatATR(metric.value) }}</span>
          <span v-else>{{ metric.formattedValue }}</span>
        </div>
      </div>
      <template #definition>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📖 Définition de la Métrique</div>
          <div class="tooltip-section-text">{{ metric.definition }}</div>
        </div>
      </template>
      <template #usage>
        <div class="tooltip-section">
          <div class="tooltip-section-title">📊 Interprétation du Score</div>
          <div class="tooltip-section-text tooltip-multiline">{{ metric.usage }}</div>
        </div>
      </template>
      <template #scoring>
        <div class="tooltip-section">
          <div class="tooltip-section-title">🎨 Échelle de Couleurs</div>
          <div class="tooltip-section-text tooltip-multiline">{{ metric.scoring }}</div>
        </div>
      </template>
      <template #realUseCases>
        <div class="tooltip-section">
          <div class="tooltip-section-title">🎯 Cas d'Usage Réel</div>
          <div class="tooltip-section-text tooltip-multiline">{{ metric.realUseCases }}</div>
        </div>
      </template>
    </MetricTooltip>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import MetricTooltip from '../MetricTooltip.vue'
import { pipsToDisplayValue } from '../../utils/assetUnit'

interface GlobalMetrics {
  mean_atr: number
  mean_volatility: number
  mean_body_range: number
  mean_noise_ratio: number
  mean_volume_imbalance: number
  mean_breakout_percentage: number
  mean_range: number
  total_candles: number
}

interface MetricConfig {
  key: string
  label: string
  value: number
  formattedValue: string
  unit?: string
  definition: string
  usage: string
  scoring: string
  realUseCases: string
}

const props = defineProps<{
  globalMetrics: GlobalMetrics
  pointValue?: number
  unit?: string
  symbol?: string
  recommendation?: string
  riskLevel?: string
  confidenceScore?: number
}>()

function getMetricQuality(metric: string, value: number): string {
  switch (metric) {
    case 'bougies':
      if (value > 500) return 'excellent'
      if (value > 200) return 'good'
      if (value > 100) return 'acceptable'
      return 'poor'
    case 'atr':
    case 'range':
      // ATR is already normalized in points/pips
      // We can use raw value for quality check if we assume standard ranges
      // Or we can use percentage of price if we had price
      // For now, let's keep simple thresholds based on points
      if (value > 50) return 'excellent'
      if (value > 20) return 'good'
      if (value > 10) return 'acceptable'
      return 'poor'
    case 'volatility':
      if (value >= 0.30) return 'excellent'
      if (value >= 0.15) return 'good'
      if (value >= 0.05) return 'acceptable'
      return 'poor'
    case 'bodyrange':
      if (value > 45) return 'excellent'
      if (value > 35) return 'good'
      if (value > 15) return 'acceptable'
      return 'poor'
    case 'noiseratio':
      if (value < 2.0) return 'excellent'
      if (value < 3.0) return 'good'
      if (value < 4.0) return 'acceptable'
      return 'poor'
    case 'directionstrength':
      if (value >= 0.20) return 'excellent'
      if (value >= 0.10) return 'good'
      if (value >= 0.05) return 'acceptable'
      return 'poor'
    case 'volumeimbalance':
      if (value > 0.5) return 'excellent'
      if (value > 0.3) return 'good'
      if (value > 0.1) return 'acceptable'
      return 'poor'
    case 'breakout':
      if (value >= 20) return 'excellent'
      if (value >= 10) return 'good'
      if (value >= 5) return 'acceptable'
      return 'poor'
    case 'setupquality':
      if (value >= 80) return 'excellent'
      if (value >= 65) return 'good'
      if (value >= 50) return 'acceptable'
      return 'poor'
    case 'movementquality':
      if (value >= 80) return 'excellent'
      if (value >= 55) return 'good'
      if (value >= 30) return 'acceptable'
      return 'poor'
    case 'confidence':
      if (value >= 80) return 'excellent'
      if (value >= 65) return 'good'
      if (value >= 50) return 'acceptable'
      if (value >= 35) return 'poor'
      return 'poor'
    default:
      return 'neutral'
  }
}

function formatATR(atr: number): string {
  const converted = props.symbol ? pipsToDisplayValue(atr, props.symbol) : atr
  const unit = props.unit || 'pts'
  const prefix = unit === '$' ? '$' : ''
  const suffix = unit === '$' ? '' : ` ${unit}`
  return `${prefix}${converted.toFixed(1)}${suffix}`
}

function getColorClass(metric: string, value: number): string {
  return `metric-${getMetricQuality(metric, value)}`
}

function isSpecialMetric(key: string): boolean {
  return ['setupquality', 'movementquality', 'confidence'].includes(key)
}

const displayedMetrics = computed(() => [
  {
    key: 'atr',
    label: 'ATR moyen',
    value: props.globalMetrics.mean_atr,
    formattedValue: formatATR(props.globalMetrics.mean_atr),
    unit: props.unit || 'pts',
    definition: 'Average True Range (14 périodes) : mesure la volatilité vraie en points. Détermine directement la largeur du stop-loss et take-profit pour le straddle (2-3× ATR).',
    usage: '>100 points = volatilité excellente, spreads serrés\n50-100 points = bon (straddle profitable)\n20-50 points = acceptable\n<20 points = faible (gaps risqués).',
    scoring: '🟢 Excellent (>100 points) = ATR très élevé, gains potentiels importants\n🔵 Bon (50-100 points) = conditions optimales straddle\n🟡 Acceptable (20-50 points) = possible mais serré\n🔴 Pauvre (<20 points) = straddle peu rentable',
    realUseCases: 'EUR/USD à 15h (NY open), ATR = 130 points\n→ SL = 130 × 1.5 = 195 points, TP = 130 × 2.5 = 325 points\n→ Conditions optimales pour entrer\n\nMême instrument à 12h, ATR = 40 points\n→ SL = 60 points, TP = 100 points\n→ Spreads très serrés, risque/récompense faible\n→ Recommandation: passer, attendre conditions plus volatiles'
  },
  {
    key: 'volatility',
    label: 'Volatilité %',
    value: props.globalMetrics.mean_volatility,
    formattedValue: `${(props.globalMetrics.mean_volatility * 100).toFixed(1)}%`,
    definition: 'Ratio ATR/Close en pourcentage : mesure la volatilité relative. Pour le straddle, indique le potentiel de mouvement par rapport au prix (plus élevé = plus de profit possible).',
    usage: '>30% = pics de volatilité rares mais très profitables\n15-30% = volatilité normale, conditions stables pour straddle\n5-15% = faible, mouvements limités\n<5% = stagnation, à éviter.',
    scoring: '🟢 Excellent (>30%) = Pic exceptionnel, gains énormes possibles\n🔵 Bon (15-30%) = Conditions optimales\n🟡 Acceptable (5-15%) = Rendement limité\n🔴 Pauvre (<5%) = Trop calme, risque/récompense mauvais',
    realUseCases: 'GBP/USD après communiqué de la BoE, volatilité = 28%\n→ Conditions très favorables\n→ Position size: normal\n→ Attendre breakout confirmé\n\nMême paire en milieu d\'après-midi, volatilité = 3%\n→ Marché endormi (range-bound)\n→ Rejets fréquents, fausses cassures\n→ Recommandation: SKIP, attendre le prochain événement'
  },
  {
    key: 'bodyrange',
    label: 'Body Range %',
    value: props.globalMetrics.mean_body_range,
    formattedValue: `${props.globalMetrics.mean_body_range.toFixed(1)}%`,
    definition: 'Pourcentage du range formant le body (fermeture réelle) : mesure la PURETÉ du signal. High body % = mouvement directionnel clair et non bruyant. Essentiel pour straddle: besoin d\'une direction nette.',
    usage: '>45% = signal TRÈS pur, direction confirmée = excellent straddle\n25-45% = acceptable, mouvement net\n15-25% = bruyant avec mèches\n<15% = très bruyant, beaucoup d\'indécision.',
    scoring: '🟢 Excellent (>45%) = Signal directif parfait\n🔵 Bon (25-45%) = Direction claire\n🟡 Acceptable (15-25%) = Bruyant mais jouable\n🔴 Pauvre (<15%) = Indécision totale, fausses mèches',
    realUseCases: 'EUR/USD suite NFP, body range = 52%\n→ Direction très claire (peu de queue)\n→ Fermeture proche du high/low\n→ Signal de conviction forte\n→ Recommandation: prendre straddle directionnel\n\nMême jour, différente paire, body range = 18%\n→ Mèches énormes = rejet du mouvement\n→ Indécision du marché\n→ Augmenter SL de 20-30%'
  },
  {
    key: 'volumeimbalance',
    label: 'Direction Strength',
    value: props.globalMetrics.mean_volume_imbalance,
    formattedValue: `${(props.globalMetrics.mean_volume_imbalance * 100).toFixed(1)}%`,
    definition: 'Force du mouvement directionnel [Ratio 0-1] = (Body Range / 100) × (Breakout / 100). Stocké comme ratio, affiché en %. Mesure la COMBINAISON de pureté du signal ET des cassures. Critique pour straddle: besoin de direction forte.',
    usage: '>20% = direction TRÈS forte confirmée\n10-20% = bon directif\n5-10% = moyen, pas assez fort\n<5% = trop faible, movement indécis.',
    scoring: '🟢 Excellent (>20%) = Force directionnelle maximale\n🔵 Bon (10-20%) = Momentum clair\n🟡 Acceptable (5-10%) = Modéré, risqué\n🔴 Pauvre (<5%) = Pas assez de conviction',
    realUseCases: 'DAX à 8h, direction strength = 22%\n→ Force maximale = momentum confirmé (0.22 ratio)\n→ Probabilité de poursuite: 65%+\n→ Position full size\n→ TP agressif (+2.5× ATR)\n\nMême créneau autre jour, direction strength = 4%\n→ Force minimale = indécision (0.04 ratio)\n→ Probabilité de reversal: 50%+\n→ Recommandation: réduire 50% ou SKIP'
  },
  {
    key: 'noiseratio',
    label: 'Noise Ratio',
    value: props.globalMetrics.mean_noise_ratio,
    formattedValue: `${props.globalMetrics.mean_noise_ratio.toFixed(2)}x`,
    definition: 'Ratio Wicks/Body : mesure le ratio bruit/signal. Bas = direction confirmée, spread étroit. Haut = beaucoup de rejets (fausses mèches) = problème majeur pour straddle.',
    usage: '<2.0x = signal excellent, spreads serrés\n2.0-3.0x = acceptable, quelques rejets\n3.0-4.0x = très bruyant, spreads larges\n>4.0x = chaotique, rejets constants.',
    scoring: '🟢 Excellent (<2.0x) = Direction nette, pas de spreads larges\n🔵 Bon (2.0-3.0x) = Acceptable\n🟡 Acceptable (3.0-4.0x) = Rejets importants, TP/SL plus large\n🔴 Pauvre (>4.0x) = Chaos, à éviter absolument',
    realUseCases: 'Gold à NFP, noise ratio = 1.8x\n→ Peu de rejets, direction confirmée\n→ SL standard (1.5× ATR)\n→ Breakout fiable\n→ Recommandation: TRADE en confiance\n\nMême paire en CPI, noise ratio = 3.2x\n→ Beaucoup de fausses mèches\n→ Augmenter SL de 30% (2× ATR au lieu de 1.5×)\n→ Réduire position size de 20%'
  },
  {
    key: 'breakout',
    label: 'Breakout %',
    value: props.globalMetrics.mean_breakout_percentage,
    formattedValue: `${props.globalMetrics.mean_breakout_percentage.toFixed(1)}%`,
    definition: 'Pourcentage de cassures de niveaux clés (True Range distribuée). Mesure la fréquence des mouvements impulsifs. Haut = marché actif, parfait pour straddle.',
    usage: '>15% = breakouts fréquents, marché actif = excellent\n10-15% = bon, quelques impulsions\n5-10% = moyen, range-bound\n<5% = consolidation, peu de mouvement.',
    scoring: '🟢 Excellent (>15%) = Marché très impulsif, gains fréquents\n🔵 Bon (10-15%) = Activité normale\n🟡 Acceptable (5-10%) = Peu de dynamique\n🔴 Pauvre (<5%) = Marché range-bound, stagnant',
    realUseCases: 'Bitcoin après news positive, breakout % = 18%\n→ 18 cassures par 100 bougies = très actif\n→ Chaque signal a 70% chance de suivre\n→ Taille position: normal\n→ Récompense: gains rapides\n\nBitcoin en sideways, breakout % = 3%\n→ 3 cassures par 100 bougies = très peu\n→ 95% du temps = fausses cassures\n→ Recommandation: SKIP, attendre volatilité'
  },
  {
    key: 'setupquality',
    label: 'Setup Quality',
    value: getSetupQualityScore(props.recommendation),
    formattedValue: `${getSetupQualityScore(props.recommendation)}/100`,
    definition: 'Score de qualité du setup Straddle (0-100) : évalue la qualité globale des conditions économiques et techniques pour exécuter un straddle basé sur les patterns historiques.',
    usage: '80-100 = SETUP OPTIMAL, conditions idéales\n65-80 = SETUP CORRECT, bon setup\n50-65 = SETUP ACCEPTABLE, moyen\n35-50 = SETUP RISQUÉ, conditions médiocres\n<35 = NE PAS TRADER, conditions inadaptées',
    scoring: '🟢 Excellent (80-100) = Conditions optimales pour straddle\n🔵 Bon (65-80) = Conditions favorables\n🟡 Acceptable (50-65) = Conditions moyennes\n🔴 Risqué (35-50) = Envisager de passer\n🔴 Pauvre (<35) = Ne pas trader',
    realUseCases: 'EUR/USD à 15h (NY Open), setup quality = 95\n→ Patterns historiques favorables\n→ Volatilité attendue élevée\n→ Recommandation: TRADE en confiance, position full size\n\nMême paire le jour suivant, setup quality = 32\n→ Patterns défavorables\n→ Volatilité imprévisible\n→ Recommandation: SKIP, attendre conditions meilleures'
  },
  {
    key: 'movementquality',
    label: 'Movement Quality',
    value: getMovementQualityScore(props.riskLevel),
    formattedValue: getMovementQualityLabel(props.riskLevel),
    definition: 'Qualité du mouvement attendu (Directional/Moderate/Erratic) : caractérise le type de volatilité basée sur l\'analyse des patterns.',
    usage: 'DIRECTIONNEL (Low) = volatilité 15-30% avec faible bruit, idéal straddle\nMODÉRÉ (Medium) = volatilité 5-15% avec bruit acceptable\nERRATIQUE (High) = soit <5% soit >30%, à éviter',
    scoring: '🟢 Excellent (Directionnel) = Volatilité nette, spreads serrés\n🔵 Bon (Modéré) = Conditions acceptables\n🔴 Pauvre (Erratique) = Trop calme ou chaotique',
    realUseCases: 'NFP sur EUR/USD, movement quality = DIRECTIONNEL\n→ Volatilité 22%, noise ratio 1.9x\n→ Direction nette, peu de rejets\n→ Recommandation: TRADE normal\n\nMême événement, autre paire, movement quality = ERRATIQUE\n→ Volatilité 3%, chaos des spreads\n→ Trop calme\n→ Recommandation: SKIP, attendre meilleur setup'
  },
  {
    key: 'confidence',
    label: 'Confidence Score',
    value: props.confidenceScore || 0,
    formattedValue: `${Math.round(props.confidenceScore || 0)}/100`,
    definition: 'Score de confiance global (0-100) : mesure "à quel point on peut confier sa stratégie Straddle scalping à cette paire durant cette période". Somme pondérée de 6 facteurs: ATR (30pts), Body Range (25pts), Volatilité (25pts), Noise Ratio (10pts), Breakout (10pts), Bonus Données (5pts).',
    usage: '80-100 = Excellent, conditions optimales pour scalper agressivement\n65-80 = Bon, scalper normalement\n50-65 = Prudent, scalper avec SL serrés\n35-50 = Risqué, très prudent ou breakouts only\n0-35 = Mauvais, ne pas trader',
    scoring: '🟢 Excellent (80-100) = Volatilité constante, signal pur, données fiables\n🔵 Bon (65-80) = Conditions stables, peu de risques\n🟡 Acceptable (50-65) = Conditions moyennes, rendement limité\n🔴 Pauvre (35-50) = Beaucoup de rejets ou volatilité basse\n🔴 Mauvais (<35) = À éviter complètement',
    realUseCases: 'EURUSD 10h-11h UTC, confiance = 95\n→ ATR 2.5 pips (30pts) + BodyRange 52% (25pts) + Vol 25% (25pts) + NR 1.8 (10pts) + Breakout 18% (10pts) + Bonus (5pts)\n→ = 105 → cappé à 100\n→ Recommandation: TRADE agressif, position full size\n\nMême paire 13h, confiance = 32\n→ Peu de volatilité (8pts), signal bruyant (4pts), peu de cassures (2pts)\n→ Recommandation: SKIP, attendre meilleur setup'
  }
] as MetricConfig[])

function getSetupQualityScore(recommendation?: string): number {
  switch (recommendation) {
    case 'StraddleOptimal':
      return 90
    case 'StraddleGood':
      return 72
    case 'StraddleCautious':
      return 57
    case 'StraddleRisky':
      return 42
    case 'NoTrade':
      return 20
    default:
      return 50
  }
}

function getMovementQualityScore(riskLevel?: string): number {
  switch (riskLevel) {
    case 'Low':
      return 85
    case 'Medium':
      return 60
    case 'High':
      return 25
    default:
      return 50
  }
}

function getMovementQualityLabel(riskLevel?: string): string {
  switch (riskLevel) {
    case 'Low':
      return '🟢 DIRECTIONNEL'
    case 'Medium':
      return '🔵 MODÉRÉ'
    case 'High':
      return '🔴 ERRATIQUE'
    default:
      return '⚪ NEUTRE'
  }
}
</script>

<style scoped>
.metrics-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 30px; }
.metric-card { background: #1a202c; padding: 15px; border-radius: 8px; border-left: 3px solid #667eea; cursor: help; }
.metric-card h4 { margin: 0 0 10px 0; color: #e2e8f0; }
.metric-value { font-size: 1.5em; font-weight: bold; transition: color 0.3s ease; }
.metric-value.metric-excellent { color: #10b981; text-shadow: 0 0 8px rgba(16, 185, 129, 0.3); }
.metric-value.metric-good { color: #3b82f6; text-shadow: 0 0 8px rgba(59, 130, 246, 0.3); }
.metric-value.metric-acceptable { color: #f59e0b; text-shadow: 0 0 8px rgba(245, 158, 11, 0.3); }
.metric-value.metric-poor { color: #ef4444; text-shadow: 0 0 8px rgba(239, 68, 68, 0.3); }
.metric-value.metric-neutral { color: #667eea; text-shadow: 0 0 8px rgba(102, 126, 234, 0.3); }
.metric-card:has(.metric-excellent) { border-left-color: #10b981; }
.metric-card:has(.metric-good) { border-left-color: #3b82f6; }
.metric-card:has(.metric-acceptable) { border-left-color: #f59e0b; }
.metric-card:has(.metric-poor) { border-left-color: #ef4444; }

/* Special metrics with pastel backgrounds */
.metric-special-metric-excellent { background: rgba(16, 185, 129, 0.1) !important; border-left-color: #10b981 !important; }
.metric-special-metric-good { background: rgba(59, 130, 246, 0.1) !important; border-left-color: #3b82f6 !important; }
.metric-special-metric-acceptable { background: rgba(245, 158, 11, 0.1) !important; border-left-color: #f59e0b !important; }
.metric-special-metric-poor { background: rgba(239, 68, 68, 0.1) !important; border-left-color: #ef4444 !important; }

.tooltip-section { margin-bottom: 15px; }
.tooltip-section:last-child { margin-bottom: 0; }
.tooltip-section-title { font-weight: bold; color: #60a5fa; margin-bottom: 8px; font-size: 0.95em; }
.tooltip-section-text { color: #cbd5e0; font-size: 0.9em; line-height: 1.5; }
.tooltip-multiline { white-space: pre-wrap; word-wrap: break-word; }
</style>
