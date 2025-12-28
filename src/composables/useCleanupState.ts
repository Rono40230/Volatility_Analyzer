import { ref, computed } from 'vue'
import type { RareEventSummary, CurrencySummary, OrphanEventSummary, ImpactGroupSummary, CalendarEvent } from '../types/cleanup'

export function useCleanupState(minOccurrences: number) {
  const activeTab = ref<'rare' | 'country' | 'orphan' | 'impact'>('rare')
  const showConfirmation = ref(false)
  const confirmationMessage = ref('')
  const deleteAction = ref<() => Promise<void>>(async () => {})

  const previewMode = ref(false)
  const previewEvents = ref<CalendarEvent[]>([])
  const previewTitle = ref('')
  const loadingPreview = ref(false)

  const events = ref<RareEventSummary[]>([])
  const loading = ref(true)
  const threshold = ref(minOccurrences)

  const countries = ref<CurrencySummary[]>([])
  const loadingCountries = ref(false)

  const orphans = ref<OrphanEventSummary[]>([])
  const loadingOrphans = ref(false)
  const totalOrphans = computed(() => orphans.value.reduce((acc, o) => acc + o.count, 0))

  const impacts = ref<ImpactGroupSummary[]>([])
  const loadingImpacts = ref(false)

  return {
    activeTab, showConfirmation, confirmationMessage, deleteAction,
    previewMode, previewEvents, previewTitle, loadingPreview,
    events, loading, threshold,
    countries, loadingCountries,
    orphans, loadingOrphans, totalOrphans,
    impacts, loadingImpacts
  }
}
