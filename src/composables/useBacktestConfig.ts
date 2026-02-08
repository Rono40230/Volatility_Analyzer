import { ref, watch, onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useBacktestStore, BacktestType } from '../stores/backtest'
import { useVolatilityStore } from '../stores/volatility'
import { invoke } from '@tauri-apps/api/core'
import { eventTranslations } from '../utils/eventTranslations'

export function useBacktestConfig(props: { backtestType: BacktestType }) {
  const backtestStore = useBacktestStore()
  const volatilityStore = useVolatilityStore()
  const { config, loading } = storeToRefs(backtestStore)
  const { symbols } = storeToRefs(volatilityStore)

  const selectedSymbol = ref('')
  const selectedEvent = ref('')
  const selectedTime = ref('15:30')
  const startDate = ref('2020-01-01')
  const endDate = ref(new Date().toISOString().split('T')[0])

  const availableEvents = ref<{name: string, label: string, count: number}[]>([])

  const configMode = ref<'manual' | 'auto'>('manual')

  // Helper pour les coûts par défaut (Miroir de la logique Rust)
  function getDefaultCosts(symbol: string) {
    const s = symbol.toUpperCase()
    if (s.includes('JPY') && (s.includes('GBP') || s.includes('EUR'))) {
      return { spread: 6.0, slippage: 3.0 } // Crosses volatils
    } else if (s.includes('GBP')) {
      return { spread: 4.0, slippage: 2.0 } // Majors volatiles
    } else if (s.includes('XAU') || s.includes('GOLD')) {
      return { spread: 5.0, slippage: 2.0 } // Or
    } else if (s.includes('BTC')) {
      return { spread: 50.0, slippage: 20.0 } // Crypto
    } else if (s.includes('DAX') || s.includes('GER40')) {
      return { spread: 6.0, slippage: 3.0 } // DAX
    } else if (s.includes('US30') || s.includes('DJI')) {
      return { spread: 8.0, slippage: 5.0 } // Dow Jones
    } else {
      return { spread: 2.5, slippage: 1.0 } // Majors liquides (EURUSD)
    }
  }

  // Watcher pour mettre à jour la valeur du point et les coûts quand le symbole change
  watch(selectedSymbol, async (newSymbol) => {
    configMode.value = 'manual' // Reset du mode auto au changement de symbole
    if (newSymbol) {
      await backtestStore.mettreAJourProprietesSymbole(newSymbol)
      
      // Mise à jour automatique des coûts recommandés
      const costs = getDefaultCosts(newSymbol)
      config.value.spread_pips = costs.spread
      config.value.slippage_pips = costs.slippage
    }
  })

  onMounted(async () => {
    // Charger les dates du calendrier actif
    try {
      const activeCalId = localStorage.getItem('activeCalendarId')
      if (activeCalId) {
        const calendars = await invoke<{ id: number; start_date?: string; end_date?: string }[]>('get_calendars_metadata')
        const active = calendars?.find(c => c.id === parseInt(activeCalId, 10))
        if (active?.start_date) startDate.value = active.start_date.substring(0, 10)
        if (active?.end_date) endDate.value = active.end_date.substring(0, 10)
      }
    } catch (_e) {
      // Fallback: garder les valeurs par défaut
    }

    // Charger les types d'événements disponibles
    try {
      const response = await invoke<{ types: { name: string, count: number }[] }>('get_event_types')
      availableEvents.value = response.types.map(e => {
        const translation = eventTranslations[e.name]
        const label = translation 
          ? `${e.name} (${translation.fr}) ${translation.flag}` 
          : e.name
        return { name: e.name, label, count: e.count }
      })
    } catch (e) {
      // Fallback mock data if command not ready
      availableEvents.value = [
        { name: 'Non-Farm Employment Change', label: 'Non-Farm Employment Change', count: 0 },
        { name: 'CPI', label: 'CPI', count: 0 },
        { name: 'Interest Rate Decision', label: 'Interest Rate Decision', count: 0 }
      ]
    }
  })

  async function appliquerParamsAuto() {
    if (!selectedSymbol.value) return
    try {
      const recommended = await invoke<{
        stop_loss_pips: number
        tp_rr: number
        trailing_atr_coef: number
        atr_period: number
        trailing_refresh_seconds: number
        timeout_minutes: number
        sl_recovery_pips: number | null
        spread_pips: number
        slippage_pips: number
        point_value: number
      }>('get_recommended_backtest_config', { symbol: selectedSymbol.value })
      config.value = { ...recommended }
      configMode.value = 'auto'
    } catch (_e) {
      configMode.value = 'manual'
    }
  }

  async function lancerBacktest() {
    if (!selectedSymbol.value) return

    if (props.backtestType === BacktestType.Event) {
      if (!selectedEvent.value) return
      await backtestStore.runBacktest(selectedSymbol.value, selectedEvent.value)
    } else {
      if (!selectedTime.value || !startDate.value || !endDate.value) return
      await backtestStore.runBacktestTime(
        selectedSymbol.value, 
        selectedTime.value,
        startDate.value,
        endDate.value
      )
    }
  }

  return {
    config,
    configMode,
    loading,
    symbols,
    selectedSymbol,
    selectedEvent,
    selectedTime,
    startDate,
    endDate,
    availableEvents,
    lancerBacktest,
    appliquerParamsAuto,
  }
}
