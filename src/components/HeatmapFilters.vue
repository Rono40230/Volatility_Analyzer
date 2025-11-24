<template>
  <div class="filters-container">
    <div class="filter-group">
      <label for="volatility-threshold">Volatilité minimale :</label>
      <select id="volatility-threshold" v-model.number="minVol" class="filter-select">
        <option value="3">≥3 pips</option>
        <option value="6">≥6 pips</option>
        <option value="9">≥9 pips</option>
        <option value="12">≥12 pips</option>
      </select>
    </div>
    <div class="filter-group">
      <label for="max-events">Nombre d'événements max :</label>
      <select id="max-events" v-model.number="maxEvents" class="filter-select">
        <option value="5">5 événements</option>
        <option value="10">10 événements</option>
        <option value="15">15 événements</option>
        <option value="20">20 événements</option>
      </select>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{ minVolatility: number; maxEvents: number }>()
const emit = defineEmits<{ 'update:minVolatility': [val: number]; 'update:maxEvents': [val: number] }>()

const minVol = ref(props.minVolatility)
const maxEvents = ref(props.maxEvents)

watch(minVol, (v) => emit('update:minVolatility', v))
watch(maxEvents, (v) => emit('update:maxEvents', v))
</script>

<style scoped>
.filters-container { display: flex; gap: 30px; align-items: center; margin-bottom: 25px; padding: 15px; background: #2d3748; border-radius: 8px; }
.filter-group { display: flex; align-items: center; gap: 10px; }
.filter-group label { color: #cbd5e0; font-size: 0.9em; font-weight: 500; }
.filter-select { padding: 8px 12px; background: #1a202c; border: 1px solid #4a5568; border-radius: 6px; color: #e2e8f0; cursor: pointer; }
.filter-select:hover { border-color: #667eea; }
</style>
