<script setup lang="ts">
import type { MinuteDetail } from '../types/entryAnalysis'
import { pipsToDisplayValue } from '../utils/assetUnit'

const props = defineProps<{
  details: MinuteDetail[]
  optimalOffset: number
  quarterStartMinute: number
  symbol?: string
}>()

function toDisplay(val: number): number {
  return props.symbol ? pipsToDisplayValue(val, props.symbol) : val
}

function cellClass(d: MinuteDetail): string {
  if (!d.tradable) return 'non-tradable'
  if (d.offset === props.optimalOffset) return 'optimal'
  if (d.avg_net_profit_pips > 0 && d.win_rate >= 0.55) return 'profitable'
  if (d.avg_net_profit_pips > 0) return 'marginal'
  return 'losing'
}

function formatWinRate(rate: number): string {
  return (rate * 100).toFixed(0) + '%'
}

function formatProfit(val: number): string {
  const d = toDisplay(val)
  return d >= 0 ? '+' + d.toFixed(1) : d.toFixed(1)
}
</script>

<template>
  <div class="minute-breakdown">
    <div class="breakdown-header">
      <h3>Détail par minute (offset 0–14)</h3>
    </div>
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            <th>Minute</th>
            <th>Win Rate</th>
            <th>Profit Net</th>
            <th>Spread</th>
            <th>Samples</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="d in details"
            :key="d.offset"
            :class="cellClass(d)"
          >
            <td class="minute-col">
              <span class="minute-time">
                {{ String(Math.floor((quarterStartMinute + d.offset) / 60)).padStart(2, '0') }}:{{ String((quarterStartMinute + d.offset) % 60).padStart(2, '0') }}
              </span>
              <span class="minute-offset">+{{ d.offset }}</span>
            </td>
            <td class="wr-col">
              <span>{{ formatWinRate(d.win_rate) }}</span>
            </td>
            <td class="profit-col">
              <span :class="d.avg_net_profit_pips >= 0 ? 'profit-positive' : 'profit-negative'">
                {{ formatProfit(d.avg_net_profit_pips) }}
              </span>
            </td>
            <td class="spread-col">{{ toDisplay(d.avg_spread_pips).toFixed(1) }}</td>
            <td class="sample-col">{{ d.sample_size }}</td>
            <td class="status-col">
              <span v-if="d.offset === optimalOffset" class="badge badge-optimal">★ OPTIMAL</span>
              <span v-else-if="!d.tradable" class="badge badge-blocked">BLOCKED</span>
              <span v-else-if="d.avg_net_profit_pips > 0 && d.win_rate >= 0.55" class="badge badge-good">OK</span>
              <span v-else-if="d.avg_net_profit_pips > 0" class="badge badge-marginal">~</span>
              <span v-else class="badge badge-loss">—</span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.minute-breakdown { background: #161b22; border: 1px solid #30363d; border-radius: 10px; overflow: hidden; }
.breakdown-header { padding: 12px 16px; border-bottom: 1px solid #21262d; }
.breakdown-header h3 { margin: 0; font-size: 0.9em; color: #8b949e; text-transform: uppercase; letter-spacing: 0.5px; }
.table-wrapper { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85em; }
thead th { padding: 8px 12px; text-align: left; color: #8b949e; font-weight: 600; font-size: 0.8em; text-transform: uppercase; background: #0d1117; border-bottom: 1px solid #21262d; }
tbody tr { border-bottom: 1px solid #21262d; transition: background 0.15s; }
tbody tr:hover { background: rgba(56, 139, 253, 0.05); }
tbody td { padding: 8px 12px; color: #e6edf3; }
tr.optimal { background: rgba(31, 111, 235, 0.12); border-left: 3px solid #1f6feb; }
tr.profitable { background: rgba(35, 134, 54, 0.06); }
tr.marginal { background: rgba(210, 153, 34, 0.04); }
tr.losing { opacity: 0.7; }
tr.non-tradable { background: rgba(248, 81, 73, 0.06); opacity: 0.6; }
.minute-col { display: flex; align-items: center; gap: 8px; }
.minute-time { font-family: 'JetBrains Mono', monospace; font-weight: 600; }
.minute-offset { color: #6e7681; font-size: 0.8em; }
.wr-col { text-align: center; }
.profit-positive { color: #3fb950; font-weight: 600; }
.profit-negative { color: #f85149; font-weight: 600; }
.spread-col { color: #8b949e; }
.sample-col { color: #6e7681; text-align: center; }
.badge { padding: 2px 8px; border-radius: 4px; font-size: 0.75em; font-weight: 700; text-transform: uppercase; }
.badge-optimal { background: #1f6feb; color: #fff; }
.badge-good { background: rgba(35, 134, 54, 0.2); color: #3fb950; }
.badge-marginal { background: rgba(210, 153, 34, 0.2); color: #d29922; }
.badge-loss { background: rgba(110, 118, 129, 0.15); color: #6e7681; }
.badge-blocked { background: rgba(248, 81, 73, 0.15); color: #f85149; }
</style>
