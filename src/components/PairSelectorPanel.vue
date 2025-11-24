<template>
  <div v-if="!selectedPair && !loading" class="welcome">
    <div class="welcome-icon">💱</div>
    <h3>Analyse Rétrospective par Paire</h3>
    <div class="welcome-select-container">
      <label for="pair-select">Sélectionnez une paire :</label>
      <select id="pair-select" :value="modelValue" class="welcome-symbol-select" @change="$emit('update:modelValue', $event.target.value); $emit('load')">
        <option value="">Choisir une paire</option>
        <option v-for="pair in availablePairs" :key="pair" :value="pair">{{ pair }}</option>
      </select>
    </div>
  </div>

  <div v-else-if="pairCorrelation" class="pair-info-card">
    <div class="pair-header">
      <h3>{{ selectedPair }}</h3>
      <select v-if="!isArchiveMode" :value="modelValue" class="inline-select" @change="$emit('update:modelValue', $event.target.value); $emit('load')">
        <option value="">Changer de paire</option>
        <option v-for="pair in availablePairs" :key="pair" :value="pair">{{ pair }}</option>
      </select>
      <button v-if="!isArchiveMode" class="btn-archive" @click="$emit('archive')">💾 Archiver</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineProps, defineEmits } from 'vue'

defineProps<{
  modelValue: string
  availablePairs: string[]
  selectedPair: string
  pairCorrelation: any
  loading: boolean
  isArchiveMode: boolean
}>()

defineEmits<{
  'update:modelValue': [value: string]
  load: []
  archive: []
}>()
</script>

<style scoped>
.welcome { text-align: center; padding: 60px 20px; background: #1a202c; border-radius: 12px; border: 1px solid #2d3748; }
.welcome-icon { font-size: 4em; margin-bottom: 20px; }
.welcome h3 { font-size: 1.8em; color: #e2e8f0; margin-bottom: 15px; }
.welcome-select-container { display: flex; flex-direction: column; align-items: center; gap: 20px; margin-top: 30px; }
.welcome-select-container label { font-weight: 600; color: #cbd5e0; }
.welcome-symbol-select { padding: 12px 24px; font-size: 1.1em; border-radius: 8px; border: 2px solid #4a5568; background: #fff; color: #000; cursor: pointer; min-width: 300px; }
.welcome-symbol-select:hover { border-color: #667eea; background: #f7fafc; }
.pair-info-card { background: #1a202c; padding: 25px; border-radius: 12px; border: 1px solid #2d3748; }
.pair-header { display: flex; justify-content: space-between; align-items: center; gap: 15px; }
.pair-header h3 { margin: 0; color: #e2e8f0; font-size: 1.5em; }
.inline-select { padding: 8px 12px; border: 1px solid #4a5568; background: #1c2128; color: #000; border-radius: 6px; font-size: 0.95em; cursor: pointer; }
.inline-select:hover { border-color: #58a6ff; }
.btn-archive { padding: 10px 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: #fff; border: none; border-radius: 6px; font-weight: 600; cursor: pointer; white-space: nowrap; }
.btn-archive:hover { transform: translateY(-2px); box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4); }
</style>
