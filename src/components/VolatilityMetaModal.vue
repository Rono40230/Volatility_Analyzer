<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Archive } from '../stores/archiveStore'

const props = defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits(['close'])
const fullArchives = ref<Archive[]>([])

onMounted(async () => {
  try {
    fullArchives.value = await invoke<Archive[]>('list_archives')
  } catch {
    // Erreur silencieuse
  }
})

function close() {
  emit('close')
}

// Extraction des données Volatilité
const parsedData = computed(() => {
  return fullArchives.value
    .filter(a => a.archive_type === 'Volatilité brute' || a.archive_type === 'Volatilité brute Paire/Période' || a.archive_type === 'METRICS')
    .map(a => {
      try {
        const data = JSON.parse(a.data_json)
        return {
          id: a.id,
          title: a.title,
          date: a.created_at,
          data: data
        }
      } catch {
        return null
      }
    })
    .filter(d => d !== null)
})
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-content">
      <div class="modal-header">
        <h2>📊 Méta-Analyse Volatilité</h2>
        <button class="close-btn" @click="close">×</button>
      </div>

      <div class="modal-body">
        <div v-if="parsedData.length === 0" class="empty-state">
          <p>Aucune archive de Volatilité trouvée.</p>
        </div>
        <div v-else class="placeholder-content">
          <div class="info-box">
            <h3>📈 Analyse des Tendances de Volatilité</h3>
            <p>Cette fonctionnalité permettra d'analyser les cycles de volatilité sur le long terme.</p>
            <ul>
              <li>Quelles heures sont structurellement les plus volatiles ?</li>
              <li>Évolution de l'ATR moyen par paire</li>
              <li>Détection des changements de régime de marché</li>
            </ul>
            <p class="dev-note">🚧 En cours de développement</p>
          </div>
          
          <div class="stats-grid">
            <div class="stat-card">
              <span class="label">Archives Volatilité</span>
              <span class="value">{{ parsedData.length }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
}

.modal-content {
  background: #1e1e1e;
  width: 80%;
  max-width: 800px;
  height: 60vh;
  border-radius: 12px;
  border: 1px solid #333;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
}

.modal-header {
  padding: 20px;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #252525;
  border-radius: 12px 12px 0 0;
}

.modal-header h2 {
  margin: 0;
  color: #e0e0e0;
}

.close-btn {
  background: none;
  border: none;
  color: #888;
  font-size: 24px;
  cursor: pointer;
}

.modal-body {
  flex: 1;
  padding: 30px;
  overflow-y: auto;
  background: #1e1e1e;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
  color: #666;
}

.info-box {
  background: #252525;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 20px;
  border-left: 4px solid #ff9800;
}

.info-box h3 {
  margin-top: 0;
  color: #ff9800;
}

.info-box ul {
  color: #ccc;
  line-height: 1.6;
}

.dev-note {
  margin-top: 20px;
  font-style: italic;
  color: #888;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 20px;
}

.stat-card {
  background: #2a2a2a;
  padding: 15px;
  border-radius: 8px;
  text-align: center;
}

.stat-card .label {
  display: block;
  color: #888;
  font-size: 0.9rem;
  margin-bottom: 5px;
}

.stat-card .value {
  font-size: 1.5rem;
  font-weight: bold;
  color: #fff;
}
</style>
