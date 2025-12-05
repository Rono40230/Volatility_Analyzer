<template>
  <div class="archives-container">
    <div class="archives-header">
      <div class="header-content">
        <div>
          <h1>🗄️ Archives</h1>
          <p class="subtitle">
            Consultez vos analyses sauvegardées
          </p>
        </div>
        <button
          class="ai-btn"
          @click="isGlobalAnalysisOpen = true"
        >
          ✨ IAnalyse
        </button>
      </div>
    </div>

    <GlobalAnalysisModal 
      :is-open="isGlobalAnalysisOpen"
      @close="isGlobalAnalysisOpen = false"
    />

    <!-- Modal de confirmation de suppression -->
    <div v-if="showDeleteConfirmModal" class="delete-confirm-overlay">
      <div class="delete-confirm-modal">
        <div class="delete-confirm-header">
          <div class="delete-confirm-icon">🗑️</div>
          <h2>Supprimer cette archive ?</h2>
        </div>
        
        <div class="delete-confirm-body">
          <p v-if="archiveToDelete" class="archive-title">
            <strong>{{ archiveToDelete.title }}</strong>
          </p>
          <p class="warning-text">
            Cette action est <strong>irréversible</strong>. L'archive sera supprimée de manière permanente.
          </p>
        </div>
        
        <div class="delete-confirm-actions">
          <button 
            class="btn-cancel"
            @click="cancelDelete"
          >
            ❌ Annuler
          </button>
          <button 
            class="btn-confirm-delete"
            @click="confirmArchiveDeletion"
          >
            🗑️ Supprimer définitivement
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="archiveStore.loading"
      class="loading"
    >
      <div class="spinner" />
      <p>Chargement des archives...</p>
    </div>

    <div
      v-else-if="archiveStore.archives.length === 0"
      class="empty-state"
    >
      <div class="empty-icon">
        📭
      </div>
      <h3>Aucune archive</h3>
      <p>Vous n'avez pas encore archivé d'analyse.</p>
      <p class="hint">
        Utilisez le bouton "💾 Archiver" dans vos analyses pour les sauvegarder ici.
      </p>
    </div>

    <div
      v-else
      class="archives-grid"
    >
      <div 
        v-for="archive in archiveStore.archives" 
        :key="archive.id" 
        class="archive-card"
      >
        <div class="archive-header">
          <div
            class="archive-type-badge"
            :class="getTypeClass(archive.archive_type)"
          >
            {{ archive.archive_type }}
          </div>
          <button
            class="delete-btn"
            title="Supprimer"
            @click="confirmDelete(archive)"
          >
            🗑️
          </button>
        </div>

        <h3
          class="archive-title"
          v-html="formatTitleHTML(archive.title)"
        />
        
        <div class="archive-meta">
          <div class="meta-item">
            <span class="meta-label">📅 Période:</span>
            <span class="meta-value">{{ formatPeriod(archive.period_start, archive.period_end) }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">🕒 Créé le:</span>
            <span class="meta-value">{{ formatDate(archive.created_at) }}</span>
          </div>
          <div
            v-if="archive.comment"
            class="meta-item comment"
          >
            <span class="meta-label">💬 Commentaire:</span>
            <span class="meta-value">{{ archive.comment }}</span>
          </div>
        </div>

        <div class="archive-actions">
          <button
            class="btn-view"
            @click="viewArchive(archive)"
          >
            👁️ Voir
          </button>
          <button
            class="btn-pdf"
            @click="exportPDF(archive)"
          >
            📄 PDF
          </button>
        </div>
      </div>
    </div>


    <!-- Modale de visualisation -->
    <MetricsAnalysisModal 
      v-if="showViewer && (selectedArchive?.archive_type === 'Volatilité brute' || selectedArchive?.archive_type === 'METRICS')"
      :analysis-result="viewerData.analysisResult"
      :is-open="showViewer"
      :is-archive-mode="true"
      :archived-data="viewerData"
      @close="closeViewer" 
    />

    <div
      v-else-if="showViewer"
      class="viewer-overlay"
      @click.self="closeViewer"
    >
      <div
        class="viewer-content"
        :class="{ 'viewer-large': selectedArchive?.archive_type === 'Heatmap' || selectedArchive?.archive_type === 'HEATMAP' }"
      >
        <div class="viewer-header">
          <h2>{{ selectedArchive?.title }}</h2>
          <button
            class="close-btn"
            @click="closeViewer"
          >
            ✕
          </button>
        </div>
        <div class="viewer-body scrollable">
          <EventCorrelationHeatmap
            v-if="selectedArchive?.archive_type === 'Heatmap' || selectedArchive?.archive_type === 'HEATMAP'"
            :archive-data="viewerData.heatmapData"
            :is-archive-mode="true"
          />
           
          <div
            v-else
            class="unsupported-type"
          >
            Type d'archive non supporté pour la visualisation: {{ selectedArchive?.archive_type }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useArchiveStore, type Archive } from '../stores/archiveStore'
import MetricsAnalysisModal from '../components/MetricsAnalysisModal.vue'
import EventCorrelationHeatmap from '../components/EventCorrelationHeatmap.vue'
import GlobalAnalysisModal from '../components/GlobalAnalysisModal.vue'

const archiveStore = useArchiveStore()
const selectedArchive = ref<Archive | null>(null)
const showViewer = ref(false)
const viewerData = ref<any>(null)
const isGlobalAnalysisOpen = ref(false)
const showDeleteConfirmModal = ref(false)
const archiveToDelete = ref<Archive | null>(null)

onMounted(async () => {
  await archiveStore.loadArchives()
})

function getTypeClass(type: string): string {
  const mapping: Record<string, string> = {
    'Volatilité brute': 'type-metrics',
    'Corrélation événement/paire': 'type-event',
    'Corrélation paire/événement': 'type-pair',
    'Heatmap': 'type-heatmap',
    // Anciens types pour rétrocompatibilité
    'METRICS': 'type-metrics',
    'EVENT_IMPACT': 'type-event',
    'PAIR_IMPACT': 'type-pair',
    'HEATMAP': 'type-heatmap'
  }
  return mapping[type] || 'type-default'
}

function formatPeriod(start: string, end: string): string {
  const startDate = new Date(start)
  const endDate = new Date(end)
  
  const format = (date: Date) => {
    return date.toLocaleDateString('fr-FR', { 
      day: 'numeric', 
      month: 'short', 
      year: 'numeric' 
    })
  }
  
  return `${format(startDate)} - ${format(endDate)}`
}

function formatTitleHTML(title: string): string {
  // Regex pour trouver (du ... au ...) au début et le séparer du reste
  const match = title.match(/^(\(du .*? au .*?\))\s+(.*)/)
  if (match) {
    return `${match[1]}<br>${match[2]}`
  }
  return title
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr)
  return date.toLocaleDateString('fr-FR', { 
    day: 'numeric', 
    month: 'long', 
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })
}

function viewArchive(archive: Archive) {
  try {
    const data = JSON.parse(archive.data_json)
    viewerData.value = data
    selectedArchive.value = archive
    showViewer.value = true
  } catch (e) {
    // Erreur silencieuse - JSON invalide
  }
}

function closeViewer() {
  showViewer.value = false
  selectedArchive.value = null
  viewerData.value = null
}

async function exportPDF(archive: Archive) {
  // 1. Ouvrir l'archive en mode visualisation
  viewArchive(archive)
  
  // 2. Attendre que le DOM soit mis à jour et que les composants soient rendus
  // On utilise un setTimeout pour laisser le temps aux graphiques/composants de s'initialiser
  setTimeout(() => {
    window.print()
  }, 500)
}

async function confirmDelete(archive: Archive) {
  archiveToDelete.value = archive
  showDeleteConfirmModal.value = true
}

async function confirmArchiveDeletion() {
  if (!archiveToDelete.value) return
  try {
    await archiveStore.deleteArchive(archiveToDelete.value.id)
    showDeleteConfirmModal.value = false
    archiveToDelete.value = null
  } catch (error) {
    // Erreur silencieuse - suppression échouée
  }
}

function cancelDelete() {
  showDeleteConfirmModal.value = false
  archiveToDelete.value = null
}
</script>

<style scoped>
.archives-container {
  padding: 30px;
  background: #0d1117;
  min-height: 100vh;
}

.archives-header {
  margin-bottom: 30px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.archives-header h1 {
  color: #e2e8f0;
  font-size: 2.5em;
  margin: 0 0 10px 0;
}

.ai-btn {
  background: linear-gradient(135deg, #64c8ff 0%, #a78bfa 100%);
  color: #1a1a2e;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  font-weight: 700;
  font-size: 1.1em;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 4px 15px rgba(167, 139, 250, 0.3);
  display: flex;
  align-items: center;
  gap: 10px;
}

.ai-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(167, 139, 250, 0.5);
  filter: brightness(1.1);
}

.ai-btn:active {
  transform: translateY(0);
}

.subtitle {
  color: #8b949e;
  font-size: 1.1em;
  margin: 0;
}

.loading {
  text-align: center;
  padding: 60px 20px;
  color: #e2e8f0;
}

.spinner {
  width: 50px;
  height: 50px;
  border: 4px solid #2d3748;
  border-top: 4px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 80px 20px;
  background: #161b22;
  border-radius: 12px;
  border: 1px solid #30363d;
}

.empty-icon {
  font-size: 5em;
  margin-bottom: 20px;
}

.empty-state h3 {
  color: #e2e8f0;
  font-size: 1.8em;
  margin-bottom: 10px;
}

.empty-state p {
  color: #8b949e;
  font-size: 1.1em;
  margin: 10px 0;
}

.hint {
  color: #58a6ff !important;
  font-style: italic;
}

.archives-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 25px;
}

.archive-card {
  background: #161b22;
  border-radius: 12px;
  border: 1px solid #30363d;
  padding: 20px;
  transition: all 0.3s;
}

.archive-card:hover {
  border-color: #58a6ff;
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(88, 166, 255, 0.2);
}

.archive-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.archive-type-badge {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.85em;
  font-weight: 600;
}

.type-metrics {
  background: #1f6feb;
  color: white;
}

.type-event {
  background: #f59e0b;
  color: white;
}

.type-pair {
  background: #10b981;
  color: white;
}

.type-heatmap {
  background: #dc2626;
  color: white;
}

.delete-btn {
  background: none;
  border: none;
  font-size: 1.2em;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.2s;
}

.delete-btn:hover {
  background: #dc2626;
}

.archive-title {
  color: #e2e8f0;
  font-size: 0.95em; /* Même taille que les métadonnées */
  font-weight: bold;
  margin: 0 0 15px 0;
  line-height: 1.4;
  /* Suppression des contraintes de ligne unique */
  white-space: normal;
  overflow: visible;
  text-overflow: clip;
}

.archive-meta {
  margin-bottom: 20px;
}

.meta-item {
  margin-bottom: 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  color: #8b949e;
  font-size: 0.9em;
  font-weight: 600;
}

.meta-value {
  color: #cbd5e0;
  font-size: 0.95em;
}

.meta-item.comment .meta-value {
  font-style: italic;
  color: #a0aec0;
}

.archive-actions {
  display: flex;
  gap: 10px;
}

.btn-view,
.btn-pdf {
  flex: 1;
  padding: 10px;
  border-radius: 6px;
  border: none;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-view {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-view:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-pdf {
  background: #2d3748;
  color: #cbd5e0;
}

.btn-pdf:hover {
  background: #4a5568;
}

/* Styles pour la modale viewer */
.viewer-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.viewer-content {
  background: #0d1117;
  border-radius: 12px;
  width: 90%;
  max-width: 1000px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  border: 1px solid #30363d;
}

.viewer-large {
  max-width: 1400px;
}

.viewer-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 25px;
  border-bottom: 1px solid #30363d;
  background: #161b22;
  border-radius: 12px 12px 0 0;
}

.viewer-header h2 {
  margin: 0;
  color: #e2e8f0;
  font-size: 1.5em;
}

.close-btn {
  background: none;
  border: none;
  color: #cbd5e0;
  font-size: 1.5em;
  cursor: pointer;
  padding: 0;
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: #30363d;
  color: #fff;
}

.viewer-body {
  padding: 25px;
  overflow-y: auto;
  flex: 1;
}

.unsupported-type {
  text-align: center;
  padding: 40px;
  color: #8b949e;
  font-size: 1.2em;
}

/* Modal de confirmation de suppression - Stylisé */
.delete-confirm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.delete-confirm-modal {
  background: linear-gradient(135deg, #1f1f2e 0%, #2a2a3e 100%);
  border-radius: 16px;
  border: 2px solid #f85149;
  box-shadow: 0 20px 60px rgba(248, 81, 73, 0.3), 0 0 40px rgba(248, 81, 73, 0.1);
  padding: 0;
  max-width: 450px;
  width: 90%;
  animation: modalSlideIn 0.3s ease-out;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.delete-confirm-header {
  padding: 30px;
  border-bottom: 2px solid rgba(248, 81, 73, 0.2);
  display: flex;
  align-items: center;
  gap: 15px;
}

.delete-confirm-icon {
  font-size: 32px;
  animation: iconBounce 0.6s ease-in-out;
}

@keyframes iconBounce {
  0%, 100% { transform: scale(1) rotate(0deg); }
  50% { transform: scale(1.1) rotate(-5deg); }
}

.delete-confirm-header h2 {
  margin: 0;
  color: #f85149;
  font-size: 1.5em;
  font-weight: 700;
}

.delete-confirm-body {
  padding: 25px 30px;
  color: #e6edf3;
}

.delete-confirm-body .archive-title {
  background: rgba(248, 81, 73, 0.1);
  border-left: 4px solid #f85149;
  padding: 15px;
  border-radius: 8px;
  margin: 0 0 15px 0;
  color: #f85149;
  font-weight: 600;
  word-break: break-word;
}

.delete-confirm-body .warning-text {
  margin: 0;
  color: #cbd5e0;
  line-height: 1.6;
  font-size: 0.95em;
}

.delete-confirm-actions {
  padding: 25px 30px;
  border-top: 1px solid rgba(248, 81, 73, 0.2);
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn-cancel {
  padding: 12px 24px;
  background: #2a2a3e;
  border: 2px solid #4a5568;
  color: #cbd5e0;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0.95em;
}

.btn-cancel:hover {
  background: #3a3a4e;
  border-color: #667eea;
  color: #e6edf3;
}

.btn-cancel:active {
  transform: scale(0.98);
}

.btn-confirm-delete {
  padding: 12px 24px;
  background: linear-gradient(135deg, #f85149 0%, #dc3545 100%);
  border: 2px solid #f85149;
  color: #ffffff;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0.95em;
  box-shadow: 0 4px 12px rgba(248, 81, 73, 0.3);
}

.btn-confirm-delete:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(248, 81, 73, 0.5);
  background: linear-gradient(135deg, #f95950 0%, #e03e4f 100%);
}

.btn-confirm-delete:active {
  transform: translateY(0) scale(0.98);
}

@media print {
  /* Masquer tout sauf la modale de visualisation */
  body > *:not(#app) {
    display: none !important;
  }

  #app > * {
    display: none !important;
  }

  /* Rendre visible le conteneur de l'application pour que ses enfants puissent être affichés */
  #app {
    display: block !important;
  }

  /* Cibler spécifiquement la vue Archives */
  .archives-container {
    display: block !important;
    padding: 0 !important;
    background: white !important;
    min-height: auto !important;
  }

  /* Masquer l'interface de la liste des archives */
  .archives-header,
  .loading,
  .empty-state,
  .archives-grid {
    display: none !important;
  }

  /* Afficher la modale de visualisation et ses enfants */
  .viewer-overlay,
  .modal-overlay {
    position: static !important;
    display: block !important;
    background: white !important;
    width: 100% !important;
    height: auto !important;
    overflow: visible !important;
  }

  .viewer-content,
  .modal-content {
    position: static !important;
    width: 100% !important;
    max-width: none !important;
    max-height: none !important;
    box-shadow: none !important;
    border: none !important;
    background: white !important;
    color: black !important;
    overflow: visible !important;
  }

  .viewer-body,
  .modal-section {
    padding: 0 !important;
    overflow: visible !important;
  }

  /* Masquer les éléments d'interface inutiles à l'impression */
  .viewer-header button,
  .modal-header button,
  .close-btn,
  .modal-footer,
  .btn-archive,
  .btn-primary {
    display: none !important;
  }

  /* Ajustements de couleurs pour l'impression */
  * {
    -webkit-print-color-adjust: exact !important;
    print-color-adjust: exact !important;
    color-adjust: exact !important;
  }

  h1, h2, h3, h4, h5, h6, p, span, div {
    color: black !important;
  }
  
  /* Exceptions pour les badges et éléments colorés importants */
  .rank-badge, .score, .scale-item, .heatmap-cell {
    color: inherit !important;
  }
  
  /* Fond sombre pour certains blocs si nécessaire, ou passage en clair */
  .slice-card, .metrics-section, .heatmap-container {
    background: white !important;
    border: 1px solid #ddd !important;
    color: black !important;
  }
}
</style>
