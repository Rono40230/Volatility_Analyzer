import { cleanupApi } from '../services/cleanupApi'
import { formatEventLabel } from '../utils/cleanupFormatter'
import type { CleanupState } from './useCleanupState'

export function useCleanupPreview(
  state: CleanupState,
  props: { calendarId: number | null }
) {
  const { previewMode, loadingPreview, previewTitle, previewEvents } = state

  async function loadPreview(filterType: string, filterValue: string, title: string) {
    previewMode.value = true
    loadingPreview.value = true
    previewTitle.value = title
    previewEvents.value = []
    
    try {
      const rawEvents = await cleanupApi.previewCleanupEvents(filterType, filterValue, props.calendarId)
      previewEvents.value = rawEvents.map(e => ({
        ...e,
        label: formatEventLabel(e.description)
      }))
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

  return {
    loadPreview,
    closePreview
  }
}
