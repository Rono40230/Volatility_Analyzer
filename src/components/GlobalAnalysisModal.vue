<template>
  <div
    v-if="isOpen"
    class="modal-overlay"
    @click.self="close"
  >
    <div class="modal-content">
      <div class="modal-header">
        <h2>✨ IAnalyse Statistique - Archives</h2>
        <button
          class="close-button"
          @click="close"
        >
          ×
        </button>
      </div>

      <div class="modal-body">
        <!-- Section: Statistiques Clés -->
        <section class="content-section">
          <h3 class="section-title">📊 Statistiques Globales</h3>
          <GlobalStatsBlock />
        </section>

        <!-- Section: Événements Tradables -->
        <section class="content-section">
          <h3 class="section-title">📅 Analyse des Événements</h3>
          <EventAnalysisBlock />
        </section>

        <!-- Section: Performance par Paire -->
        <section class="content-section">
          <h3 class="section-title">💱 Analyse par Paire</h3>
          <PairAnalysisBlock />
        </section>

        <!-- Section: Timing & Setup Straddle -->
        <section class="content-section">
          <h3 class="section-title">⏱️ Timing & Setup Straddle</h3>
          <TimingAnalysisBlock />
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { defineEmits, defineProps } from 'vue'
import GlobalStatsBlock from './analysis/GlobalStatsBlock.vue'
import EventAnalysisBlock from './analysis/EventAnalysisBlock.vue'
import PairAnalysisBlock from './analysis/PairAnalysisBlock.vue'
import TimingAnalysisBlock from './analysis/TimingAnalysisBlock.vue'
import { useArchiveStatistics } from '../composables/useArchiveStatistics'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

useArchiveStatistics()

function close() {
  emit('close')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(5px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: #13131f;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  width: calc(100vw - 32px);
  max-width: calc(100vw - 32px);
  height: auto;
  max-height: calc(100vh - 32px);
  display: flex;
  flex-direction: column;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  overflow: hidden;
  color: #e2e8f0;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: linear-gradient(90deg, #1a1a2e, #16213e);
}

.modal-header h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(90deg, #4ecdc4, #556270);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  display: flex;
  align-items: center;
  gap: 10px;
}

.close-button {
  background: none;
  border: none;
  color: #a0aec0;
  font-size: 24px;
  cursor: pointer;
  transition: color 0.2s;
}

.close-button:hover {
  color: #fff;
}

.modal-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px;
  background: radial-gradient(circle at top right, #1a1a2e 0%, #13131f 100%);
}

.results-container {
  animation: fadeIn 0.5s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

/* Scrollbar */
.modal-body::-webkit-scrollbar {
  width: 8px;
}

.modal-body::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.2);
}

.modal-body::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}

.modal-body::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

.content-section {
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.content-section:last-child {
  border-bottom: none;
  margin-bottom: 0;
  padding-bottom: 0;
}

.section-title {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 600;
  color: #4ecdc4;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
