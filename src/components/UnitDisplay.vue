<script setup lang="ts">
import { computed } from 'vue'
import { obtenirPointsParPip } from '../utils/pipConverter'

const props = defineProps<{
  value: number
  unit: string
  decimals?: number
  symbol?: string
}>()

// Valeur invalide (NaN, Infinity) → afficher N/A
const isInvalidValue = computed(() => {
  return isNaN(props.value) || !isFinite(props.value)
})

const pointsPerPip = computed(() => {
  if (props.symbol) return obtenirPointsParPip(props.symbol)
  // Sans symbole, pas de conversion fiable → afficher en unité brute (×1)
  return 1
})

// HYPOTHÈSE CRITIQUE (Suite Audit) : 
// La valeur d'entrée (props.value) venant du backend est TOUJOURS en PIPS (Normalisée).
// Le backend divise par 0.0001 pour le Forex, donc envoie des Pips.

const pipsValue = computed(() => props.value)

const pointsValue = computed(() => {
  return props.value * pointsPerPip.value
})

const pipsDecimals = computed(() => {
  const baseDecimals = props.decimals ?? 1
  return Math.max(baseDecimals, 1)
})

const pointsDecimals = computed(() => {
    return props.decimals ?? 1
})

const isPipsOrPoints = computed(() => {
  return ['pips', 'pts', 'points'].includes(props.unit)
})

const displayUnit = computed(() => {
  switch (props.unit) {
    case 'pips': return 'pips'
    case 'points':
    case 'pts': return 'pts'
    case '$': return ''
    default: return props.unit
  }
})

const prefix = computed(() => {
  return props.unit === '$' ? '$' : ''
})
</script>

<template>
  <span class="unit-display">
    <template v-if="isInvalidValue">
      <span class="na-value">N/A</span>
    </template>

    <template v-else-if="isPipsOrPoints">
      <!-- Affichage Standardisé : POINTS (soit PIPS) -->
      <!-- Ex: 150.0 pts (soit 15.0 pips) -->
      {{ pointsValue.toFixed(pointsDecimals) }} pts 
      <span class="sub-unit">(soit {{ pipsValue.toFixed(pipsDecimals) }} pips)</span>
    </template>
    
    <template v-else>
      <!-- Autres unités (%, $, etc.) -->
      {{ prefix }}{{ props.value.toFixed(props.decimals ?? 2) }} 
      <span v-if="displayUnit" class="unit">{{ displayUnit }}</span>
    </template>
  </span>
</template>

<style scoped>
.unit-display {
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.unit {
  font-size: 0.85em;
  opacity: 0.7;
  margin-left: 2px;
}
.sub-unit {
  font-size: 0.8em;
  opacity: 0.6;
  margin-left: 4px;
  font-style: italic;
}
.na-value {
  color: #8b949e;
  font-style: italic;
  opacity: 0.7;
}
</style>
