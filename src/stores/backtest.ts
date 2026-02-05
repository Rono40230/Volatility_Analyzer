import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface BacktestConfig {
  offset_pips: number
  stop_loss_pips: number
  timeout_minutes: number
  sl_recovery_pips: number | null
  spread_pips: number
  slippage_pips: number // Nouveau champ
  point_value: number
}


export enum BacktestType {
  Event = 'Event',
  Time = 'Time'
}

export interface TradeResult {
  event_date: string
  entry_time: string
  exit_time: string
  duration_minutes: number
  pips_net: number
  outcome: string
  max_favorable_excursion: number
  max_adverse_excursion: number
  logs: string[]
}

export interface BacktestResult {
  symbol: string
  event_name: string
  unit: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  no_entries: number
  win_rate_percent: number
  total_pips: number
  average_pips_per_trade: number
  max_drawdown_pips: number
  profit_factor: number
  trades: TradeResult[]
}

export const useBacktestStore = defineStore('backtest', () => {
  const config = ref<BacktestConfig>({
    offset_pips: 5.0,
    stop_loss_pips: 10.0,
    timeout_minutes: 60,
    sl_recovery_pips: null,
    spread_pips: 1.0,
    slippage_pips: 0.5, // Valeur par défaut conservatrice
    point_value: 0.0001 // Default for major pairs
  })

  const result = ref<BacktestResult | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  
  // Store context for export
  const currentSymbol = ref<string>('')
  const currentEvent = ref<string>('')

  async function mettreAJourProprietesSymbole(symbol: string) {
    if (!symbol) return
    try {
      const props = await invoke<{ point_value: number, pip_value: number }>('get_symbol_properties', { symbol })
      config.value.point_value = props.point_value
    } catch (e) {
      // Silent fail or handle error appropriately
    }
  }

  async function runBacktest(pair: string, eventType: string) {
    loading.value = true
    error.value = null
    result.value = null
    currentSymbol.value = pair
    currentEvent.value = eventType
    
    try {
      result.value = await invoke('run_backtest', {
        pair,
        eventType,
        config: config.value
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function runBacktestTime(pair: string, time: string, startDate: string, endDate: string) {
    loading.value = true
    error.value = null
    result.value = null
    currentSymbol.value = pair
    currentEvent.value = `Horaire ${time}`
    
    try {
      result.value = await invoke('run_backtest_time', {
        pair,
        time,
        startDate,
        endDate,
        config: config.value
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  return {
    config,
    result,
    loading,
    error,
    currentSymbol,
    currentEvent,
    runBacktest,
    runBacktestTime,
    mettreAJourProprietesSymbole,
    // Alias pour compatibilité
    updateSymbolProperties: mettreAJourProprietesSymbole
  }
})
