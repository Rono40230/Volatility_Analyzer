<template>
  <div class="retro-header">
    <div class="header-content">
      <div class="title-row">
        <span class="title-text">📊 Impact de l'événement {{ eventLabel || eventType }}{{ pair ? ' sur la volatilité de ' + pair : '' }}</span>
        <span class="separator">-</span>
        <span :class="advice.class">
          {{ advice.text }}
        </span>
      </div>
      <div class="stats-row" v-if="avgDeviation > 0">
        <span class="stat-item">
          <span class="stat-label">Écart Moyen:</span>
          <span class="stat-value">{{ avgDeviation.toFixed(2) }}</span>
        </span>
        <span class="stat-item">
          <span class="stat-label">Surprises:</span>
          <span class="stat-value">{{ surpriseEventCount }}</span>
        </span>
      </div>
    </div>
    <button v-if="!isArchiveMode" class="btn-archive" @click="$emit('archive')">💾 Archiver</button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  eventLabel?: string
  eventType: string
  pair: string
  volatilityIncreasePercent: number
  noiseRatioAfter: number
  isArchiveMode?: boolean
  avgDeviation?: number
  surpriseEventCount?: number
}

const props = withDefaults(defineProps<Props>(), {
  eventLabel: '',
  eventType: '',
  pair: '',
  volatilityIncreasePercent: 0,
  noiseRatioAfter: 0,
  isArchiveMode: false,
  avgDeviation: 0,
  surpriseEventCount: 0
})

defineEmits<{ archive: [] }>()

const advice = computed(() => {
  const vol = props.volatilityIncreasePercent
  const noise = props.noiseRatioAfter

  if (vol < 10) {
    return {
      text: `⚠️ Impact faible (+${vol.toFixed(1)}%) - Pas d'opportunité claire`,
      class: 'conclusion-neutral'
    }
  }

  if (noise > 3.0) {
    return {
      text: `⛔ DANGER (+${vol.toFixed(1)}% Vol) : Trop de bruit (Whipsaw) - Ratio R/R défavorable`,
      class: 'conclusion-danger'
    }
  }

  if (noise > 2.0) {
    return {
      text: `⚠️ PRUDENCE (+${vol.toFixed(1)}% Vol) : Mouvement bruyant - Nécessite SL large`,
      class: 'conclusion-warning'
    }
  }

  return {
    text: `✅ OPPORTUNITÉ (+${vol.toFixed(1)}% Vol) : Mouvement directionnel propre`,
    class: 'conclusion-success'
  }
})
</script>

<style scoped>
.retro-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 8px 20px;
  margin-bottom: 10px;
  height: 60px;
}

.header-content {
  display: flex;
  align-items: center;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 1.1em;
  font-weight: 600;
}

.title-text {
  color: #e6edf3;
}

.separator {
  color: #8b949e;
}

.stats-row {
  display: flex;
  gap: 15px;
  margin-top: 4px;
  font-size: 0.9em;
  color: #8b949e;
}

.stat-item {
  display: flex;
  gap: 6px;
}

.stat-value {
  color: #e6edf3;
  font-weight: 500;
}

.conclusion-success {
  color: #3fb950;
}

.conclusion-warning {
  color: #d29922;
}

.conclusion-danger {
  color: #f85149;
}

.conclusion-neutral {
  color: #8b949e;
}

.btn-archive {
  background: #238636;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.9em;
  transition: background 0.2s;
}

.btn-archive:hover {
  background: #2ea043;
}
</style>


