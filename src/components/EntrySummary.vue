<script setup lang="ts">
import { computed } from 'vue'
import type { EntryAnalysisResult } from '../types/entryAnalysis'
import { pipsToDisplayValue } from '../utils/assetUnit'

const props = defineProps<{
  result: EntryAnalysisResult
}>()

function toDisplay(val: number): number {
  return pipsToDisplayValue(val, props.result.symbol, props.result.unit)
}

const verdict = computed(() => {
  const r = props.result
  const wr = r.real_win_rate
  const profit = r.avg_net_profit_pips
  const consistency = r.consistency_score
  const blocked = r.non_tradable_minutes.length

  if (wr >= 0.65 && profit > 0 && consistency >= 0.6 && blocked <= 3) return 'excellent'
  if (wr >= 0.55 && profit > 0 && blocked <= 5) return 'good'
  if (wr >= 0.45 && profit > 0) return 'acceptable'
  if (profit > 0) return 'fragile'
  return 'avoid'
})

const verdictConfig = computed(() => {
  const map: Record<string, { icon: string; label: string; color: string; desc: string }> = {
    excellent: { icon: '🟢', label: 'EXCELLENT', color: '#3fb950', desc: 'Conditions optimales pour un straddle' },
    good: { icon: '🔵', label: 'BON', color: '#58a6ff', desc: 'Conditions favorables, entrée recommandée' },
    acceptable: { icon: '🟡', label: 'ACCEPTABLE', color: '#d29922', desc: 'Mérite une entrée prudente avec SL serré' },
    fragile: { icon: '🟠', label: 'FRAGILE', color: '#f0883e', desc: 'Profit positif mais consistance faible' },
    avoid: { icon: '🔴', label: 'À ÉVITER', color: '#f85149', desc: 'Conditions défavorables, ne pas trader' },
  }
  return map[verdict.value] ?? map.avoid
})

const insights = computed(() => {
  const r = props.result
  const lines: { icon: string; text: string; type: 'positive' | 'negative' | 'neutral' }[] = []
  const unit = r.unit || 'pips'

  // Win Rate
  if (r.real_win_rate >= 0.65) lines.push({ icon: '✅', text: `Win Rate solide à ${(r.real_win_rate * 100).toFixed(0)}%`, type: 'positive' })
  else if (r.real_win_rate >= 0.50) lines.push({ icon: '⚠️', text: `Win Rate moyen à ${(r.real_win_rate * 100).toFixed(0)}%`, type: 'neutral' })
  else lines.push({ icon: '❌', text: `Win Rate insuffisant (${(r.real_win_rate * 100).toFixed(0)}%)`, type: 'negative' })

  // Profit vs Spread
  const profitDisplay = toDisplay(r.avg_net_profit_pips)
  const spreadDisplay = toDisplay(r.avg_spread_at_entry_pips)
  const ratio = spreadDisplay > 0 ? profitDisplay / spreadDisplay : 0
  if (ratio >= 5) lines.push({ icon: '✅', text: `Ratio profit/spread excellent (${ratio.toFixed(1)}×)`, type: 'positive' })
  else if (ratio >= 2) lines.push({ icon: '✅', text: `Profit net ${profitDisplay.toFixed(1)} ${unit} couvre bien le spread (${ratio.toFixed(1)}×)`, type: 'positive' })
  else if (ratio >= 1) lines.push({ icon: '⚠️', text: `Profit net faible vs spread (${ratio.toFixed(1)}×)`, type: 'neutral' })
  else lines.push({ icon: '❌', text: `Profit net ne couvre pas le spread`, type: 'negative' })

  // Durée + Decay
  if (r.decay_speed === 'FAST') lines.push({ icon: '⚡', text: `Mouvement rapide (pic +${r.peak_minute} min) — nécessite réactivité`, type: 'neutral' })
  else if (r.decay_speed === 'SLOW') lines.push({ icon: '🐢', text: `Mouvement lent, durée ${r.movement_duration_minutes.toFixed(0)} min — confortable`, type: 'positive' })
  else lines.push({ icon: '🔄', text: `Décroissance modérée, durée ${r.movement_duration_minutes.toFixed(0)} min`, type: 'neutral' })

  // Consistance
  if (r.consistency_score >= 0.6) lines.push({ icon: '✅', text: `Mouvement consistant entre les sessions`, type: 'positive' })
  else if (r.consistency_score >= 0.3) lines.push({ icon: '⚠️', text: `Variabilité notable entre les sessions`, type: 'neutral' })
  else lines.push({ icon: '❌', text: `Très variable d'une session à l'autre`, type: 'negative' })

  // Zones bloquées
  const blocked = r.non_tradable_minutes.length
  if (blocked === 0) lines.push({ icon: '✅', text: `Aucune minute bloquée par le spread`, type: 'positive' })
  else if (blocked <= 5) lines.push({ icon: '⚠️', text: `${blocked}/15 minutes bloquées par le spread`, type: 'neutral' })
  else lines.push({ icon: '❌', text: `${blocked}/15 minutes bloquées — spread trop large`, type: 'negative' })

  return lines
})

const recommendation = computed(() => {
  const r = props.result
  const unit = r.unit || 'pips'
  const entry = r.optimal_entry_time_label
  const duration = r.movement_duration_minutes.toFixed(0)
  const tp = toDisplay(r.avg_net_profit_pips * 0.7).toFixed(1)
  const sl = toDisplay(r.avg_movement_pips * 0.5).toFixed(1)

  if (verdict.value === 'avoid') return `Ce créneau n'est pas favorable au trading.`
  return `Entrer à ${entry}, TP ~${tp} ${unit}, SL ~${sl} ${unit}, expiration ${duration} min.`
})
</script>

<template>
  <div class="entry-summary">
    <div class="summary-header">
      <h3>Synthèse</h3>
      <div class="verdict-badge" :style="{ background: verdictConfig.color + '22', borderColor: verdictConfig.color, color: verdictConfig.color }">
        {{ verdictConfig.icon }} {{ verdictConfig.label }}
      </div>
    </div>

    <p class="verdict-desc">{{ verdictConfig.desc }}</p>

    <div class="insights-list">
      <div
        v-for="(insight, i) in insights"
        :key="i"
        class="insight-row"
        :class="insight.type"
      >
        <span class="insight-icon">{{ insight.icon }}</span>
        <span class="insight-text">{{ insight.text }}</span>
      </div>
    </div>

    <div class="recommendation">
      <span class="reco-label">💡 Recommandation</span>
      <p class="reco-text">{{ recommendation }}</p>
    </div>
  </div>
</template>

<style scoped>
.entry-summary { background: #161b22; border: 1px solid #30363d; border-radius: 10px; padding: 16px; display: flex; flex-direction: column; gap: 12px; }
.summary-header { display: flex; justify-content: space-between; align-items: center; }
.summary-header h3 { margin: 0; font-size: 0.9em; color: #8b949e; text-transform: uppercase; letter-spacing: 0.5px; }
.verdict-badge { padding: 4px 14px; border-radius: 6px; font-weight: 700; font-size: 0.85em; border: 1px solid; }
.verdict-desc { margin: 0; font-size: 0.85em; color: #8b949e; }
.insights-list { display: flex; flex-direction: column; gap: 6px; }
.insight-row { display: flex; align-items: center; gap: 8px; padding: 6px 10px; border-radius: 6px; font-size: 0.85em; background: rgba(255, 255, 255, 0.02); }
.insight-row.positive { color: #3fb950; }
.insight-row.negative { color: #f85149; }
.insight-row.neutral { color: #d29922; }
.insight-icon { font-size: 1em; flex-shrink: 0; }
.insight-text { color: #e6edf3; }
.recommendation { background: rgba(31, 111, 235, 0.08); border: 1px solid rgba(31, 111, 235, 0.25); border-radius: 8px; padding: 10px 14px; }
.reco-label { font-size: 0.8em; font-weight: 600; color: #58a6ff; text-transform: uppercase; letter-spacing: 0.5px; }
.reco-text { margin: 4px 0 0; font-size: 0.9em; color: #e6edf3; line-height: 1.4; }
</style>
