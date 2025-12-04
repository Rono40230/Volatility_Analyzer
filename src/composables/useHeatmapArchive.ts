// composable pour gérer l'archivage de la heatmap
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export function useHeatmapArchive() {
  const showArchiveModal = ref(false)
  const archiveDataJson = ref('')
  const archivePeriodStart = ref('')
  const archivePeriodEnd = ref('')

  async function openArchiveModal(
    heatmapComponentRef: any,
    selectedCalendarId: number | null
  ) {
    if (!heatmapComponentRef?.value) return

    const archiveData = heatmapComponentRef.value.getHeatmapArchiveData()

    archiveDataJson.value = JSON.stringify({
      heatmapData: archiveData.heatmapData,
      minVolatilityThreshold: archiveData.minVolatilityThreshold,
      maxEventsToDisplay: archiveData.maxEventsToDisplay,
      selectedEventType: archiveData.selectedEventType,
    })

    if (selectedCalendarId) {
      try {
        const period = await invoke<{
          start_date: string | null
          end_date: string | null
        }>('get_calendar_period_by_id', { calendar_id: selectedCalendarId })

        archivePeriodStart.value = period.start_date || ''
        archivePeriodEnd.value = period.end_date || ''
      } catch {
        setDefaultDates()
      }
    } else {
      setDefaultDates()
    }

    showArchiveModal.value = true
  }

  function setDefaultDates() {
    const today = new Date()
    archivePeriodEnd.value = today.toISOString().split('T')[0]
    const thirtyDaysAgo = new Date(today.getTime() - 30 * 24 * 60 * 60 * 1000)
    archivePeriodStart.value = thirtyDaysAgo.toISOString().split('T')[0]
  }

  return {
    showArchiveModal,
    archiveDataJson,
    archivePeriodStart,
    archivePeriodEnd,
    openArchiveModal,
  }
}
