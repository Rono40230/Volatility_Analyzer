<template>
  <div class="retro-header">
    <div class="header-content">
      <div class="title-row">
        <span class="title-text">📊 Impact de l'événement {{ eventLabel || eventType }}{{ pair ? ' sur la volatilité de ' + pair : '' }}</span>
        <span class="separator">-</span>
        <span v-if="volatilityIncreasePercent > 0" class="conclusion-positive">
          ✅ Événement génère {{ (volatilityIncreasePercent).toFixed(1) }}% de volatilité {{ noiseQualityAfter === 'clean' ? 'directionnelle' : 'avec bruit' }}
        </span>
        <span v-else class="conclusion-negative">
          ⚠️ Événement peu corrélé à la volatilité sur {{ pair }}
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
}

const props = withDefaults(defineProps<Props>(), {
  eventLabel: '',
  eventType: '',
  pair: '',
  volatilityIncreasePercent: 0,
  noiseRatioAfter: 0,
  isArchiveMode: false
})

defineEmits<{ archive: [] }>()

const noiseQualityAfter = computed(() => props.noiseRatioAfter < 1.5 ? 'clean' : props.noiseRatioAfter < 2.5 ? 'mixed' : 'choppy')
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

.conclusion-positive {
  color: #3fb950;
}

.conclusion-negative {
  color: #f85149;
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


