<template>
  <div class="metrics-grid">
    <MetricTooltip
      v-for="(metric, index) in displayedMetrics"
      :key="index"
      :title="metric.label"
    >
      <div class="metric-card">
        <h4>{{ metric.label }}</h4>
        <div
          :class="['metric-value', getColorClass(metric.key, metric.value)]"
        >
          {{ metric.formattedValue }}
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
  definition: string
  usage: string
  scoring: string
  realUseCases: string
}

const props = defineProps<{
  globalMetrics: GlobalMetrics
  estimatedPrice: number
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
      const atrPercent = value < 1 ? value * 100 : (value / props.estimatedPrice) * 100
      if (atrPercent > 2.5) return 'excellent'
      if (atrPercent > 1.5) return 'good'
      if (atrPercent > 1.0) return 'acceptable'
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
    default:
      return 'neutral'
  }
}

function formatATR(atr: number): string {
  const atrPercent = (atr / props.estimatedPrice) * 100
  return `${atrPercent.toFixed(2)}%`
}

function getColorClass(metric: string, value: number): string {
  return `metric-${getMetricQuality(metric, value)}`
}

const displayedMetrics = computed(() => [
  {
    key: 'bougies',
    label: 'Bougies',
    value: props.globalMetrics.total_candles,
    formattedValue: props.globalMetrics.total_candles.toLocaleString(),
    definition: 'Nombre total de bougies analysées. Plus il y a de données, plus l\'analyse statistique est fiable et robuste pour identifier les patterns récurrents.',
    usage: '>500 bougies = données abondantes, idéal pour straddle\n200-500 = bon volume\n100-200 = acceptable\n<100 = insuffisant pour fiabilité.',
    scoring: '🟢 Excellent (>500) = Confiance maximale\n🔵 Bon (200-500) = Fiable\n🟡 Acceptable (100-200) = Modéré\n🔴 Pauvre (<100) = Données trop limitées',
    realUseCases: 'Vous analysez le DAX avec 250 bougies (15 min × 250 = 62h d\'historique)\n→ Bon volume pour straddle\n→ Les patterns identifiés sont fiables\n\nMais si vous n\'avez que 80 bougies\n→ Manque de données\n→ Risque de résultats biaisés\n→ Recommandation: attendre plus de données avant de trader'
  },
  {
    key: 'atr',
    label: 'ATR moyen',
    value: props.globalMetrics.mean_atr,
    formattedValue: formatATR(props.globalMetrics.mean_atr),
    definition: 'Average True Range (14 périodes) : mesure la volatilité vraie en points. Détermine directement la largeur du stop-loss et take-profit pour le straddle (2-3× ATR).',
    usage: '>2.5% du prix = volatilité excellente, spreads serrés\n1.5-2.5% = bon (straddle profitable)\n1-1.5% = acceptable\n<1% = faible (gaps risqués).',
    scoring: '🟢 Excellent (>2.5%) = ATR très élevé, gains potentiels importants\n🔵 Bon (1.5-2.5%) = conditions optimales straddle\n🟡 Acceptable (1-1.5%) = possible mais serré\n🔴 Pauvre (<1%) = straddle peu rentable',
    realUseCases: 'EUR/USD à 15h (NY open), ATR = 2.2%\n→ SL = 2.2 × 1.5 = 3.3%, TP = 2.2 × 2.5 = 5.5%\n→ Conditions optimales pour entrer\n\nMême instrument à 12h, ATR = 0.8%\n→ SL = 1.2%, TP = 2.0%\n→ Spreads très serrés, risque/récompense faible\n→ Recommandation: passer, attendre conditions plus volatiles'
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
    key: 'range',
    label: 'Range',
    value: props.globalMetrics.mean_range,
    formattedValue: `${(props.globalMetrics.mean_range / props.estimatedPrice * 100).toFixed(2)}%`,
    definition: 'True Range (H-L avec gaps) : capture le mouvement RÉEL exploitable (contrairement au simple range). Évalue l\'amplitude vraie que le straddle peut capturer.',
    usage: '>2.5% = mouvement énorme exploitable\n1.5-2.5% = bon range, straddle bien positionné\n1-1.5% = acceptable mais serré\n<1% = peu de mouvement.',
    scoring: '🟢 Excellent (>2.5%) = Énorme amplitude, profit assuré\n🔵 Bon (1.5-2.5%) = Range parfait straddle\n🟡 Acceptable (1-1.5%) = Limité mais jouable\n🔴 Pauvre (<1%) = Mouvement insuffisant',
    realUseCases: 'DAX à 8h (London open), range = 2.1%\n→ Si vous entrez au milieu du range\n→ TP à +1% = réaliste et atteignable\n→ Position: entrer avec confiance\n\nS&P 500 en consolidation, range = 0.6%\n→ Très peu d\'espace pour profit\n→ SL et TP trop proches = FX coûts élevés\n→ Recommandation: SKIP, trop de friction'
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
    key: 'noiseratio',
    label: 'Noise Ratio',
    value: props.globalMetrics.mean_noise_ratio,
    formattedValue: `${props.globalMetrics.mean_noise_ratio.toFixed(2)}`,
    definition: 'Ratio Wicks/Body : mesure le ratio bruit/signal. Bas = direction confirmée, spread étroit. Haut = beaucoup de rejets (fausses mèches) = problème majeur pour straddle.',
    usage: '<2.0 = signal excellent, spreads serrés\n2.0-3.0 = acceptable, quelques rejets\n3.0-4.0 = très bruyant, spreads larges\n>4.0 = chaotique, rejets constants.',
    scoring: '🟢 Excellent (<2.0) = Direction nette, pas de spreads larges\n🔵 Bon (2.0-3.0) = Acceptable\n🟡 Acceptable (3.0-4.0) = Rejets importants, TP/SL plus large\n🔴 Pauvre (>4.0) = Chaos, à éviter absolument',
    realUseCases: 'Gold à NFP, noise ratio = 1.8\n→ Peu de rejets, direction confirmée\n→ SL standard (1.5× ATR)\n→ Breakout fiable\n→ Recommandation: TRADE en confiance\n\nMême paire en CPI, noise ratio = 3.2\n→ Beaucoup de fausses mèches\n→ Augmenter SL de 30% (2× ATR au lieu de 1.5×)\n→ Réduire position size de 20%'
  },
  {
    key: 'volumeimbalance',
    label: 'Direction Strength',
    value: props.globalMetrics.mean_volume_imbalance,
    formattedValue: `${(props.globalMetrics.mean_volume_imbalance * 100).toFixed(1)}%`,
    definition: 'Force du mouvement directionnel = (Body Range % × Breakout %). Mesure la COMBINAISON de pureté du signal ET des cassures. Critique pour straddle: besoin de direction forte.',
    usage: '>20% = direction TRÈS forte confirmée\n10-20% = bon directif\n5-10% = moyen, pas assez fort\n<5% = trop faible, movement indécis.',
    scoring: '🟢 Excellent (>20%) = Force directionnelle maximale\n🔵 Bon (10-20%) = Momentum clair\n🟡 Acceptable (5-10%) = Modéré, risqué\n🔴 Pauvre (<5%) = Pas assez de conviction',
    realUseCases: 'DAX à 8h, direction strength = 22%\n→ Force maximale = momentum confirmé\n→ Probabilité de poursuite: 65%+\n→ Position full size\n→ TP agressif (+2.5× ATR)\n\nMême créneau autre jour, direction strength = 4%\n→ Force minimale = indécision\n→ Probabilité de reversal: 50%+\n→ Recommandation: réduire 50% ou SKIP'
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
  }
] as MetricConfig[])
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

.tooltip-section { margin-bottom: 15px; }
.tooltip-section:last-child { margin-bottom: 0; }
.tooltip-section-title { font-weight: bold; color: #60a5fa; margin-bottom: 8px; font-size: 0.95em; }
.tooltip-section-text { color: #cbd5e0; font-size: 0.9em; line-height: 1.5; }
.tooltip-multiline { white-space: pre-wrap; word-wrap: break-word; }
</style>
