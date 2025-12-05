// composable pour gérer l'archivage de la heatmap
import { ref, Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

interface HeatmapComponentInstance {
  getHeatmapArchiveData: () => {
    heatmapData: {
      pairs: string[]
      event_types: Array<{ name: string; has_data?: boolean }>
      data: Record<string, Record<string, number>>
    }
    minVolatilityThreshold: number
    maxEventsToDisplay: number
    selectedEventType: string
  }
}

export function useHeatmapArchive() {
  const showArchiveModal = ref(false)
  const archiveDataJson = ref('')
  const archivePeriodStart = ref('')
  const archivePeriodEnd = ref('')

  async function openArchiveModal(
    heatmapComponentRef: Ref<HeatmapComponentInstance | undefined>,
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
        }>('get_calendar_period_by_id', { calendarId: selectedCalendarId })

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
