<template>
  <svg
    :width="chartWidth"
    :height="chartHeight"
    class="decay-chart-svg"
  >
    <!-- Gradients -->
    <defs>
      <linearGradient id="profitGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" style="stop-color: #10b981; stop-opacity: 0.1;" />
        <stop offset="100%" style="stop-color: #10b981; stop-opacity: 0.05;" />
      </linearGradient>
      <linearGradient id="cautionGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" style="stop-color: #f59e0b; stop-opacity: 0.1;" />
        <stop offset="100%" style="stop-color: #f59e0b; stop-opacity: 0.05;" />
      </linearGradient>
      <linearGradient id="riskGradient" x1="0%" y1="0%" x2="0%" y2="100%">
        <stop offset="0%" style="stop-color: #ef4444; stop-opacity: 0.1;" />
        <stop offset="100%" style="stop-color: #ef4444; stop-opacity: 0.05;" />
      </linearGradient>
      <linearGradient id="curveGradient" x1="0%" y1="0%" x2="100%" y2="0%">
        <stop offset="0%" style="stop-color: #3b82f6; stop-opacity: 1;" />
        <stop offset="50%" style="stop-color: #06b6d4; stop-opacity: 1;" />
        <stop offset="100%" style="stop-color: #10b981; stop-opacity: 1;" />
      </linearGradient>
    </defs>

    <!-- Axes -->
    <line :x1="padding" :y1="chartHeight - padding" :x2="chartWidth - padding" :y2="chartHeight - padding" class="axis" />
    <line :x1="padding" :y1="padding" :x2="padding" :y2="chartHeight - padding" class="axis" />

    <!-- Grille -->
    <line v-for="i in 4" :key="`grid-h-${i}`" :x1="padding" :x2="chartWidth - padding" :y1="chartHeight - padding - (i * (chartHeight - 2 * padding) / 4)" :y2="chartHeight - padding - (i * (chartHeight - 2 * padding) / 4)" class="grid-line" />

    <!-- Zones -->
    <rect v-if="recommendedX > 0" :x="padding" :y="padding" :width="recommendedX - padding" :height="chartHeight - 2 * padding" fill="url(#profitGradient)" />
    <rect v-if="halfLifeX > recommendedX" :x="recommendedX" :y="padding" :width="halfLifeX - recommendedX" :height="chartHeight - 2 * padding" fill="url(#cautionGradient)" />
    <rect v-if="halfLifeX < chartWidth - padding" :x="halfLifeX" :y="padding" :width="chartWidth - padding - halfLifeX" :height="chartHeight - 2 * padding" fill="url(#riskGradient)" />

    <!-- Courbe -->
    <path :d="curvePathD" class="decay-curve" stroke="url(#curveGradient)" fill="none" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />

    <!-- Marqueurs -->
    <line :x1="padding" :y1="padding" :x2="padding" :y2="chartHeight - padding" class="marker-line peak" stroke-dasharray="5,5" />
    <circle :cx="padding" :cy="padding" r="6" class="marker-dot peak" />
    <text :x="padding" :y="padding - 15" class="marker-label">🟢 Pic</text>

    <line v-if="recommendedX > padding" :x1="recommendedX" :y1="padding" :x2="recommendedX" :y2="chartHeight - padding" class="marker-line recommended" stroke-dasharray="5,5" />
    <circle v-if="recommendedX > padding" :cx="recommendedX" :cy="recommendedY" r="6" class="marker-dot recommended" />
    <text v-if="recommendedX > padding" :x="recommendedX - 40" :y="recommendedY - 15" class="marker-label">🎯 TP</text>
    <text v-if="recommendedX > padding" :x="recommendedX - 50" :y="recommendedY + 5" class="marker-value">{{ recommendedDuration }}m</text>

    <line v-if="halfLifeX > padding" :x1="halfLifeX" :y1="padding" :x2="halfLifeX" :y2="chartHeight - padding" class="marker-line halflife" stroke-dasharray="5,5" />
    <circle v-if="halfLifeX > padding" :cx="halfLifeX" :cy="halfLifeY" r="6" class="marker-dot halflife" />
    <text v-if="halfLifeX > padding" :x="halfLifeX - 60" :y="halfLifeY - 15" class="marker-label">🟠 Half-life</text>
    <text v-if="halfLifeX > padding" :x="halfLifeX - 70" :y="halfLifeY + 5" class="marker-value">{{ halfLifeMinutes }}m</text>

    <!-- Étiquettes axes -->
    <text v-for="t in [0, 60, 120, 180, 240]" :key="`time-${t}`" :x="xScale(t)" :y="chartHeight - padding + 25" class="axis-label" text-anchor="middle">{{ formatTime(t) }}</text>
    <text :x="chartWidth / 2" :y="chartHeight - 5" class="axis-title">Heure de clôture</text>

    <text v-for="(label, i) in [100, 75, 50, 25, 0]" :key="`vol-${label}`" :x="padding - 15" :y="chartHeight - padding - (i * (chartHeight - 2 * padding) / 4) + 5" class="axis-label" text-anchor="end">{{ label }}%</text>
    <text :x="20" :y="chartHeight / 2" class="axis-title" text-anchor="middle" transform="rotate(-90 20 400)">Volatilité résiduelle</text>
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  peakVolatility: number
  halfLifeMinutes: number
  recommendedDuration: number
  maxTime?: number
  startHour?: number
  startMinute?: number
}

const props = withDefaults(defineProps<Props>(), {
  maxTime: 240,
  startHour: 13,
  startMinute: 0
})

const chartWidth = 800
const chartHeight = 400
const padding = 50

const lambda = Math.log(2) / props.halfLifeMinutes

const formatTime = (minutes: number): string => {
  const totalMinutes = props.startMinute + minutes
  let hours = props.startHour + Math.floor(totalMinutes / 60)
  let mins = totalMinutes % 60
  hours = hours % 24
  return `${String(hours).padStart(2, '0')}:${String(mins).padStart(2, '0')}`
}

const xScale = (t: number) => padding + (t / props.maxTime) * (chartWidth - 2 * padding)
const yScale = (volatility: number) => chartHeight - padding - volatility * (chartHeight - 2 * padding)
const getVolatilityAt = (t: number) => Math.exp(-lambda * t)

const curvePathD = computed(() => {
  const points: string[] = []
  points.push(`M ${xScale(0)} ${yScale(1)}`)
  for (let t = 0; t <= props.maxTime; t += 5) {
    const vol = getVolatilityAt(t)
    points.push(`L ${xScale(t)} ${yScale(vol)}`)
  }
  return points.join(' ')
})

const recommendedX = computed(() => xScale(props.recommendedDuration))
const recommendedY = computed(() => yScale(getVolatilityAt(props.recommendedDuration)))
const halfLifeX = computed(() => xScale(props.halfLifeMinutes))
const halfLifeY = computed(() => yScale(0.5))
</script>

<style scoped>
.decay-chart-svg {
  background: linear-gradient(to bottom, #1a202c 0%, #0f172a 100%);
  border-radius: 4px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.3);
}

.axis { stroke: #667eea; stroke-width: 2; }
.grid-line { stroke: #667eea; stroke-width: 1; opacity: 0.2; stroke-dasharray: 3,3; }
.decay-curve { filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3)); }

.marker-line { stroke-width: 2; opacity: 0.7; }
.marker-line.peak { stroke: #10b981; }
.marker-line.recommended { stroke: #3b82f6; }
.marker-line.halflife { stroke: #f59e0b; }

.marker-dot { stroke-width: 2; stroke: #0f172a; }
.marker-dot.peak { fill: #10b981; }
.marker-dot.recommended { fill: #3b82f6; }
.marker-dot.halflife { fill: #f59e0b; }

.marker-label { font-size: 12px; font-weight: 600; fill: #e2e8f0; text-anchor: middle; }
.marker-value { font-size: 11px; fill: #cbd5e0; text-anchor: middle; }
.axis-label { font-size: 12px; fill: #cbd5e0; }
.axis-title { font-size: 13px; font-weight: 500; fill: #cbd5e0; }
</style>
