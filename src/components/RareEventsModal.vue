<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal-container">
      <div class="modal-header">
        <div class="header-content">
          <div class="icon-wrapper">🧹</div>
          <div class="title-section">
            <h3>Nettoyage de la base de données</h3>
            <p class="subtitle">Optimisez votre base de données en supprimant les données inutiles</p>
          </div>
        </div>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </div>

      <div class="tabs-header">
        <button v-for="tab in (['rare', 'country', 'impact'] as const)" :key="tab"
          class="tab-btn" :class="{ active: activeTab === tab }" 
          @click="activeTab = tab"
        >
          {{ tab === 'rare' ? 'Par occurences' : tab === 'country' ? 'Par pays' : 'Par impact' }}
        </button>
      </div>

      <div class="modal-body">
        <CleanupPreview v-if="previewMode"
          :events="previewEvents" :title="previewTitle" :loading="loadingPreview"
          :all-countries="countries"
          @close="closePreview" @refresh="loadEvents"
        />

        <RareEventsTab v-else-if="activeTab === 'rare'"
          :events="events" :loading="loading" v-model:threshold="threshold"
          @preview="loadPreview"
        />

        <CountryEventsTab v-else-if="activeTab === 'country'"
          :countries="countries" :loading="loadingCountries"
          @preview="loadPreview" @delete="confirmDeleteCountry"
        />

        <ImpactEventsTab v-else-if="activeTab === 'impact'"
          :impacts="impacts" :loading="loadingImpacts"
          @preview="loadPreview" @update="updateImpact" @delete="confirmDeleteImpact"
        />
      </div>

      <div class="modal-footer">
        <button class="btn-secondary" @click="$emit('close')">Fermer</button>
        <button v-if="activeTab === 'rare' && events.length > 0 && !previewMode" 
          class="btn-primary delete-btn" @click="confirmDeleteRare"
        >
          🗑️ Supprimer {{ events.length }} types rares
        </button>
      </div>

      <div v-if="showConfirmation" class="confirmation-overlay">
        <div class="confirmation-box">
          <div class="warning-icon">⚠️</div>
          <h4>Confirmation requise</h4>
          <p>{{ confirmationMessage }}</p>
          <p class="warning-text">Cette action est irréversible.</p>
          <div class="confirmation-actions">
            <button class="btn-secondary" @click="showConfirmation = false">Annuler</button>
            <button class="btn-danger" @click="executeDelete">Confirmer la suppression</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import CleanupPreview from './cleanup/CleanupPreview.vue'
import RareEventsTab from './cleanup/RareEventsTab.vue'
import CountryEventsTab from './cleanup/CountryEventsTab.vue'
import ImpactEventsTab from './cleanup/ImpactEventsTab.vue'
import { useCleanupLogic, type CurrencySummary } from '../composables/useCleanupLogic'

const props = defineProps<{ minOccurrences: number; calendarId: number | null }>()
const emit = defineEmits(['close', 'deleted'])

const {
  activeTab, showConfirmation, confirmationMessage, deleteAction,
  previewMode, previewEvents, previewTitle, loadingPreview,
  events, loading, threshold,
  countries, loadingCountries,
  orphans, loadingOrphans, totalOrphans,
  impacts, loadingImpacts,
  loadEvents, loadCountries, loadOrphans, loadImpacts, loadPreview, closePreview,
  updateImpact, deleteEventsByImpact
} = useCleanupLogic(props)

onMounted(() => {
  loadEvents()
  loadCountries()
})

function confirmDeleteRare() {
  confirmationMessage.value = `Vous êtes sur le point de supprimer définitivement ${events.value.length} types d'événements rares.`
  deleteAction.value = async () => {
    await invoke('delete_rare_events', { minOccurrences: threshold.value, calendarId: props.calendarId })
    await loadEvents()
  }
  showConfirmation.value = true
}

function confirmDeleteCountry(c: CurrencySummary) {
  confirmationMessage.value = `Voulez-vous vraiment supprimer TOUS les événements pour ${c.country_name} (${c.symbol}) - ${c.count} événements ?`
  deleteAction.value = async () => {
    await invoke('delete_currency_events', { currencySymbol: c.symbol, calendarId: props.calendarId })
    await loadCountries()
  }
  showConfirmation.value = true
}

function confirmDeleteOrphans() {
  confirmationMessage.value = `Voulez-vous supprimer les ${totalOrphans.value} événements orphelins détectés ?`
  deleteAction.value = async () => {
    await invoke('delete_orphan_events', { calendarId: props.calendarId })
    await loadOrphans()
  }
  showConfirmation.value = true
}

function confirmDeleteImpact(rawImpacts: string[], count: number, label: string) {
  confirmationMessage.value = `Voulez-vous vraiment supprimer TOUS les événements avec l'impact "${label}" (${count} types d'événements) ?`
  deleteAction.value = async () => {
    await deleteEventsByImpact(rawImpacts)
  }
  showConfirmation.value = true
}

async function executeDelete() {
  try {
    await deleteAction.value()
    emit('deleted')
    showConfirmation.value = false
  } catch (e) {
    // Silent error
  }
}
</script>

<style scoped>
.modal-overlay { position: fixed; inset: 0; background: rgba(0,0,0,0.75); backdrop-filter: blur(4px); display: flex; justify-content: center; align-items: center; z-index: 1000; animation: fadeIn 0.2s ease-out; }
.modal-container { background: #1e293b; border-radius: 16px; width: 700px; max-width: 90vw; max-height: 85vh; display: flex; flex-direction: column; box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5); border: 1px solid #334155; position: relative; overflow: hidden; animation: slideUp 0.3s cubic-bezier(0.16, 1, 0.3, 1); }
.modal-header { padding: 20px 24px; background: linear-gradient(to right, #1e293b, #0f172a); border-bottom: 1px solid #334155; display: flex; justify-content: space-between; align-items: flex-start; }
.header-content { display: flex; gap: 16px; align-items: center; }
.icon-wrapper { width: 48px; height: 48px; background: rgba(245, 158, 11, 0.1); border-radius: 12px; display: flex; align-items: center; justify-content: center; font-size: 24px; border: 1px solid rgba(245, 158, 11, 0.2); }
.title-section h3 { margin: 0; color: #f8fafc; font-size: 1.25rem; font-weight: 600; }
.subtitle { margin: 4px 0 0; color: #94a3b8; font-size: 0.875rem; }
.close-btn { background: transparent; border: none; color: #64748b; font-size: 20px; cursor: pointer; padding: 4px; border-radius: 4px; transition: all 0.2s; }
.close-btn:hover { color: #f8fafc; background: rgba(255, 255, 255, 0.1); }
.tabs-header { display: flex; background: #0f172a; border-bottom: 1px solid #334155; padding: 0 24px; }
.tab-btn { padding: 16px 20px; background: transparent; border: none; color: #94a3b8; font-weight: 600; cursor: pointer; border-bottom: 2px solid transparent; transition: all 0.2s; }
.tab-btn:hover { color: #e2e8f0; }
.tab-btn.active { color: #f59e0b; border-bottom-color: #f59e0b; }
.modal-body { flex: 1; padding: 24px; overflow: hidden; display: flex; flex-direction: column; background: #1e293b; }
.modal-footer { padding: 16px 24px; border-top: 1px solid #334155; background: #1e293b; display: flex; justify-content: flex-end; gap: 12px; }
button { padding: 10px 20px; border-radius: 8px; font-weight: 600; font-size: 0.9rem; cursor: pointer; transition: all 0.2s; border: none; }
.btn-secondary { background: #334155; color: #e2e8f0; }
.btn-secondary:hover { background: #475569; }
.btn-primary { background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%); color: white; box-shadow: 0 4px 6px -1px rgba(245, 158, 11, 0.3); }
.btn-primary:hover { transform: translateY(-1px); box-shadow: 0 6px 8px -1px rgba(245, 158, 11, 0.4); }
.confirmation-overlay { position: absolute; inset: 0; background: rgba(15, 23, 42, 0.9); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 10; animation: fadeIn 0.2s; }
.confirmation-box { background: #1e293b; border: 1px solid #ef4444; padding: 32px; border-radius: 16px; text-align: center; max-width: 400px; box-shadow: 0 20px 25px -5px rgba(0,0,0,0.5); animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1); }
.warning-icon { font-size: 48px; margin-bottom: 16px; }
.confirmation-box h4 { color: #f8fafc; font-size: 1.25rem; margin: 0 0 12px; }
.confirmation-box p { color: #cbd5e1; margin: 0 0 8px; line-height: 1.5; }
.warning-text { color: #ef4444 !important; font-weight: 600; font-size: 0.9rem; }
.confirmation-actions { display: flex; gap: 12px; margin-top: 24px; justify-content: center; }
.btn-danger { background: #ef4444; color: white; }
.btn-danger:hover { background: #dc2626; }
@keyframes spin { to { transform: rotate(360deg); } }
@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes slideUp { from { opacity: 0; transform: translateY(20px); } to { opacity: 1; transform: translateY(0); } }
@keyframes scaleIn { from { opacity: 0; transform: scale(0.95); } to { opacity: 1; transform: scale(1); } }
</style>
