<template>
  <div class="filters-container">
    <div class="filter-group">
      <label for="volatility-threshold">Volatilité minimale :</label>
      <select id="volatility-threshold" v-model.number="minVol" class="filter-select">
        <option value="3">≥30 points</option>
        <option value="6">≥60 points</option>
        <option value="9">≥90 points</option>
        <option value="12">≥120 points</option>
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
    <div class="filter-group">
      <label for="event-type">Type d'événement :</label>
      <select id="event-type" v-model="selectedEventType" class="filter-select">
        <option value="">Tous les événements</option>
        <option v-for="event in availableEventTypes" :key="event" :value="event">
          {{ event }}
        </option>
      </select>
    </div>
    <button 
      class="reload-button"
      title="Recharger la heatmap avec les données actuelles"
      @click="$emit('reload-heatmap')"
    >
      🔄 Recharger
    </button>
    <button 
      class="archive-button"
      title="Archiver la heatmap actuelle"
      @click="$emit('archive-heatmap')"
    >
      💾 Archiver
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'

const props = defineProps<{ 
  minVolatility: number
  maxEvents: number
  availableEventTypes?: string[]
}>()

const emit = defineEmits<{ 
  'update:minVolatility': [val: number]
  'update:maxEvents': [val: number]
  'update:selectedEventType': [val: string]
  'reload-heatmap': []
  'archive-heatmap': []
}>()

const minVol = ref(props.minVolatility)
const maxEvents = ref(props.maxEvents)
const selectedEventType = ref('')

watch(minVol, (v) => emit('update:minVolatility', v))
watch(maxEvents, (v) => emit('update:maxEvents', v))
watch(selectedEventType, (v) => emit('update:selectedEventType', v))

const availableEventTypes = computed(() => props.availableEventTypes ?? [])
</script>

<style scoped>
.filters-container { display: flex; gap: 30px; align-items: center; margin-bottom: 25px; padding: 15px; background: #2d3748; border-radius: 8px; flex-wrap: wrap; }
.filter-group { display: flex; align-items: center; gap: 10px; }
.filter-group label { color: #cbd5e0; font-size: 0.9em; font-weight: 500; white-space: nowrap; }
.filter-select { 
  padding: 8px 12px; 
  background: #1a202c; 
  border: 1px solid #4a5568; 
  border-radius: 6px; 
  color: #000000; 
  cursor: pointer; 
  min-width: 150px;
  font-size: 1em;
  font-weight: 500;
}
.filter-select:hover { border-color: #667eea; }
.filter-select option { 
  color: #000000;
}
.reload-button {
  padding: 8px 16px;
  background: linear-gradient(135deg, #1f6feb 0%, #388bfd 100%);
  border: 2px solid #58a6ff;
  color: #ffffff;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
  white-space: nowrap;
}
.reload-button:hover {
  background: linear-gradient(135deg, #388bfd 0%, #1f6feb 100%);
  box-shadow: 0 0 12px rgba(88, 166, 255, 0.4);
  transform: translateY(-2px);
}
.archive-button {
  padding: 8px 16px;
  background: linear-gradient(135deg, #27ae60 0%, #2ecc71 100%);
  border: 2px solid #2ecc71;
  color: #ffffff;
  border-radius: 6px;
  font-size: 1em;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
  white-space: nowrap;
}
.archive-button:hover {
  background: linear-gradient(135deg, #2ecc71 0%, #27ae60 100%);
  box-shadow: 0 0 12px rgba(46, 204, 113, 0.4);
  transform: translateY(-2px);
}
</style>
