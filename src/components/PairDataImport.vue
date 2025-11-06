<template>
  <div class="pair-import">
    <!-- Liste des fichiers CSV disponibles -->
    <PairFilesList ref="filesListRef" />

    <!-- CSV Cleaner Section -->
    <CleanerSection 
      @import-completed="handleImportCompleted"
      @error="handleError"
    />

    <!-- Afficher uniquement les erreurs -->
    <div v-if="importError" class="import-error-message">
      ❌ {{ importError }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useVolatilityStore } from '../stores/volatility'
import { useDataRefresh } from '../composables/useDataRefresh'
import PairFilesList from './PairFilesList.vue'
import CleanerSection from './CleanerSection.vue'

const volatilityStore = useVolatilityStore()
const { triggerPairDataRefresh } = useDataRefresh()
const filesListRef = ref<InstanceType<typeof PairFilesList>>()
const importError = ref('')

async function handleImportCompleted(success: boolean) {
  if (success) {
    // Rafraîchir automatiquement la liste des fichiers
    if (filesListRef.value) {
      await filesListRef.value.refreshFiles()
    }
    
    // Rafraîchir automatiquement le store Pinia (pour le dropdown)
    await volatilityStore.loadSymbols()
    
    // Notifier tous les autres composants abonnés
    await triggerPairDataRefresh()
    
    // Effacer toute erreur précédente
    importError.value = ''
  } else {
    importError.value = 'Une erreur s\'est produite lors de l\'import'
  }
}

function handleError(message: string) {
  importError.value = message
}
</script>

<style scoped>
.pair-import {
  padding: 20px 0;
}

.import-error-message {
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid #f85149;
  border-radius: 8px;
  padding: 15px;
  margin: 20px 0;
  color: #f85149;
  font-weight: 500;
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
