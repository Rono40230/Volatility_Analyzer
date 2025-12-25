import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface RareEventSummary { description: string; count: number }
export interface CurrencySummary { symbol: string; country_name: string; count: number }
export interface OrphanEventSummary { reason: string; count: number }
export interface CalendarEvent {
  id: number
  symbol: string
  event_time: string
  impact: string
  description: string
  actual: number | null
  forecast: number | null
  previous: number | null
}

export function useCleanupLogic(props: { minOccurrences: number; calendarId: number | null }) {
  // State
  const activeTab = ref<'rare' | 'country' | 'orphan'>('rare')
  const showConfirmation = ref(false)
  const confirmationMessage = ref('')
  const deleteAction = ref<() => Promise<void>>(async () => {})

  // Preview State
  const previewMode = ref(false)
  const previewEvents = ref<CalendarEvent[]>([])
  const previewTitle = ref('')
  const loadingPreview = ref(false)

  // Rare Events State
  const events = ref<RareEventSummary[]>([])
  const loading = ref(true)
  const threshold = ref(props.minOccurrences)

  // Country State
  const countries = ref<CurrencySummary[]>([])
  const loadingCountries = ref(false)

  // Orphan State
  const orphans = ref<OrphanEventSummary[]>([])
  const loadingOrphans = ref(false)
  const totalOrphans = computed(() => orphans.value.reduce((acc, o) => acc + o.count, 0))

  // --- LOADERS ---

  async function loadEvents() {
    loading.value = true
    try {
      events.value = await invoke('list_rare_events', { 
        minOccurrences: threshold.value,
        calendarId: props.calendarId 
      })
    } catch (e) {
      // Silent error
    } finally {
      loading.value = false
    }
  }

  async function loadCountries() {
    loadingCountries.value = true
    try {
      countries.value = await invoke('list_currencies', {
        calendarId: props.calendarId
      })
    } catch (e) {
      // Silent error
    } finally {
      loadingCountries.value = false
    }
  }

  async function loadOrphans() {
    loadingOrphans.value = true
    try {
      orphans.value = await invoke('list_orphan_events', {
        calendarId: props.calendarId
      })
    } catch (e) {
      // Silent error
    } finally {
      loadingOrphans.value = false
    }
  }

  async function loadPreview(filterType: string, filterValue: string, title: string) {
    previewMode.value = true
    loadingPreview.value = true
    previewTitle.value = title
    previewEvents.value = []
    
    try {
      previewEvents.value = await invoke('preview_cleanup_events', {
        filterType,
        filterValue,
        calendarId: props.calendarId
      })
    } catch (e) {
      // Silent error
    } finally {
      loadingPreview.value = false
    }
  }

  function closePreview() {
    previewMode.value = false
    previewEvents.value = []
  }

  // Watch tab changes to load data
  watch(activeTab, (newTab) => {
    if (newTab === 'rare') loadEvents()
    if (newTab === 'country') loadCountries()
    if (newTab === 'orphan') loadOrphans()
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
    loadEvents,
    loadCountries,
    loadOrphans,
    loadPreview,
    closePreview
  }
}
