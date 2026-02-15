<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>⚡ Analyse Rapide : {{ eventName }}</h2>
        <button class="close-button" @click="close">×</button>
      </div>
      <div class="modal-body">
        <RetroactiveAnalysisView 
          ref="retroViewRef"
          :calendar-id="calendarId ?? null" 
          :show-calendar-selector="false"
          :initial-pair="initialPair"
          :initial-event-type="eventName"
          :debug-log="pushDebugLog"
        />
      </div>
      <!-- Debug panel masqué en production -->
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import RetroactiveAnalysisView from '../RetroactiveAnalysisView.vue'

const props = defineProps<{
  isOpen: boolean
  eventName: string
  initialPair: string
  calendarId?: number | null
}>()

const emit = defineEmits<{
  close: []
}>()

const retroViewRef = ref<InstanceType<typeof RetroactiveAnalysisView> | null>(null)
const debugLogs = ref<string[]>([])

function pushDebugLog(message: string) {
  const ts = new Date().toLocaleTimeString('fr-FR', { hour12: false })
  debugLogs.value = [`${ts} ${message}`, ...debugLogs.value].slice(0, 50)
}

watch(
  () => [props.isOpen, props.initialPair, props.eventName, props.calendarId],
  async ([isOpen, pair, eventName]) => {
    if (!isOpen || !pair || !eventName) return
    await nextTick()
    pushDebugLog(`Modal open -> load ${pair} / ${eventName}`)
    retroViewRef.value?.triggerLoad(pair, eventName)
  },
  { immediate: true }
)

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
  z-index: 2000;
  padding: 20px;
}

.modal-content {
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 12px;
  width: 95vw;
  height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0,0,0,0.5);
  position: relative;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  border-bottom: 1px solid #30363d;
  background: #161b22;
  border-radius: 12px 12px 0 0;
}

.modal-header h2 {
  margin: 0;
  font-size: 1.2em;
  color: #e2e8f0;
}

.close-button {
  background: none;
  border: none;
  color: #8b949e;
  font-size: 24px;
  cursor: pointer;
  padding: 0 8px;
}

.close-button:hover {
  color: white;
}

.modal-body {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* Debug panel supprimé */

.debug-line {
  white-space: pre-wrap;
}

/* Force le composant enfant à prendre toute la hauteur */
.modal-body :deep(.container) {
  height: 100%;
  border-radius: 0 0 12px 12px;
}
</style>
