export interface RareEventSummary { description: string; count: number; label?: string }
export interface CurrencySummary { symbol: string; country_name: string; count: number }
export interface OrphanEventSummary { reason: string; count: number }
export interface ImpactGroupSummary { description: string; impact: string; count: number; label?: string }
export type { CalendarEvent } from '../stores/volatilityTypes'
