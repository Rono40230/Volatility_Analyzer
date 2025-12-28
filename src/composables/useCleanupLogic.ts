import { watch } from 'vue'
import { cleanupApi } from '../services/cleanupApi'
import { useCleanupState } from './useCleanupState'
import { formatEventLabel } from '../utils/cleanupFormatter'
import { useCleanupPreview } from './useCleanupPreview'

export function useCleanupLogic(props: { minOccurrences: number; calendarId: number | null }) {
  const state = useCleanupState(props.minOccurrences)
  const {
    activeTab, showConfirmation, confirmationMessage, deleteAction,
    previewMode, previewEvents, previewTitle, loadingPreview,
    events, loading, threshold,
    countries, loadingCountries,
    orphans, loadingOrphans, totalOrphans,
    impacts, loadingImpacts
  } = state

  const { loadPreview, closePreview } = useCleanupPreview(state, props)

  // --- LOADERS ---

  async function loadEvents() {
    loading.value = true
    try {
      const rawEvents = await cleanupApi.listRareEvents(threshold.value, props.calendarId)
      events.value = rawEvents.map(e => ({
        ...e,
        label: formatEventLabel(e.description)
      }))
    } catch (e) {
      // Silent error
    } finally {
      loading.value = false
    }
  }

  async function loadCountries() {
    loadingCountries.value = true
    try {
      countries.value = await cleanupApi.listCurrencies(props.calendarId)
    } catch (e) {
      // Silent error
    } finally {
      loadingCountries.value = false
    }
  }

  async function loadOrphans() {
    loadingOrphans.value = true
    try {
      orphans.value = await cleanupApi.listOrphanEvents(props.calendarId)
    } catch (e) {
      // Silent error
    } finally {
      loadingOrphans.value = false
    }
  }

  async function loadImpacts() {
    loadingImpacts.value = true
    try {
      const rawImpacts = await cleanupApi.listImpactGroups(props.calendarId)
      impacts.value = rawImpacts.map(i => ({
        ...i,
        label: formatEventLabel(i.description)
      }))
    } catch (e) {
      // Silent error
    } finally {
      loadingImpacts.value = false
    }
  }

  async function updateImpact(description: string, newImpact: string) {
    try {
      await cleanupApi.updateImpact(description, newImpact, props.calendarId)
      await loadImpacts()
    } catch (e) {
      // Silent error
    }
  }

  async function deleteEventsByImpact(impactsList: string[]) {
    try {
      for (const impact of impactsList) {
        await cleanupApi.deleteEventsByImpact(impact, props.calendarId)
      }
      await loadImpacts()
    } catch (e) {
      // Silent error
    }
  }

  // Watch tab changes to load data
  watch(activeTab, (newTab) => {
    if (newTab === 'rare') loadEvents()
    if (newTab === 'country') loadCountries()
    if (newTab === 'orphan') loadOrphans()
    if (newTab === 'impact') loadImpacts()
  })

  // Watch threshold changes to reload rare events
  watch(threshold, () => {
    if (activeTab.value === 'rare') {
      loadEvents()
    }
  })

  return {
    activeTab,
    showConfirmation,
    confirmationMessage,
    deleteAction,
    previewMode,
    previewEvents,
    previewTitle,
    loadingPreview,
    events,
    loading,
    threshold,
    countries,
    loadingCountries,
    orphans,
    loadingOrphans,
    totalOrphans,
    impacts,
    loadingImpacts,
    loadEvents,
    loadCountries,
    loadOrphans,
    loadImpacts,
    updateImpact,
    deleteEventsByImpact,
    loadPreview,
    closePreview
  }
}
