import { invoke } from '@tauri-apps/api/core'
import type { RareEventSummary, CurrencySummary, OrphanEventSummary, ImpactGroupSummary, CalendarEvent } from '../types/cleanup'

export const cleanupApi = {
  async listRareEvents(minOccurrences: number, calendarId: number | null): Promise<RareEventSummary[]> {
    return invoke('list_rare_events', { minOccurrences, calendarId })
  },

  async listCurrencies(calendarId: number | null): Promise<CurrencySummary[]> {
    return invoke('list_currencies', { calendarId })
  },

  async listOrphanEvents(calendarId: number | null): Promise<OrphanEventSummary[]> {
    return invoke('list_orphan_events', { calendarId })
  },

  async listImpactGroups(calendarId: number | null): Promise<ImpactGroupSummary[]> {
    return invoke('list_impact_groups', { calendarId })
  },

  async updateImpact(description: string, newImpact: string, calendarId: number | null): Promise<void> {
    return invoke('update_impact_for_description', { description, newImpact, calendarId })
  },

  async deleteEventsByImpact(impact: string, calendarId: number | null): Promise<void> {
    return invoke('delete_events_by_impact', { impact, calendarId })
  },

  async previewCleanupEvents(filterType: string, filterValue: string, calendarId: number | null): Promise<CalendarEvent[]> {
    return invoke('preview_cleanup_events', { filterType, filterValue, calendarId })
  }
}
