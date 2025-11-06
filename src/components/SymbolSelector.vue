<template>
  <div class="symbol-selector-container">
    <label for="symbol-select">Sélectionnez une paire forex :</label>
    <select 
      id="symbol-select"
      v-model="selectedSymbol" 
      @change="onSymbolChange"
      :disabled="loading"
      class="symbol-select"
    >
      <option value="">-- Choisir un symbole --</option>
      <option 
        v-for="symbol in symbols" 
        :key="symbol.symbol" 
        :value="symbol.symbol"
      >
        {{ symbol.symbol }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import { storeToRefs } from 'pinia'

const store = useVolatilityStore()
const { symbols, loading } = storeToRefs(store)
const selectedSymbol = ref('')

const { onPairDataRefresh } = useDataRefresh()

const emit = defineEmits<{
  symbolSelected: [symbol: string]
}>()

onMounted(() => {
  store.loadSymbols()
  
  // S'abonner aux événements de rafraîchissement
  const unsubscribe = onPairDataRefresh(() => {
    store.loadSymbols()
  })
  
  // Se désabonner au démontage
  onBeforeUnmount(unsubscribe)
})

function onSymbolChange() {
  if (selectedSymbol.value) {
    emit('symbolSelected', selectedSymbol.value)
  }
}
</script>

<style scoped>
.symbol-selector-container {
  background: #1a202c;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
  margin-bottom: 30px;
  border: 1px solid #2d3748;
}

.symbol-selector-container label {
  display: block;
  color: #e2e8f0;
  font-weight: 600;
  margin-bottom: 10px;
  font-size: 1.05em;
}

.symbol-select {
  width: 100%;
  padding: 12px 16px;
  font-size: 1.1em;
  border: 2px solid #4a5568;
  border-radius: 8px;
  background: #2d3748;
  color: #e2e8f0;
  cursor: pointer;
  transition: all 0.3s;
}

.symbol-select:hover:not(:disabled) {
  border-color: #667eea;
  background: #374151;
}

.symbol-select:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.2);
}

.symbol-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.symbol-select option {
  background: #2d3748;
  color: #e2e8f0;
}
</style>
