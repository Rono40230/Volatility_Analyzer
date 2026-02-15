import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ConversionEntry {
  symbol: string
  pip_value: number
  unit: string
  display_digits: number
  is_custom: boolean
  hidden?: boolean
}

export const useConversionStore = defineStore('conversions', () => {
  const conversions = ref<ConversionEntry[]>([])
  const loading = ref(false)
  const updateSignal = ref(0) // Signal pour forcer les rafraîchissements

  async function loadConversions() {
    loading.value = true
    try {
      conversions.value = await invoke<ConversionEntry[]>('get_all_conversions')
      // Incrémenter le signal pour notifier tous les listeners
      updateSignal.value++
    } catch (e) {
      console.error('Failed to load conversions:', e)
    } finally {
      loading.value = false
    }
  }

  function getUnitForSymbol(symbol: string): string {
    const s = symbol.toUpperCase()
    // 1. Chercher dans les réglages (BD ou hardcodés enrichis)
    const conv = conversions.value.find(c => s.includes(c.symbol.toUpperCase()) || c.symbol.toUpperCase().includes(s))
    if (conv) return conv.unit === 'points' ? 'pts' : conv.unit

    // 2. Heuristique par défaut si pas trouvé
    const DOLLAR_ASSETS = ['XAU', 'GOLD', 'XAG', 'SILVER', 'BTC', 'ETH', 'SOL', 'BNB', 'XRP', 'ADA', 'DOT', 'LTC', 'BCH', 'DOGE', 'SHIB', 'LINK', 'MATIC', 'AVAX', 'UNI', 'XLM', 'TRX', 'ATOM', 'NEAR', 'PEPE', 'OIL', 'WTI', 'BRENT', 'CRUDE', 'XPT', 'XPD', 'NGAS']
    const POINT_ASSETS = ['IDX', 'US30', 'DAX', 'NAS', 'GER', 'SPX', 'US100', 'US500', 'FRA40', 'UK100', 'EUSTX', 'JPN225', 'USATEC', 'USAIDX', 'DEUIDX', 'USTEC', 'HK50', 'FR40', 'GR30', 'DE40', 'WS30', 'NDX', 'VIX', 'DXY', 'STOXX', 'CAC', 'FTSE', 'NI225', 'ASX', 'HSI']

    if (DOLLAR_ASSETS.some(k => s.includes(k))) return '$'
    if (POINT_ASSETS.some(k => s.includes(k))) return 'pts'
    return 'pips'
  }

  function pipsToDisplayValue(pipsValue: number, symbol: string): number {
    const s = symbol.toUpperCase()
    const conv = conversions.value.find(c => s.includes(c.symbol.toUpperCase()) || c.symbol.toUpperCase().includes(s))
    
    if (conv) {
      if (conv.unit === 'pips') return pipsValue
      return pipsValue * conv.pip_value
    }

    // Heuristique par défaut
    if (getUnitForSymbol(symbol) === 'pips') return pipsValue
    return pipsValue // Fallback 1:1 si on ne connaît pas la pip_value
  }

  function isSymbolHidden(symbol: string): boolean {
    const s = symbol.toUpperCase()
    const conv = conversions.value.find(c => s.includes(c.symbol.toUpperCase()) || c.symbol.toUpperCase().includes(s))
    return conv?.hidden ?? false
  }

  return {
    conversions,
    loading,
    updateSignal,
    loadConversions,
    getUnitForSymbol,
    pipsToDisplayValue,
    isSymbolHidden
  }
})
