<script setup lang="ts">
import { ref, computed } from 'vue'
import { 
  getDefaultPairForCurrency, 
  getEventDisplayInfo, 
  getImpactColor, 
  getConfidenceColor 
} from '../../utils/planningUtils'

interface EventAction {
  id: string
  time: string
  name: string
  currency: string
  impact: 'High' | 'Medium' | 'Low'
  pair: string
  confidence_score: number
  source: string
  has_history: boolean
  occurrence_count: number
}

const props = defineProps<{
  event: EventAction
  availablePairs: string[]
}>()

const emit = defineEmits<{
  (e: 'analyze', event: EventAction, pair: string): void
}>()

const selectedPair = ref(props.event.source === 'Archive' ? 'EURUSD' : getDefaultPairForCurrency(props.event.currency))

const hasArchive = computed(() => props.event.source === 'Archive')
const canAnalyze = computed(() => props.event.has_history)

const impactColor = computed(() => getImpactColor(props.event.impact))
const confidenceColor = computed(() => getConfidenceColor(props.event.confidence_score))
const displayInfo = computed(() => getEventDisplayInfo(props.event.name, props.event.currency, props.event.has_history))
</script>

<template>
  <div class="event-card">
    <div class="event-header">
      <div class="time-badge">{{ event.time }}</div>
      <div class="event-info">
        <div class="event-title">
          <span class="name" :title="displayInfo.title">
            {{ displayInfo.title }} <span class="flag">{{ displayInfo.flag }}</span>
          </span>
        </div>
        
        <!-- Dropdown des paires sous le titre -->
        <div class="pair-selector-row">
          <select v-model="selectedPair" class="pair-select" @click.stop>
            <option v-for="pair in availablePairs" :key="pair" :value="pair">{{ pair }}</option>
          </select>

          <span class="impact-badge" :style="{ backgroundColor: impactColor }">
            {{ event.impact }} Impact
          </span>
          
          <span class="occurrence-badge" title="Nombre d'occurrences historiques">
            📚 {{ event.occurrence_count }} occurences dans le calendrier historique
          </span>
          
          <!-- Bouton Analyser (si pas d'archive mais historique dispo) -->
          <button 
            v-if="!hasArchive && canAnalyze"
            class="btn-analyze" 
            title="Lancer l'analyse historique" 
            @click="emit('analyze', event, selectedPair)"
          >
            📊 Analyser
          </button>
        </div>

        <div class="indicators">
          <span v-if="hasArchive" class="confidence-score" :style="{ color: confidenceColor }">
            {{ event.confidence_score }}% Confiance
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.event-card {
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  transition: transform 0.2s, box-shadow 0.2s;
}

.event-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  border-color: var(--primary-color);
}

.event-header {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
}

.time-badge {
  background: var(--bg-secondary);
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  white-space: nowrap;
}

.event-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  min-width: 0; /* Permet au text-overflow de fonctionner */
}

.event-title {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.1rem;
}

.name {
  font-weight: 600;
  font-size: 0.95rem;
  color: var(--text-primary);
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.flag {
  font-size: 1.1em;
  margin-left: 4px;
}

.pair-selector-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
  height: 24px;
}

.pair-select {
  background: var(--bg-tertiary);
  color: #000000;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.8rem;
  cursor: pointer;
  min-width: 80px;
}

.pair-select:focus {
  outline: none;
  border-color: var(--primary-color);
}

.btn-analyze {
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: background-color 0.2s;
  white-space: nowrap;
}

.btn-analyze:hover {
  background: var(--primary-hover);
}

.indicators {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.8rem;
}

.impact-badge {
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  white-space: nowrap;
  color: white;
}

.occurrence-badge {
  font-size: 0.7rem;
  padding: 2px 6px;
  border-radius: 4px;
  font-weight: 600;
  white-space: nowrap;
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 4px;
}

.confidence-score {
  font-weight: 600;
}
</style>
