import { computed } from 'vue'
import type { BacktestResult, BacktestConfig } from '../stores/backtest'

export function useBacktestAnalysis(result: BacktestResult, config: BacktestConfig) {
  // --- Helpers ---
  const totalEvents = computed(() => result.total_trades + result.no_entries)
  const noEntryCount = computed(() => result.no_entries)
  const noEntryPercent = computed(() => totalEvents.value > 0 ? Math.round((noEntryCount.value / totalEvents.value) * 100) : 0)
  const maxDrawdown = computed(() => result.max_drawdown_pips.toFixed(1))

  // Consecutive Losses Calculation
  const consecutiveLosses = computed(() => {
    let maxLosses = 0
    let currentLosses = 0
    result.trades.forEach(t => {
      if (t.pips_net < 0) {
        currentLosses++
      } else {
        maxLosses = Math.max(maxLosses, currentLosses)
        currentLosses = 0
      }
    })
    return Math.max(maxLosses, currentLosses)
  })

  // MFE / MAE Calculation
  const avgMfe = computed(() => {
    const trades = result.trades
    if (trades.length === 0) return '0.0'
    const sum = trades.reduce((acc, t) => acc + t.max_favorable_excursion, 0)
    return (sum / trades.length).toFixed(1)
  })

  const avgMae = computed(() => {
    const trades = result.trades
    if (trades.length === 0) return '0.0'
    const sum = trades.reduce((acc, t) => acc + t.max_adverse_excursion, 0)
    return (sum / trades.length).toFixed(1)
  })

  // --- 1. Verdict Logic ---
  const pf = computed(() => result.profit_factor)
  const verdictClass = computed(() => {
    if (pf.value < 1.0) return 'verdict-bad'
    if (pf.value < 1.5) return 'verdict-neutral'
    return 'verdict-good'
  })
  const verdictIcon = computed(() => {
    if (pf.value < 1.0) return '📉'
    if (pf.value < 1.5) return '😐'
    return '🚀'
  })
  const verdictTitle = computed(() => {
    if (pf.value < 1.0) return 'Non Rentable'
    if (pf.value < 1.5) return 'Marginalement Rentable'
    return 'Stratégie Solide'
  })
  const verdictText = computed(() => {
    if (pf.value < 1.0) return 'La stratégie perd de l\'argent. Vos pertes moyennes dépassent vos gains.'
    if (pf.value < 1.5) return 'Attention aux frais (spread/slippage) qui pourraient effacer ces gains en réel.'
    return 'Excellente espérance de gain. Le ratio risque/récompense est favorable.'
  })

  // --- 2. Activity Logic ---
  const noEntryClass = computed(() => noEntryPercent.value > 50 ? 'text-warning' : '')
  const activityAdvice = computed(() => {
    if (noEntryPercent.value > 60) return `Votre Offset (${config.offset_pips} pips) est trop conservateur. Le prix n'atteint pas vos ordres.`
    if (noEntryPercent.value < 10) return 'Votre Offset est très court, vous captez presque tous les mouvements (attention au bruit).'
    return 'Le taux de déclenchement est équilibré.'
  })

  // --- 3. Risk Logic ---
  const riskAdvice = computed(() => {
    const drawdown = Math.abs(result.max_drawdown_pips)
    const totalGain = result.total_pips
    if (totalGain > 0 && drawdown > totalGain * 0.5) return 'Ratio Calmar faible : vous risquez beaucoup pour gagner peu.'
    if (consecutiveLosses.value > 4) return 'La série de pertes est longue. Vérifiez si ces pertes arrivent sur des types d\'événements spécifiques.'
    return 'Gestion du risque acceptable.'
  })

  // --- 4. Exit Logic ---
  const exitAdvice = computed(() => {
    const mfe = parseFloat(avgMfe.value)
    const avgPips = result.average_pips_per_trade
    if (mfe > avgPips * 2) return 'Vos trades vont souvent beaucoup plus loin que vos gains réels. Augmentez le Trailing Stop ou le TP.'
    const mae = parseFloat(avgMae.value)
    const sl = config.stop_loss_pips
    if (mae < sl * 0.5 && result.losing_trades > 0) return 'Vos pertes touchent le SL rapidement. Le sens du trade est souvent invalidé dès le départ.'
    return 'Les sorties semblent cohérentes avec la volatilité.'
  })

  // --- 5. Final Recommendation ---
  const finalRecommendation = computed(() => {
    if (pf.value < 1.0) return 'Réduisez le Stop Loss ou filtrez mieux les événements (utilisez le Seuil de Surprise).'
    if (noEntryPercent.value > 50) return 'Réduisez l\'Offset pour entrer plus souvent en position.'
    const mfe = parseFloat(avgMfe.value)
    if (mfe > 50 && config.trailing_stop_pips < 20) return 'Augmentez le Trailing Stop pour laisser courir les gains sur les gros mouvements.'
    return 'La configuration actuelle est équilibrée. Vous pouvez affiner le Slippage pour plus de réalisme.'
  })

  return {
    totalEvents,
    noEntryCount,
    noEntryPercent,
    maxDrawdown,
    consecutiveLosses,
    avgMfe,
    avgMae,
    verdictClass,
    verdictIcon,
    verdictTitle,
    verdictText,
    noEntryClass,
    activityAdvice,
    riskAdvice,
    exitAdvice,
    finalRecommendation
  }
}
