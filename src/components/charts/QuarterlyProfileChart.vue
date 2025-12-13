<template>
  <div class="chart-container">
    <svg viewBox="0 0 400 200" class="chart-svg">
      <!-- Grid Lines -->
      <line x1="40" y1="180" x2="380" y2="180" stroke="#30363d" stroke-width="1" />
      <line x1="40" y1="20" x2="40" y2="180" stroke="#30363d" stroke-width="1" />

      <!-- Quarter Background (Pastel Red) -->
      <rect :x="getX(0)" y="20" :width="getX(15) - getX(0)" height="160" fill="#f87171" opacity="0.1" stroke="none" />

      <!-- Vertical Markers for Quarter Start/End -->
      <line v-if="isExtended" :x1="getX(0)" y1="20" :x2="getX(0)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />
      <line v-if="isExtended" :x1="getX(15)" y1="20" :x2="getX(15)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />

      <!-- X Axis Labels (Minutes) -->
      <text v-for="m in xAxisLabels" :key="m" :x="getX(m)" y="195" font-size="7" fill="#8b949e" text-anchor="middle">{{ m }}m</text>

      <!-- Y Axis Labels -->
      <text x="35" :y="getY(maxValue)" font-size="7" fill="#8b949e" text-anchor="end">{{ maxValue.toFixed(1) }}</text>
      <text x="35" :y="getY(0)" font-size="7" fill="#8b949e" text-anchor="end">0</text>

      <!-- Profile Line -->
      <polyline :points="points" fill="none" stroke="#58a6ff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />

      <!-- Area under curve -->
      <path :d="areaPath" fill="url(#blueGradient)" stroke="none" opacity="0.3" />

      <!-- Volatility Zone (Pastel Green) -->
      <rect v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="getX(optimalEntry)"
            y="20"
            :width="volatilityZoneWidth"
            height="160"
            fill="#bbf7d0"
            opacity="0.2"
            stroke="none" />

      <!-- Volatility Zone Label -->
      <text v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="getX(optimalEntry) + volatilityZoneWidth / 2"
            y="100"
            :transform="`rotate(-90, ${getX(optimalEntry) + volatilityZoneWidth / 2}, 100)`"
            font-size="7"
            fill="#22c55e"
            text-anchor="middle"
            dominant-baseline="middle"
            font-weight="bold"
            style="pointer-events: none; letter-spacing: 0.5px; text-shadow: 0px 0px 2px rgba(0,0,0,0.5);">
        Durée de l'impulsion : {{ duration }}m
      </text>

      <!-- Optimal Entry Marker -->
      <line v-if="optimalEntry !== undefined" :x1="getX(optimalEntry)" y1="20" :x2="getX(optimalEntry)" y2="180" stroke="#10b981" stroke-width="1" stroke-dasharray="4,4" />
      <text v-if="optimalEntry !== undefined" :x="getX(optimalEntry)" y="15" font-size="7" fill="#10b981" text-anchor="middle" font-weight="bold">{{ entryLabel || `Entrée (${optimalEntry}m)` }}</text>

      <!-- Gradients -->
      <defs>
        <linearGradient id="blueGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" style="stop-color:#58a6ff;stop-opacity:0.5" />
          <stop offset="100%" style="stop-color:#58a6ff;stop-opacity:0" />
        </linearGradient>
      </defs>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  profile: number[]
  optimalEntry?: number
  duration?: number
  entryLabel?: string
}>()

const isExtended = computed(() => props.profile.length > 15)
const minMinute = computed(() => isExtended.value ? -5 : 0)
const maxMinute = computed(() => isExtended.value ? 30 : 14) // 30 to include up to +15min after end (15+15)
const totalRange = computed(() => maxMinute.value - minMinute.value)

const xAxisLabels = computed(() => {
  if (isExtended.value) {
    // Échelle de 5 minutes : -5, 0, 5, 10, 15, 20, 25, 30, 35, 40, 45
    const labels = []
    for (let i = minMinute.value; i <= maxMinute.value; i += 5) {
      labels.push(i)
    }
    return labels
  }
  return [0, 5, 10, 14]
})

const maxValue = computed(() => {
  const max = Math.max(...props.profile)
  return max === 0 ? 1 : max // Évite la division par zéro, mais ne force pas 1 si max est petit (ex: 0.005)
})

const getX = (minute: number) => {
  const width = 340 // 380 - 40
  const offset = minute - minMinute.value
  // Ensure we don't divide by zero, though totalRange should be > 0
  const range = totalRange.value || 1
  return 40 + (offset / range) * width
}

const getY = (val: number) => {
  const height = 160 // 180 - 20
  const max = maxValue.value
  if (max === 0) return 180
  return 180 - (val / max) * height
}

const points = computed(() => {
  return props.profile.map((val, idx) => {
    const minute = minMinute.value + idx
    return `${getX(minute)},${getY(val)}`
  }).join(' ')
})

const areaPath = computed(() => {
  const pts = points.value
  const firstX = getX(minMinute.value)
  const lastX = getX(minMinute.value + props.profile.length - 1)
  const bottomY = 180
  return `M ${firstX},${bottomY} L ${pts} L ${lastX},${bottomY} Z`
})

const volatilityZoneWidth = computed(() => {
  if (props.optimalEntry === undefined || props.duration === undefined) return 0
  const startX = getX(props.optimalEntry)
  const endX = getX(props.optimalEntry + props.duration)
  // Clamp width to chart area
  const maxX = 380
  return Math.min(endX, maxX) - startX
})
</script>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  min-height: 150px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
}

.chart-svg {
  width: 100%;
  height: 100%;
}
</style>
