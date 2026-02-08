import { computed } from 'vue'
import type { BacktestResult, BacktestConfig } from '../stores/backtest'

export function useBacktestAnalysis(result: BacktestResult, config: BacktestConfig) {
  const executedTrades = computed(() => result.trades.filter(t => t.outcome !== 'NoEntry'))
  const executedCount = computed(() => executedTrades.value.length)

  const toPercent = (value: number, total: number) => {
    if (total <= 0) return '0.0'
    return ((value / total) * 100).toFixed(1)
  }

  const mean = (values: number[]) => {
    if (values.length === 0) return 0
    return values.reduce((acc, v) => acc + v, 0) / values.length
  }

  const median = (values: number[]) => {
    if (values.length === 0) return 0
    const sorted = [...values].sort((a, b) => a - b)
    const mid = Math.floor(sorted.length / 2)
    if (sorted.length % 2 === 0) {
      return (sorted[mid - 1] + sorted[mid]) / 2
    }
    return sorted[mid]
  }
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

  const avgMfeValue = computed(() => parseFloat(avgMfe.value))
  const avgMaeValue = computed(() => parseFloat(avgMae.value))

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
    if (noEntryPercent.value > 60) return 'Beaucoup d’événements sans entrée. Vérifiez les données M1 autour de T0 et la période analysée.'
    if (noEntryPercent.value < 10) return 'Les entrées se déclenchent quasi systématiquement (logique en straddle immédiat).'
    return 'Le taux de déclenchement est stable.'
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
    if (mfe > avgPips * 2) return 'Vos trades vont souvent bien plus loin que vos gains réels. Augmentez le TP (R) ou relâchez le Trailing Stop.'
    const mae = parseFloat(avgMae.value)
    const sl = config.stop_loss_pips
    if (mae < sl * 0.5 && result.losing_trades > 0) return 'Vos pertes touchent le SL rapidement. Le sens du trade est souvent invalidé dès le départ.'
    return 'Les sorties semblent cohérentes avec la volatilité.'
  })

  // --- 5. Final Recommendation ---
  const finalRecommendation = computed(() => {
    if (pf.value < 1.0) return 'Réduisez le Stop Loss ou filtrez mieux les événements (utilisez le Seuil de Surprise).'
    if (noEntryPercent.value > 50) return 'Vérifiez la qualité des données et la fenêtre d’analyse autour de T0.'
    const mfe = parseFloat(avgMfe.value)
    if (mfe > 50 && config.tp_rr < 5) return 'Augmentez le TP (R) pour capter plus de tendance sur les gros mouvements.'
    return 'La configuration actuelle est équilibrée. Vous pouvez affiner le Slippage pour plus de réalisme.'
  })

  const durationValues = computed(() => executedTrades.value.map(t => t.duration_minutes))
  const avgDuration = computed(() => mean(durationValues.value).toFixed(1))
  const medianDuration = computed(() => median(durationValues.value).toFixed(1))

  const winTrades = computed(() => executedTrades.value.filter(t => t.pips_net > 0))
  const lossTrades = computed(() => executedTrades.value.filter(t => t.pips_net < 0))
  const timeoutTrades = computed(() => executedTrades.value.filter(t => t.outcome === 'Timeout'))
  const takeProfitTrades = computed(() => executedTrades.value.filter(t => t.outcome === 'TakeProfit'))

  const quickWinRate = computed(() =>
    toPercent(winTrades.value.filter(t => t.duration_minutes <= 1).length, winTrades.value.length)
  )
  const quickLossRate = computed(() =>
    toPercent(lossTrades.value.filter(t => t.duration_minutes <= 1).length, lossTrades.value.length)
  )
  const tpRate = computed(() => toPercent(takeProfitTrades.value.length, executedCount.value))
  const timeoutRate = computed(() => toPercent(timeoutTrades.value.length, executedCount.value))

  const tpPips = computed(() => config.stop_loss_pips * config.tp_rr)
  const tpPotentialCount = computed(() =>
    executedTrades.value.filter(t => t.max_favorable_excursion >= tpPips.value).length
  )
  const tpMissCount = computed(() =>
    executedTrades.value.filter(t => t.max_favorable_excursion >= tpPips.value && t.outcome !== 'TakeProfit').length
  )
  const tpPotentialRate = computed(() => toPercent(tpPotentialCount.value, executedCount.value))
  const tpMissRate = computed(() => toPercent(tpMissCount.value, executedCount.value))

  const mfeMaeRatio = computed(() => {
    if (avgMaeValue.value <= 0) return '0.0'
    return (avgMfeValue.value / avgMaeValue.value).toFixed(2)
  })

  const beHitCount = computed(() =>
    executedTrades.value.filter(t => t.logs.some(log => log.includes('BE Long') || log.includes('BE Short'))).length
  )
  const beHitRate = computed(() => toPercent(beHitCount.value, executedCount.value))

  const trailingExitCount = computed(() =>
    executedTrades.value.filter(t => t.logs.some(log => log.includes('TS Long') || log.includes('TS Short'))).length
  )
  const trailingExitRate = computed(() => toPercent(trailingExitCount.value, executedCount.value))

  const monthStats = computed(() => {
    const stats = new Map<string, { profit: number; loss: number; net: number; trades: number }>()
    for (const trade of executedTrades.value) {
      const date = new Date(trade.event_date)
      if (Number.isNaN(date.getTime())) continue
      const key = `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}`
      const entry = stats.get(key) || { profit: 0, loss: 0, net: 0, trades: 0 }
      if (trade.pips_net >= 0) {
        entry.profit += trade.pips_net
      } else {
        entry.loss += Math.abs(trade.pips_net)
      }
      entry.net += trade.pips_net
      entry.trades += 1
      stats.set(key, entry)
    }
    return stats
  })

  const monthSummaries = computed(() => {
    const items: { key: string; net: number; pf: number; trades: number }[] = []
    for (const [key, value] of monthStats.value.entries()) {
      const pfVal = value.loss > 0 ? value.profit / value.loss : value.profit > 0 ? 999 : 0
      items.push({ key, net: value.net, pf: pfVal, trades: value.trades })
    }
    return items
  })

  const bestMonth = computed(() => {
    if (monthSummaries.value.length === 0) return 'n/a'
    const sorted = [...monthSummaries.value].sort((a, b) => b.net - a.net)
    const top = sorted[0]
    return `${top.key} (PF ${top.pf.toFixed(2)}, ${top.net.toFixed(1)} pips)`
  })

  const worstMonth = computed(() => {
    if (monthSummaries.value.length === 0) return 'n/a'
    const sorted = [...monthSummaries.value].sort((a, b) => a.net - b.net)
    const bottom = sorted[0]
    return `${bottom.key} (PF ${bottom.pf.toFixed(2)}, ${bottom.net.toFixed(1)} pips)`
  })

  const profitableMonths = computed(() =>
    monthSummaries.value.filter(m => m.net > 0).length
  )

  const weekdayStats = computed(() => {
    const labels = ['Dim', 'Lun', 'Mar', 'Mer', 'Jeu', 'Ven', 'Sam']
    const stats = new Map<number, { net: number; trades: number }>()
    for (const trade of executedTrades.value) {
      const date = new Date(trade.event_date)
      if (Number.isNaN(date.getTime())) continue
      const day = date.getDay()
      const entry = stats.get(day) || { net: 0, trades: 0 }
      entry.net += trade.pips_net
      entry.trades += 1
      stats.set(day, entry)
    }
    const items = Array.from(stats.entries()).map(([day, value]) => ({
      label: labels[day],
      net: value.net,
      trades: value.trades,
      avg: value.trades > 0 ? value.net / value.trades : 0
    }))
    return items
  })

  const bestWeekday = computed(() => {
    if (weekdayStats.value.length === 0) return 'n/a'
    const sorted = [...weekdayStats.value].sort((a, b) => b.avg - a.avg)
    const top = sorted[0]
    return `${top.label} (${top.avg.toFixed(1)} pips/trade)`
  })

  const worstWeekday = computed(() => {
    if (weekdayStats.value.length === 0) return 'n/a'
    const sorted = [...weekdayStats.value].sort((a, b) => a.avg - b.avg)
    const bottom = sorted[0]
    return `${bottom.label} (${bottom.avg.toFixed(1)} pips/trade)`
  })

  const costPerTrade = computed(() => (config.spread_pips * 2 + config.slippage_pips * 2).toFixed(1))
  const costTotal = computed(() =>
    (parseFloat(costPerTrade.value) * executedCount.value).toFixed(1)
  )
  const costRatio = computed(() => {
    const total = Math.abs(result.total_pips)
    if (total <= 0) return '0.0'
    return ((parseFloat(costTotal.value) / total) * 100).toFixed(1)
  })

  const advanced = computed(() => ({
    execution: {
      avgDuration: avgDuration.value,
      medianDuration: medianDuration.value,
      quickWinRate: quickWinRate.value,
      quickLossRate: quickLossRate.value,
      tpRate: tpRate.value,
      timeoutRate: timeoutRate.value,
      executedCount: executedCount.value
    },
    mfeMae: {
      avgMfe: avgMfe.value,
      avgMae: avgMae.value,
      mfeMaeRatio: mfeMaeRatio.value,
      tpPotentialRate: tpPotentialRate.value,
      tpMissRate: tpMissRate.value,
      tpPips: tpPips.value
    },
    trailing: {
      beHitRate: beHitRate.value,
      trailingExitRate: trailingExitRate.value
    },
    stability: {
      bestMonth: bestMonth.value,
      worstMonth: worstMonth.value,
      profitableMonths: profitableMonths.value,
      totalMonths: monthSummaries.value.length,
      bestWeekday: bestWeekday.value,
      worstWeekday: worstWeekday.value
    },
    fees: {
      costPerTrade: costPerTrade.value,
      costTotal: costTotal.value,
      costRatio: costRatio.value
    }
  }))

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
    finalRecommendation,
    advanced
  }
}
