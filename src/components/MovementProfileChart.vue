<script setup lang="ts">
import { computed } from 'vue'
import type { MinuteDetail } from '../types/entryAnalysis'

const props = defineProps<{
  details: MinuteDetail[]
  peakMinute: number
  decaySpeed: string
}>()

const chartW = 600
const chartH = 240
const padL = 40
const padR = 16
const padT = 14
const padB = 28
const innerW = chartW - padL - padR
const innerH = chartH - padT - padB

// Échelle profit
const profitRange = computed(() => {
  const vals = props.details.map(d => d.avg_net_profit_pips)
  const min = Math.min(0, ...vals)
  const max = Math.max(1, ...vals)
  return { min, max }
})

function xAt(i: number): number {
  return padL + (i / Math.max(1, props.details.length - 1)) * innerW
}

function yProfit(val: number): number {
  const { min, max } = profitRange.value
  const ratio = (val - min) / (max - min || 1)
  return padT + innerH * (1 - ratio)
}

function ySpread(val: number): number {
  const maxS = Math.max(1, ...props.details.map(d => d.avg_spread_pips))
  const ratio = val / maxS
  return padT + innerH * (1 - ratio)
}

// Ligne SVG pour profit
const profitPath = computed(() => {
  if (props.details.length === 0) return ''
  return props.details
    .map((d, i) => `${i === 0 ? 'M' : 'L'}${xAt(i).toFixed(1)},${yProfit(d.avg_net_profit_pips).toFixed(1)}`)
    .join(' ')
})

// Ligne SVG pour spread
const spreadPath = computed(() => {
  if (props.details.length === 0) return ''
  return props.details
    .map((d, i) => `${i === 0 ? 'M' : 'L'}${xAt(i).toFixed(1)},${ySpread(d.avg_spread_pips).toFixed(1)}`)
    .join(' ')
})

// Zone remplie sous la courbe de profit (jusqu'à y=0)
const profitArea = computed(() => {
  if (props.details.length === 0) return ''
  const baseline = yProfit(0)
  const points = props.details.map((d, i) =>
    `${xAt(i).toFixed(1)},${yProfit(d.avg_net_profit_pips).toFixed(1)}`
  )
  return `M${xAt(0).toFixed(1)},${baseline} L${points.join(' L')} L${xAt(props.details.length - 1).toFixed(1)},${baseline} Z`
})

// Ligne zéro
const zeroY = computed(() => yProfit(0))

// Labels axe X (toutes les 3 minutes)
const xLabels = computed(() =>
  props.details.filter((_, i) => i % 3 === 0).map((d, _i, arr) => ({
    x: xAt(props.details.indexOf(d)),
    label: `+${d.offset}`
  }))
)

// Peak marker
const peakX = computed(() => {
  const idx = props.details.findIndex(d => d.offset === props.peakMinute)
  return idx >= 0 ? xAt(idx) : 0
})
const peakY = computed(() => {
  const d = props.details.find(d => d.offset === props.peakMinute)
  return d ? yProfit(d.avg_net_profit_pips) : 0
})
</script>

<template>
  <div class="movement-profile">
    <div class="profile-header">
      <h3>Profil de mouvement</h3>
      <div class="legend">
        <span class="legend-profit">● Profit net</span>
        <span class="legend-spread">● Spread</span>
      </div>
    </div>
    <svg
      :viewBox="`0 0 ${chartW} ${chartH}`"
      class="profile-svg"
      preserveAspectRatio="xMidYMid meet"
    >
      <!-- Grille -->
      <line
        :x1="padL" :y1="zeroY" :x2="padL + innerW" :y2="zeroY"
        stroke="#30363d" stroke-dasharray="3,3"
      />
      <!-- Zone remplie profit -->
      <path :d="profitArea" fill="rgba(63, 185, 80, 0.08)" />
      <!-- Courbe spread -->
      <path :d="spreadPath" fill="none" stroke="#f0883e" stroke-width="1.5" opacity="0.6" />
      <!-- Courbe profit -->
      <path :d="profitPath" fill="none" stroke="#3fb950" stroke-width="2" />
      <!-- Peak marker -->
      <circle
        v-if="peakX > 0"
        :cx="peakX" :cy="peakY" r="4"
        fill="#1f6feb" stroke="#fff" stroke-width="1.5"
      />
      <text
        v-if="peakX > 0"
        :x="peakX" :y="peakY - 10"
        text-anchor="middle" fill="#58a6ff" font-size="10"
      >
        pic +{{ peakMinute }}
      </text>
      <!-- Labels axe X -->
      <text
        v-for="lbl in xLabels"
        :key="lbl.label"
        :x="lbl.x" :y="chartH - 4"
        text-anchor="middle" fill="#6e7681" font-size="9"
      >
        {{ lbl.label }}
      </text>
      <!-- Label axe Y : 0 -->
      <text
        :x="padL - 4" :y="zeroY + 3"
        text-anchor="end" fill="#6e7681" font-size="9"
      >
        0
      </text>
    </svg>
    <div class="decay-info">
      Décroissance :
      <span :class="'decay-' + decaySpeed.toLowerCase()">
        {{ decaySpeed === 'FAST' ? '⚡ Rapide' : decaySpeed === 'MEDIUM' ? '🔄 Modérée' : decaySpeed === 'SLOW' ? '🐢 Lente' : '❓ N/A' }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.movement-profile {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 10px;
  padding: 14px;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.profile-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.profile-header h3 {
  margin: 0;
  font-size: 0.9em;
  color: #8b949e;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.legend {
  display: flex;
  gap: 12px;
  font-size: 0.75em;
}

.legend-profit { color: #3fb950; }
.legend-spread { color: #f0883e; }

.profile-svg {
  display: block;
  width: 100%;
  flex: 1;
  min-height: 0;
}

.decay-info {
  margin-top: 8px;
  font-size: 0.8em;
  color: #8b949e;
  text-align: center;
}

.decay-fast { color: #f85149; font-weight: 600; }
.decay-medium { color: #d29922; font-weight: 600; }
.decay-slow { color: #3fb950; font-weight: 600; }
.decay-unknown { color: #6e7681; }
</style>
