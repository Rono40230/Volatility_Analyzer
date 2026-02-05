<template>
  <div class="archives-container">
    <div class="archives-header">
      <div class="header-content">
        <div class="header-title-section">
          <div>
            <h1>🗄️ Archives</h1>
            <p class="subtitle">
              Consultez vos analyses sauvegardées
            </p>
          </div>
          <div class="header-pair-filter">
            <label for="pair-select" class="filter-label">💱 Paire :</label>
            <select 
              id="pair-select"
              v-model="selectedPair"
              class="pair-select"
            >
              <option value="all">Toutes les paires</option>
              <option 
                v-for="pair in availablePairs" 
                :key="pair"
                :value="pair"
              >
                {{ pair }}
              </option>
            </select>
          </div>
          <button 
            class="btn-delete-all" 
            @click="showDeleteAllConfirmModal = true" 
            v-if="archiveStore.archives.length > 0"
            title="Tout effacer"
          >
            🗑️ Supprimer toutes les archives
          </button>
        </div>
        <!-- Bouton IAnalyse supprimé -->
      </div>
    </div>

    <MetaAnalysisModal 
      :is-open="showMetaAnalysisModal"
      @close="showMetaAnalysisModal = false"
    />

    <HeatmapMetaModal 
      :is-open="showHeatmapMetaModal"
      @close="showHeatmapMetaModal = false"
    />

    <VolatilityMetaModal 
      :is-open="showVolatilityMetaModal"
      @close="showVolatilityMetaModal = false"
    />

    <ExportModal 
      :is-open="showExportModal"
      @close="showExportModal = false"
    />

    <!-- GlobalAnalysisModal supprimé -->

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

    <div v-if="showDeleteAllConfirmModal" class="delete-confirm-overlay">
      <div class="delete-confirm-modal delete-all-modal">
        <div class="delete-confirm-header">
          <div class="delete-confirm-icon">⚠️</div>
          <h2>ATTENTION<br>SUPPRESSION TOTALE</h2>
        </div>
        
        <div class="delete-confirm-body">
          <p class="warning-text big-warning">
            Toutes les archives seront<br><strong>DÉFINITIVEMENT SUPPRIMÉES</strong>.
          </p>
          <p class="warning-detail">
            Cette action est irréversible.<br>Êtes-vous absolument sûr de vouloir tout effacer ?
          </p>
        </div>
        
        <div class="delete-confirm-actions">
          <button 
            class="btn-cancel"
            @click="showDeleteAllConfirmModal = false"
          >
            ❌ Annuler
          </button>
          <button 
            class="btn-confirm-delete btn-danger-zone"
            @click="confirmDeleteAll"
          >
            🔥 TOUT EFFACER
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
      class="archives-container-accordion"
    >
      <!-- Sections d'archives groupées par type -->
      <div 
        v-for="(archives, type) in archivesByType" 
        :key="type"
        class="archive-section"
      >
        <div 
          class="section-header"
          @click="basculerExpansionType(type)"
        >
          <div class="section-header-left">
            <span class="section-toggle">
              {{ expandedTypes.has(type) ? '▼' : '▶' }}
            </span>
            <div
              class="archive-type-badge"
              :class="obtenirClasseType(type)"
            >
              {{ type }}
            </div>
            <span class="section-count">{{ archives.length }} archive{{ archives.length > 1 ? 's' : '' }}</span>
          </div>

          <button 
            v-if="getMetaAnalysisType(type)" 
            class="btn-meta-analysis-small"
            @click.stop="openMetaAnalysis(type)"
          >
            📊 Méta-Analyse
          </button>
        </div>

        <!-- Grille d'archives (compact) -->
        <transition name="collapse">
          <div 
            v-if="expandedTypes.has(type)"
            class="archives-grid-compact"
          >
            <div 
              v-for="archive in archives" 
              :key="archive.id" 
              class="archive-card-compact"
            >
              <div class="card-header">
                <h3 class="card-title" v-html="formatTitleHTML(archive.title)" />
                <button
                  class="delete-btn"
                  title="Supprimer"
                  @click="confirmDelete(archive)"
                >
                  🗑️
                </button>
              </div>

              <div class="card-meta-compact">
                <div class="meta-row">
                  <span class="meta-label">📅</span>
                  <span class="meta-value">{{ formatPeriod(archive.period_start, archive.period_end) }}</span>
                </div>
                <div class="meta-row">
                  <span class="meta-label">🕒</span>
                  <span class="meta-value">{{ formatDate(archive.created_at) }}</span>
                </div>
                <div v-if="(archive.archive_type === 'Métriques Rétrospectives' || archive.archive_type === 'Correlation de la volatilité Paire/Evenement') && extractEventLabel(archive) !== 'Événement inconnu'" class="meta-row">
                  <span class="meta-label">📊</span>
                  <span class="meta-value">{{ extractEventLabel(archive) }}</span>
                </div>
              </div>

              <div v-if="archive.archive_type === 'Backtest' && getBacktestSummary(archive)" class="backtest-summary">
                <div class="summary-item">
                  <span class="summary-label">P/L</span>
                  <span class="summary-value" :class="{ 'positive': getBacktestSummary(archive).total_pips > 0, 'negative': getBacktestSummary(archive).total_pips < 0 }">
                    {{ getBacktestSummary(archive).total_pips.toFixed(1) }} pips
                  </span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">Win Rate</span>
                  <span class="summary-value">{{ formatPercent(getBacktestSummary(archive).win_rate_percent) }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">PF</span>
                  <span class="summary-value">{{ getBacktestSummary(archive).profit_factor.toFixed(2) }}</span>
                </div>
                <div class="summary-item">
                  <span class="summary-label">DD</span>
                  <span class="summary-value negative">{{ getBacktestSummary(archive).max_drawdown_pips.toFixed(1) }}</span>
                </div>
              </div>

              <div class="card-actions-compact">
                <button
                  class="btn-action-compact btn-view"
                  @click="viewArchive(archive)"
                >
                  👁️ Voir
                </button>
              </div>
            </div>
          </div>
        </transition>
      </div>
    </div>


    <!-- Modale de visualisation -->
    <MetricsAnalysisModal 
      v-if="showViewer && (selectedArchive?.archive_type === 'Volatilité brute' || selectedArchive?.archive_type === 'Volatilité brute Paire/Période' || selectedArchive?.archive_type === 'METRICS')"
      :analysis-result="viewerData.analysisResult"
      :is-open="showViewer"
      :is-archive-mode="true"
      :archived-data="viewerData"
      @close="closeViewer" 
    />

    <RetroactiveAnalysisResultsViewer
      v-else-if="showViewer && (selectedArchive?.archive_type === 'Métriques Rétrospectives' || selectedArchive?.archive_type === 'Correlation de la volatilité Paire/Evenement' || selectedArchive?.archive_type === 'RETRO_ANALYSIS')"
      :data="viewerData"
      @close="closeViewer"
    />

    <div
      v-else-if="showViewer"
      class="viewer-overlay"
      @click.self="closeViewer"
    >
      <div
        class="viewer-content"
        :class="{ 'viewer-large': selectedArchive?.archive_type === 'Heatmap' || selectedArchive?.archive_type === 'HEATMAP' || selectedArchive?.archive_type === 'Backtest' }"
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

          <BacktestResultsPanel
            v-else-if="selectedArchive?.archive_type === 'Backtest'"
            :archived-data="viewerData"
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
import { onMounted, ref, computed } from 'vue'
import { useArchiveStore, type Archive } from '../stores/archiveStore'
import MetricsAnalysisModal from '../components/MetricsAnalysisModal.vue'
import RetroactiveAnalysisResultsViewer from '../components/RetroactiveAnalysisResultsViewer.vue'
import EventCorrelationHeatmap from '../components/EventCorrelationHeatmap.vue'
import BacktestResultsPanel from '../components/BacktestResultsPanel.vue'
import MetaAnalysisModal from '../components/MetaAnalysisModal.vue'
import HeatmapMetaModal from '../components/HeatmapMetaModal.vue'
import VolatilityMetaModal from '../components/VolatilityMetaModal.vue'
import ExportModal from '../components/ExportModal.vue'
import type { Archive as ArchiveType } from '../stores/archiveStore'

const archiveStore = useArchiveStore()
const selectedArchive = ref<Archive | null>(null)
const showViewer = ref(false)
const showMetaAnalysisModal = ref(false)
const showHeatmapMetaModal = ref(false)
const showVolatilityMetaModal = ref(false)
const showExportModal = ref(false)
const viewerData = ref<any>(null)
const showDeleteConfirmModal = ref(false)
const showDeleteAllConfirmModal = ref(false)
const archiveToDelete = ref<Archive | null>(null)
const selectedPair = ref<string>('all')
const expandedTypes = ref<Set<string>>(new Set())

onMounted(async () => {
  await archiveStore.loadArchives()
  // Expand the first section by default if archives exist
  if (archiveStore.archives.length > 0) {
    const firstType = archiveStore.archives[0].archive_type
    expandedTypes.value.add(firstType)
  }
})

async function confirmDeleteAll() {
  try {
    await archiveStore.supprimerToutesArchives()
    showDeleteAllConfirmModal.value = false
  } catch (e) {
    // Silent error
  }
}

// Computed property pour grouper et filtrer les archives par type
const archivesByType = computed(() => {
  let filtered = archiveStore.archives
  
  // Filtrer selon la paire sélectionnée
  if (selectedPair.value !== 'all') {
    filtered = filtered.filter(archive => {
      try {
        const data = JSON.parse(archive.data_json)
        // Chercher la paire dans les données
        const archivePair = data.pair || data.analysisResult?.symbol || null
        return archivePair === selectedPair.value
      } catch {
        return false
      }
    })
  }
  
  // Grouper par type
  const grouped: Record<string, Archive[]> = {}
  filtered.forEach(archive => {
    const type = archive.archive_type
    if (!grouped[type]) {
      grouped[type] = []
    }
    grouped[type].push(archive)
  })
  
  return grouped
})

// Basculer l'expansion/réduction d'une section de type
function basculerExpansionType(type: string) {
  if (expandedTypes.value.has(type)) {
    expandedTypes.value.delete(type)
  } else {
    expandedTypes.value.add(type)
  }
}

function getMetaAnalysisType(type: string): 'retro' | 'heatmap' | 'volatility' | null {
  if (type === 'RETRO_ANALYSIS' || type === 'Métriques Rétrospectives' || type === 'Correlation de la volatilité Paire/Evenement') {
    return 'retro'
  }
  if (type === 'Heatmap' || type === 'HEATMAP') {
    return 'heatmap'
  }
  if (type === 'Volatilité brute' || type === 'Volatilité brute Paire/Période' || type === 'METRICS') {
    return 'volatility'
  }
  return null
}

function openMetaAnalysis(type: string) {
  const metaType = getMetaAnalysisType(type)
  if (metaType === 'retro') {
    showMetaAnalysisModal.value = true
  } else if (metaType === 'heatmap') {
    showHeatmapMetaModal.value = true
  } else if (metaType === 'volatility') {
    showVolatilityMetaModal.value = true
  }
}

// Computed property pour récupérer les paires uniques disponibles
const availablePairs = computed(() => {
  const pairs = new Set<string>()
  archiveStore.archives.forEach(archive => {
    try {
      const data = JSON.parse(archive.data_json)
      // Chercher la paire dans les données de l'archive
      if (data.pair) {
        pairs.add(data.pair)
      }
      // Pour les archives Volatilité brute, chercher dans analysisResult
      if (data.analysisResult?.symbol) {
        pairs.add(data.analysisResult.symbol)
      }
      // Pour les heatmaps et corrélations
      if (data.pairMetrics) {
        Object.keys(data.pairMetrics).forEach(pair => pairs.add(pair))
      }
    } catch {
      // Ignorer les erreurs de parsing
    }
  })
  return Array.from(pairs).sort()
})

function obtenirClasseType(type: string): string {
  const mapping: Record<string, string> = {
    'Backtest': 'type-backtest',
    'Volatilité brute': 'type-metrics',
    'Volatilité brute Paire/Période': 'type-metrics',
    'Métriques Rétrospectives': 'type-default',
    'Correlation de la volatilité Paire/Evenement': 'type-default',
    'Corrélation événement/paire': 'type-event',
    'Corrélation paire/événement': 'type-pair',
    'Heatmap': 'type-heatmap',
    // Anciens types pour rétrocompatibilité
    'METRICS': 'type-metrics',
    'RETRO_ANALYSIS': 'type-default',
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

function extractEventLabel(archive: Archive): string {
  try {
    const data = JSON.parse(archive.data_json)
    // Chercher eventLabel en priorité, puis eventType, puis pair (paire)
    const label = data.eventLabel || data.eventType || data.pair || ''
    return label.trim() || 'Événement inconnu'
  } catch (e) {
    return 'Événement inconnu'
  }
}

function getBacktestSummary(archive: Archive): any | null {
  if (archive.archive_type !== 'Backtest') return null
  try {
    const data = JSON.parse(archive.data_json)
    return data.result || null
  } catch {
    return null
  }
}

function formatCurrency(value: number): string {
  return new Intl.NumberFormat('fr-FR', { style: 'currency', currency: 'USD' }).format(value)
}

function formatPercent(value: number): string {
  return new Intl.NumberFormat('fr-FR', { style: 'percent', minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(value / 100)
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
  gap: 20px;
}

.header-title-section {
  display: flex;
  align-items: flex-end;
  gap: 30px;
  flex: 1;
}

.header-pair-filter {
  display: flex;
  align-items: center;
  gap: 8px;
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

.pair-select {
  padding: 10px 14px;
  border: 2px solid #30363d;
  background: #161b22;
  color: #000000;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
  font-size: 0.95em;
  min-width: 150px;
}

.pair-select:hover {
  border-color: #58a6ff;
  background: #1a1f2e;
}

.pair-select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.1);
}

.pair-select option {
  background: #161b22;
  color: #000000;
}

.archives-container-accordion {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.archive-section {
  border: 1px solid #30363d;
  border-radius: 12px;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: linear-gradient(135deg, #161b22 0%, #1a2332 100%);
  cursor: pointer;
  transition: all 0.2s;
  user-select: none;
}

.section-header:hover {
  background: linear-gradient(135deg, #1a2332 0%, #212d3d 100%);
  border-color: #58a6ff;
}

.section-header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-meta-analysis-small {
  background: linear-gradient(135deg, #9c27b0 0%, #673ab7 100%);
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.85rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 8px rgba(156, 39, 176, 0.3);
}

.btn-meta-analysis-small:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(156, 39, 176, 0.4);
}

.section-toggle {
  color: #58a6ff;
  font-size: 1.1em;
  transition: transform 0.2s;
}

.section-count {
  margin-left: auto;
  color: #8b949e;
  font-size: 0.9em;
}

.archives-grid-compact {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 15px;
  padding: 15px;
  background: #0d1117;
}

.archive-card-compact {
  background: #161b22;
  border: 1px solid #30363d;
  border-radius: 10px;
  padding: 12px;
  transition: all 0.3s;
  height: fit-content;
}

.archive-card-compact:hover {
  border-color: #58a6ff;
  box-shadow: 0 8px 24px rgba(88, 166, 255, 0.2);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 8px;
  margin-bottom: 10px;
}

.card-title {
  color: #e2e8f0;
  font-size: 0.85em;
  font-weight: 600;
  margin: 0;
  line-height: 1.3;
  flex: 1;
  word-break: break-word;
}

.card-meta-compact {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 10px;
  font-size: 0.8em;
}

.meta-row {
  display: flex;
  gap: 6px;
  align-items: flex-start;
}

.meta-row .meta-label {
  color: #8b949e;
  min-width: 18px;
}

.meta-row .meta-value {
  color: #cbd5e0;
  font-size: 0.85em;
  flex: 1;
}

.card-actions-compact {
  display: flex;
  gap: 6px;
}

.btn-action-compact {
  flex: 1;
  padding: 6px 8px;
  border-radius: 6px;
  border: none;
  font-weight: 600;
  font-size: 0.75em;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-action-compact.btn-view {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-action-compact.btn-view:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.4);
}

.collapse-enter-active, .collapse-leave-active {
  transition: all 0.3s ease;
}

.collapse-enter-from {
  opacity: 0;
  height: 0;
}

.collapse-leave-to {
  opacity: 0;
  height: 0;
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

.type-backtest {
  background: #8b5cf6;
  color: white;
}

.backtest-summary {
  margin-top: 8px;
  background: #0d1117;
  padding: 8px;
  border-radius: 6px;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
  font-size: 0.75em;
  border: 1px solid #30363d;
}

.summary-item {
  display: flex;
  flex-direction: column;
}

.summary-label {
  color: #8b949e;
  font-size: 0.9em;
}

.summary-value {
  color: #e2e8f0;
  font-weight: 600;
}

.summary-value.positive {
  color: #238636;
}

.summary-value.negative {
  color: #da3633;
}

.type-default {
  background: #16a34a;
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
  width: 98vw;
  max-width: none;
  height: 95vh;
  max-height: none;
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
  overflow-x: hidden;
  flex: 1;
  width: 100%;
}

.unsupported-type {
  text-align: center;
  padding: 40px;
  color: #8b949e;
  font-size: 1.2em;
}

.btn-meta-analysis {
  background: linear-gradient(135deg, #9c27b0 0%, #673ab7 100%);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s;
  box-shadow: 0 4px 15px rgba(156, 39, 176, 0.3);
}

.btn-meta-analysis:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(156, 39, 176, 0.4);
}

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

.btn-delete-all {
  background: #2a2a3e;
  border: 1px solid #f85149;
  color: #f85149;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.1em;
  margin-left: auto;
}

.btn-delete-all:hover {
  background: #f85149;
  color: white;
  transform: scale(1.05);
}

.delete-all-modal {
  border-color: #ff0000;
  box-shadow: 0 0 50px rgba(255, 0, 0, 0.2);
}

.big-warning {
  font-size: 1.2em;
  color: #ff4444 !important;
  text-align: center;
  margin-bottom: 15px !important;
}

.warning-detail {
  color: #cbd5e0;
  text-align: center;
  margin: 0;
}

.btn-danger-zone {
  background: linear-gradient(135deg, #ff0000 0%, #cc0000 100%);
  border: 2px solid #ff0000;
  font-size: 1.1em;
  padding: 12px 30px;
  animation: pulse-red 2s infinite;
}

@keyframes pulse-red {
  0% { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0.7); }
  70% { box-shadow: 0 0 0 10px rgba(255, 0, 0, 0); }
  100% { box-shadow: 0 0 0 0 rgba(255, 0, 0, 0); }
}

/* Responsive styles for accordion and compact cards */
@media (max-width: 1200px) {
  .archives-grid-compact {
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  }
}

@media (max-width: 768px) {
  .header-title-section {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .header-pair-filter {
    flex-wrap: wrap;
    gap: 6px;
  }

  .filter-label {
    font-size: 0.85em;
  }

  .pair-select {
    padding: 8px 10px;
    font-size: 0.85em;
    min-width: 120px;
  }

  .archives-grid-compact {
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 10px;
    padding: 10px;
  }

  .archive-card-compact {
    padding: 10px;
  }

  .card-title {
    font-size: 0.8em;
  }

  .card-meta-compact {
    gap: 4px;
  }

  .btn-action-compact {
    padding: 4px 6px;
    font-size: 0.7em;
  }
}

@media (max-width: 480px) {
  .filters-bar {
    flex-direction: column;
    gap: 6px;
  }

  .filter-btn {
    width: 100%;
    text-align: center;
  }

  .archives-grid-compact {
    grid-template-columns: 1fr;
  }

  .section-header {
    padding: 12px 16px;
  }

  .archive-card-compact {
    padding: 8px;
  }

  .card-actions-compact {
    gap: 4px;
  }
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
  .archives-grid,
  .archives-container-accordion,
  .filters-bar {
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
