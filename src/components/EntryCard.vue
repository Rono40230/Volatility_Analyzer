<script setup lang="ts">
import type { EntryAnalysisResult } from '../types/entryAnalysis'
import { pipsToDisplayValue } from '../utils/assetUnit'

const props = defineProps<{
  result: EntryAnalysisResult
}>()

function toDisplay(val: number): number {
  return pipsToDisplayValue(val, props.result.symbol)
}

function formatPips(val: number): string {
  const d = toDisplay(val)
  return d >= 0 ? `+${d.toFixed(1)}` : d.toFixed(1)
}

function winRateClass(rate: number): string {
  if (rate >= 0.65) return 'excellent'
  if (rate >= 0.55) return 'good'
  if (rate >= 0.45) return 'neutral'
  return 'poor'
}

function consistencyLabel(score: number): string {
  if (score >= 0.8) return 'Très stable'
  if (score >= 0.6) return 'Stable'
  if (score >= 0.4) return 'Variable'
  return 'Instable'
}

function decayIcon(speed: string): string {
  switch (speed) {
    case 'FAST': return '⚡'
    case 'MEDIUM': return '🔄'
    case 'SLOW': return '🐢'
    default: return '❓'
  }
}
</script>

<template>
  <div class="entry-card">
    <div class="card-header">
      <div class="symbol-badge">{{ result.symbol }}</div>
      <div class="event-badge">{{ result.event_type }}</div>
      <div class="sample-badge">{{ result.sample_size }} occurrences</div>
    </div>

    <!-- Zone principale : minute optimale -->
    <div class="optimal-zone">
      <div class="optimal-time">
        <span class="time-label">Entrée optimale</span>
        <span class="time-value">{{ result.optimal_entry_time_label }}</span>
        <span class="offset-detail">offset +{{ result.optimal_offset_minutes }} min</span>
      </div>
    </div>

    <!-- Grille de métriques clés -->
    <div class="metrics-grid">
      <div class="metric" :class="winRateClass(result.real_win_rate)">
        <span class="metric-label">Win Rate</span>
        <span class="metric-value">{{ (result.real_win_rate * 100).toFixed(1) }}%</span>
        <span class="metric-detail">comptage réel</span>
      </div>

      <div class="metric" :class="result.avg_net_profit_pips > 0 ? 'excellent' : 'poor'">
        <span class="metric-label">Profit Net</span>
        <span class="metric-value">{{ formatPips(result.avg_net_profit_pips) }} {{ result.unit || 'pips' }}</span>
        <span class="metric-detail">après 2× spread</span>
      </div>

      <div class="metric">
        <span class="metric-label">Spread Entrée</span>
        <span class="metric-value">{{ toDisplay(result.avg_spread_at_entry_pips).toFixed(1) }} {{ result.unit || 'pips' }}</span>
        <span class="metric-detail">mesuré bid/ask</span>
      </div>

      <div class="metric">
        <span class="metric-label">Mouvement Brut</span>
        <span class="metric-value">{{ toDisplay(result.avg_movement_pips).toFixed(1) }} {{ result.unit || 'pips' }}</span>
        <span class="metric-detail">avant spread</span>
      </div>
    </div>

    <!-- Profil de mouvement -->
    <div class="movement-profile">
      <div class="profile-item">
        <span class="profile-label">Pic</span>
        <span class="profile-value">minute +{{ result.peak_minute }}</span>
      </div>
      <div class="profile-item">
        <span class="profile-label">Durée</span>
        <span class="profile-value">{{ result.movement_duration_minutes.toFixed(0) }} min</span>
      </div>
      <div class="profile-item">
        <span class="profile-label">Décroissance</span>
        <span class="profile-value">{{ decayIcon(result.decay_speed) }} {{ result.decay_speed === 'FAST' ? 'Rapide' : result.decay_speed === 'MEDIUM' ? 'Modérée' : result.decay_speed === 'SLOW' ? 'Lente' : 'N/A' }}</span>
      </div>
      <div class="profile-item">
        <span class="profile-label">Consistance</span>
        <div class="consistency-bar">
          <div
            class="consistency-fill"
            :style="{ width: (result.consistency_score * 100) + '%' }"
          />
        </div>
        <span class="profile-value-small">{{ consistencyLabel(result.consistency_score) }}</span>
      </div>
    </div>

    <!-- Alerte zones non-tradables -->
    <div
      v-if="result.non_tradable_minutes.length > 0"
      class="non-tradable-alert"
    >
      ⚠️ Spread excessif aux minutes :
      {{ result.non_tradable_minutes.map(m => '+' + m).join(', ') }}
    </div>
  </div>
</template>

<style scoped>
.entry-card { background: #161b22; border: 1px solid #30363d; border-radius: 12px; padding: 20px; display: flex; flex-direction: column; gap: 16px; }
.card-header { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.symbol-badge { background: #1f6feb; color: #fff; padding: 4px 12px; border-radius: 6px; font-weight: 700; font-size: 0.9em; }
.event-badge { background: #238636; color: #fff; padding: 4px 10px; border-radius: 6px; font-size: 0.85em; }
.sample-badge { color: #8b949e; font-size: 0.8em; margin-left: auto; }
.optimal-zone { background: linear-gradient(135deg, #0d1117 0%, #161b22 100%); border: 2px solid #1f6feb; border-radius: 10px; padding: 20px; text-align: center; }
.optimal-time { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.time-label { color: #8b949e; font-size: 0.85em; text-transform: uppercase; letter-spacing: 1px; }
.time-value { font-size: 2.2em; font-weight: 800; color: #58a6ff; font-family: 'JetBrains Mono', monospace; }
.offset-detail { color: #6e7681; font-size: 0.8em; }
.metrics-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; }
.metric { background: #0d1117; border: 1px solid #21262d; border-radius: 8px; padding: 12px; text-align: center; display: flex; flex-direction: column; gap: 4px; }
.metric.excellent { border-color: #238636; }
.metric.good { border-color: #1f6feb; }
.metric.neutral { border-color: #d29922; }
.metric.poor { border-color: #f85149; }
.metric-label { color: #8b949e; font-size: 0.75em; text-transform: uppercase; }
.metric-value { font-size: 1.3em; font-weight: 700; color: #e6edf3; }
.metric.excellent .metric-value { color: #3fb950; }
.metric.good .metric-value { color: #58a6ff; }
.metric.neutral .metric-value { color: #d29922; }
.metric.poor .metric-value { color: #f85149; }
.metric-detail { color: #6e7681; font-size: 0.7em; }
.movement-profile { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; padding: 12px; background: #0d1117; border-radius: 8px; border: 1px solid #21262d; }
.profile-item { display: flex; flex-direction: column; align-items: center; gap: 4px; }
.profile-label { color: #8b949e; font-size: 0.75em; text-transform: uppercase; }
.profile-value { color: #e6edf3; font-weight: 600; font-size: 0.95em; }
.profile-value-small { color: #8b949e; font-size: 0.7em; }
.consistency-bar { width: 60px; height: 6px; background: #21262d; border-radius: 3px; overflow: hidden; }
.consistency-fill { height: 100%; background: linear-gradient(90deg, #f85149, #d29922, #3fb950); border-radius: 3px; transition: width 0.3s ease; }
.non-tradable-alert { background: rgba(248, 81, 73, 0.1); border: 1px solid rgba(248, 81, 73, 0.3); border-radius: 8px; padding: 10px 14px; color: #f85149; font-size: 0.85em; }
@media (max-width: 700px) { .metrics-grid, .movement-profile { grid-template-columns: repeat(2, 1fr); } }
</style>
