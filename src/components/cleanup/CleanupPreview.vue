<template>
  <div class="tab-content preview-mode">
    <div class="controls-bar">
      <div class="control-group">
        <button class="btn-back" @click="$emit('close')">← Retour</button>
        <span class="preview-title">Aperçu : <strong>{{ title }}</strong></span>
        <button v-if="modifiedGroups.size > 0" class="btn-primary btn-sm ml-2" @click="saveChanges" :disabled="isSaving">
          {{ isSaving ? 'Enregistrement...' : 'Enregistrer les modifications' }}
        </button>
      </div>
      <div class="stats-badge">
        {{ events.length }} événements
      </div>
    </div>

    <div v-if="loading" class="state-container">
      <div class="spinner"></div>
      <p>Chargement de l'aperçu...</p>
    </div>

    <div v-else class="table-wrapper">
      <CleanupPreviewTable 
        :events="localEvents" 
        :allCountries="allCountries"
        @update-symbol="handleUpdateSymbol"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits, ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CleanupPreviewTable from './CleanupPreviewTable.vue'
import type { CalendarEvent, CurrencySummary } from '../../types/cleanup'

const props = defineProps<{
  events: CalendarEvent[]
  title: string
  loading: boolean
  allCountries?: CurrencySummary[]
}>()

const emit = defineEmits(['close', 'refresh'])

const localEvents = ref<CalendarEvent[]>([])
const modifiedGroups = ref<Map<string, { description: string, oldSymbol: string, newSymbol: string }>>(new Map())
const isSaving = ref(false)

watch(() => props.events, (newEvents) => {
  localEvents.value = JSON.parse(JSON.stringify(newEvents))
  modifiedGroups.value.clear()
}, { immediate: true })

function handleUpdateSymbol(event: CalendarEvent, newSymbol: string) {
  const oldSymbol = event.symbol
  const description = event.description
  
  // Update UI
  localEvents.value.forEach(e => {
    if (e.description === description && e.symbol === oldSymbol) {
      e.symbol = newSymbol
    }
  })
  
  // Track change
  const key = `${description}|${oldSymbol}`
  modifiedGroups.value.set(key, { description, oldSymbol, newSymbol })
}

async function saveChanges() {
  if (modifiedGroups.value.size === 0) return
  
  isSaving.value = true
  try {
    for (const change of modifiedGroups.value.values()) {
      await invoke('update_symbol_for_description', {
        description: change.description,
        oldSymbol: change.oldSymbol,
        newSymbol: change.newSymbol
      })
    }
    emit('refresh')
    emit('close')
  } catch (e) {
    // Silent error
  } finally {
    isSaving.value = false
  }
}
</script>

<style scoped>
.preview-mode .controls-bar {
  background: #1e293b;
  border-bottom: 1px solid #334155;
  border-radius: 0;
  margin: -24px -24px 20px -24px;
  padding: 16px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-back {
  background: transparent;
  border: 1px solid #475569;
  color: #cbd5e1;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.btn-back:hover {
  background: #334155;
  color: #fff;
}

.preview-title {
  color: #e2e8f0;
  font-size: 0.95rem;
}

.stats-badge {
  background: rgba(59, 130, 246, 0.1);
  color: #60a5fa;
  padding: 4px 12px;
  border-radius: 20px;
  font-size: 0.875rem;
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.state-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  gap: 16px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-radius: 50%;
  border-top-color: #f59e0b;
  animation: spin 1s linear infinite;
}

.table-wrapper {
  flex: 1;
  overflow-y: auto;
  border: 1px solid #334155;
  border-radius: 8px;
  background: #0f172a;
  min-height: 0;
}

.btn-primary {
  background: #3b82f6;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: background 0.2s;
}

.btn-primary:hover {
  background: #2563eb;
}

.btn-primary:disabled {
  background: #475569;
  cursor: not-allowed;
}

.ml-2 {
  margin-left: 8px;
}

@keyframes spin { to { transform: rotate(360deg); } }
</style>
