<template>
  <div class="impact-tab">
    <div class="tab-header">
      <div class="controls">
        <select v-model="selectedImpactFilter" class="impact-filter">
          <option value="" disabled>Filtrer par impact</option>
          <option v-for="impact in availableImpacts" :key="impact" :value="impact">
            {{ impact }}
          </option>
        </select>
        <button 
          class="btn-danger" 
          @click="confirmDelete" 
          :disabled="!selectedImpactFilter || filteredImpacts.length === 0"
        >
          🗑️ Supprimer les événements ({{ totalEvents }})
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading-state">Chargement...</div>
    
    <div v-else class="table-container">
      <table class="cleanup-table">
        <thead>
          <tr>
            <th>Description</th>
            <th>Impact</th>
            <th>Occurrences</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in filteredImpacts" :key="item.description">
            <td class="col-desc">{{ item.label || item.description }}</td>
            <td>
              <span class="impact-badge" :class="getImpactLabel(item.impact).toLowerCase()">
                {{ getImpactLabel(item.impact) }}
              </span>
            </td>
            <td>{{ item.count }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { ImpactGroupSummary } from '../../composables/useCleanupLogic'

const props = defineProps<{
  impacts: ImpactGroupSummary[]
  loading: boolean
}>()

const emit = defineEmits(['preview', 'update', 'delete'])

const selectedImpactFilter = ref('')

const availableImpacts = ['High', 'Medium', 'Low', 'Neutre']

const filteredImpacts = computed(() => {
  if (!selectedImpactFilter.value) return []
  
  return props.impacts.filter(i => {
    const raw = i.impact
    const normalized = raw ? raw.toLowerCase().trim() : ''
    const label = getImpactLabel(raw)
    
    if (selectedImpactFilter.value === 'Low') {
       // Force match for anything that looks like Low
       return normalized === 'low' || normalized === 'l' || label === 'Low'
    }
    
    return label === selectedImpactFilter.value
  })
})

function getImpactLabel(impact: string): string {
  if (!impact) return 'Neutre'
  const i = impact.toLowerCase().trim()
  
  // Exact matches first
  if (i === 'high' || i === 'h') return 'High'
  if (i === 'medium' || i === 'm') return 'Medium'
  if (i === 'low' || i === 'l') return 'Low'
  if (i === 'none' || i === 'n') return 'Neutre'
  
  // StartsWith fallback
  if (i.startsWith('h')) return 'High'
  if (i.startsWith('m')) return 'Medium'
  if (i.startsWith('l')) return 'Low'
  
  return 'Neutre'
}

const totalEvents = computed(() => {
  return filteredImpacts.value.length
})

function confirmDelete() {
  if (!selectedImpactFilter.value) return
  // Collect all unique raw impacts that match the current filter
  const rawImpacts = Array.from(new Set(filteredImpacts.value.map(i => i.impact)))
  emit('delete', rawImpacts, totalEvents.value, selectedImpactFilter.value)
}
</script>

<style scoped>
.impact-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0; /* Important for nested flex scrolling */
  gap: 1rem;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
}

.controls {
  display: flex;
  gap: 1rem;
  width: 100%;
}

.impact-filter {
  padding: 0.5rem;
  border-radius: 4px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  color: black;
  min-width: 200px;
  font-size: 1rem;
}

.table-container {
  flex: 1;
  overflow-y: auto;
  min-height: 0; /* Important for nested flex scrolling */
  border: 1px solid var(--border-color);
  border-radius: 4px;
}

.cleanup-table {
  width: 100%;
  border-collapse: collapse;
}

.cleanup-table th, .cleanup-table td {
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.impact-badge {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: bold;
}

.impact-badge.low { background: #4caf50; color: white; }
.impact-badge.medium { background: #ff9800; color: white; }
.impact-badge.high { background: #f44336; color: white; }
.impact-badge.neutre { background: #9e9e9e; color: white; }

.action-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2rem;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.action-btn:hover {
  opacity: 1;
}

.btn-danger {
  background: #f44336;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.btn-danger:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading-state {
  text-align: center;
  padding: 2rem;
  color: var(--text-secondary);
}
</style>
