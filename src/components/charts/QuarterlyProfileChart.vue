<template>
  <div class="chart-container">
    <!-- Event Filter Controls -->
    <div class="event-controls">
      <button 
        @click="filterMode = 'none'" 
        :class="{ active: filterMode === 'none' }"
        title="Masquer les événements"
      >
        🚫
      </button>
      <button 
        @click="filterMode = 'peak'" 
        :class="{ active: filterMode === 'peak' }"
        title="Événements proches du pic"
      >
        🎯
      </button>
      <button 
        @click="filterMode = 'all'" 
        :class="{ active: filterMode === 'all' }"
        title="Tous les événements"
      >
        👁️
      </button>
    </div>

    <svg viewBox="0 0 400 200" class="chart-svg">
      <!-- Grid Lines -->
      <line x1="40" y1="180" x2="380" y2="180" stroke="#30363d" stroke-width="1" />
      <line x1="40" y1="20" x2="40" y2="180" stroke="#30363d" stroke-width="1" />

      <!-- Quarter Background (Pastel Red) -->
      <rect :x="obtenirX(0)" y="20" :width="obtenirX(15) - obtenirX(0)" height="160" fill="#f87171" opacity="0.1" stroke="none" />

      <!-- Vertical Markers for Quarter Start/End -->
      <line v-if="isExtended" :x1="obtenirX(0)" y1="20" :x2="obtenirX(0)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />
      <line v-if="isExtended" :x1="obtenirX(15)" y1="20" :x2="obtenirX(15)" y2="180" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" />

      <!-- X Axis Labels (Exact Time) -->
      <text v-for="m in xAxisLabels" :key="m" :x="obtenirX(m)" y="195" font-size="7" fill="#8b949e" text-anchor="middle">{{ formaterHeure(m) }}</text>

      <!-- Y Axis Labels & Grid (Every 1 pip) -->
      <template v-for="tick in yAxisTicks" :key="tick">
        <line :x1="40" :y1="obtenirY(tick)" :x2="380" :y2="obtenirY(tick)" stroke="#30363d" stroke-width="1" stroke-dasharray="2,2" opacity="0.5" />
        <text x="35" :y="obtenirY(tick) + 2" font-size="7" fill="#8b949e" text-anchor="end">{{ tick }}</text>
      </template>

      <!-- Profile Line + Area (clipped to plot bounds) -->
      <g clip-path="url(#chartClip)">
        <polyline :points="points" fill="none" stroke="#58a6ff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
        <path :d="areaPath" fill="url(#blueGradient)" stroke="none" opacity="0.3" />
      </g>

      <!-- Volatility Zone (Pastel Green) -->
      <rect v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="obtenirX(optimalEntry)"
            y="20"
            :width="volatilityZoneWidth"
            height="160"
            fill="#bbf7d0"
            opacity="0.2"
            stroke="none" />

      <!-- Volatility Zone Label -->
      <text v-if="optimalEntry !== undefined && duration !== undefined && volatilityZoneWidth > 0"
            :x="obtenirX(optimalEntry) + volatilityZoneWidth / 2"
            y="100"
            :transform="`rotate(-90, ${obtenirX(optimalEntry) + volatilityZoneWidth / 2}, 100)`"
            font-size="7"
            fill="#22c55e"
            text-anchor="middle"
            dominant-baseline="middle"
            font-weight="bold"
            style="pointer-events: none; letter-spacing: 0.5px; text-shadow: 0px 0px 2px rgba(0,0,0,0.5);">
        Durée de l'impulsion : {{ duration }}m
      </text>

      <!-- Optimal Entry Marker -->
      <line v-if="optimalEntry !== undefined" :x1="obtenirX(optimalEntry)" y1="20" :x2="obtenirX(optimalEntry)" y2="180" stroke="#10b981" stroke-width="1" stroke-dasharray="4,4" />
      <text v-if="optimalEntry !== undefined" :x="obtenirX(optimalEntry)" y="15" font-size="7" fill="#10b981" text-anchor="middle" font-weight="bold">{{ entryLabel || `Entrée (${formaterHeure(optimalEntry)})` }}</text>

      <!-- Event Flags -->
      <g v-for="(event, index) in processedEvents" :key="index">
        <!-- Flag Pole -->
        <line 
          :x1="obtenirX(obtenirMinuteEvenement(event.time))" 
          y1="20" 
          :x2="obtenirX(obtenirMinuteEvenement(event.time))" 
          y2="180" 
          :stroke="obtenirCouleurEvenement(event.impact)" 
          stroke-width="1" 
          stroke-dasharray="2,2" 
          opacity="0.7" 
        />
        
        <!-- Flag Icon (Triangle) -->
        <path 
          :d="`M ${obtenirX(obtenirMinuteEvenement(event.time))} ${20 + event.stackIndex * 12} L ${obtenirX(obtenirMinuteEvenement(event.time)) + 6} ${24 + event.stackIndex * 12} L ${obtenirX(obtenirMinuteEvenement(event.time))} ${28 + event.stackIndex * 12} Z`" 
          :fill="obtenirCouleurEvenement(event.impact)" 
        />

        <!-- Frequency Label -->
        <text 
          :x="obtenirX(obtenirMinuteEvenement(event.time)) + 8" 
          :y="26 + event.stackIndex * 12" 
          font-size="6" 
          :fill="obtenirCouleurEvenement(event.impact)"
          alignment-baseline="middle"
        >x{{ event.frequency }}</text>

        <!-- Event Label (on hover via title) -->
        <title>{{ event.tooltip }}</title>
      </g>

      <!-- Gradients -->
      <defs>
        <clipPath id="chartClip">
          <rect x="40" y="20" width="340" height="160" />
        </clipPath>
        <linearGradient id="blueGradient" x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" style="stop-color:#58a6ff;stop-opacity:0.5" />
          <stop offset="100%" style="stop-color:#58a6ff;stop-opacity:0" />
        </linearGradient>
      </defs>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useQuarterlyProfileChart } from '../../composables/useQuarterlyProfileChart'

const props = defineProps<{
  profile: number[]
  optimalEntry?: number
  duration?: number
  entryLabel?: string
  hour?: number
  quarter?: number
  events?: Array<{
    time: string
    name: string
    impact: string
    currency: string
    frequency: number
  }>
}>()

const {
  filterMode,
  isExtended,
  xAxisLabels,
  yAxisTicks,
  points,
  areaPath,
  volatilityZoneWidth,
  processedEvents,
  formaterHeure,
  obtenirMinuteEvenement,
  obtenirX,
  obtenirY,
  obtenirCouleurEvenement
} = useQuarterlyProfileChart(props)

onMounted(() => {
  // Mounted
})

watch(() => props.events, (newEvents) => {
  // Events updated
}, { deep: true })
</script>

<style scoped>
.chart-container {
  width: 100%;
  height: 100%;
  min-height: 150px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 4px;
  position: relative;
}

.event-controls {
  position: absolute;
  top: 5px;
  right: 5px;
  display: flex;
  gap: 4px;
  z-index: 10;
}

.event-controls button {
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  padding: 2px 6px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
  color: rgba(255, 255, 255, 0.5);
}

.event-controls button:hover {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.8);
}

.event-controls button.active {
  background: rgba(88, 166, 255, 0.2);
  border-color: rgba(88, 166, 255, 0.5);
  color: #58a6ff;
}

.chart-svg {
  width: 100%;
  height: 100%;
}
</style>
