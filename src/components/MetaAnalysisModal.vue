<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>📊 Méta-Analyse des Archives</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-tabs">
        <button 
          :class="{ active: activeTab === 'divergence' }"
          @click="activeTab = 'divergence'"
        >
          📈 Divergence
        </button>
        <button 
          :class="{ active: activeTab === 'matrix' }"
          @click="activeTab = 'matrix'"
        >
          🗺️ Matrice Rentabilité
        </button>
        <button 
          :class="{ active: activeTab === 'leaderboard' }"
          @click="activeTab = 'leaderboard'"
        >
          🏆 Leaderboard
        </button>
        <button 
          :class="{ active: activeTab === 'optimizer' }"
          @click="activeTab = 'optimizer'"
        >
          ⚙️ Optimiseur
        </button>
      </div>

      <div class="modal-body">
        <div v-if="parsedData.length === 0" class="empty-state">
          <p>Aucune archive d'analyse rétrospective trouvée.</p>
        </div>

        <DivergenceGraph 
          v-else-if="activeTab === 'divergence'" 
          :data="divergenceData" 
        />

        <ProfitabilityMatrix 
          v-else-if="activeTab === 'matrix'" 
          :data="matrixData" 
        />

        <LeaderboardTable 
          v-else-if="activeTab === 'leaderboard'" 
          :data="leaderboardData" 
        />

        <OptimizerTable 
          v-else-if="activeTab === 'optimizer'" 
          :data="optimizerData" 
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useMetaAnalysis } from '../composables/useMetaAnalysis'
import DivergenceGraph from './meta-analysis/DivergenceGraph.vue'
import ProfitabilityMatrix from './meta-analysis/ProfitabilityMatrix.vue'
import LeaderboardTable from './meta-analysis/LeaderboardTable.vue'
import OptimizerTable from './meta-analysis/OptimizerTable.vue'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])

const activeTab = ref<'divergence' | 'matrix' | 'leaderboard' | 'optimizer'>('divergence')

const {
  parsedData,
  divergenceData,
  matrixData,
  leaderboardData,
  optimizerData
} = useMetaAnalysis()

function close() {
  emit('close')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
}

.modal-content {
  background: #1e1e1e;
  width: 90%;
  max-width: 1200px;
  height: 80vh;
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
}

.modal-header {
  padding: 20px;
  background: #252525;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #333;
}

.modal-header h2 {
  margin: 0;
  color: #fff;
}

.close-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 24px;
  cursor: pointer;
}

.close-btn:hover {
  color: #fff;
}

.modal-tabs {
  display: flex;
  background: #252525;
  padding: 0 20px;
  border-bottom: 1px solid #333;
}

.modal-tabs button {
  background: none;
  border: none;
  color: #aaa;
  padding: 15px 20px;
  cursor: pointer;
  font-size: 1rem;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.modal-tabs button:hover {
  color: #fff;
  background: #2a2a2a;
}

.modal-tabs button.active {
  color: #2196f3;
  border-bottom-color: #2196f3;
}

.modal-body {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
  color: #e0e0e0;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #888;
  font-size: 1.2rem;
}
</style>
