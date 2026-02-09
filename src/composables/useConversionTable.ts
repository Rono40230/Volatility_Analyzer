// composables/useConversionTable.ts - Logique table de conversion
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { setConversionCache } from '../utils/pipConverter'

interface ConversionEntry {
  symbol: string
  pip_value: number
  unit: string
  display_digits: number
  mt5_digits: number
  is_custom: boolean
}

interface EditingRow {
  symbol: string
  pip_value: string
  unit: string
  display_digits: string
  mt5_digits: string
}

export type { ConversionEntry }

export function useConversionTable() {
  const conversions = ref<ConversionEntry[]>([])
  const loading = ref(false)
  const saving = ref(false)
  const editingSymbol = ref<string | null>(null)
  const editRow = ref<EditingRow>({ symbol: '', pip_value: '', unit: '', display_digits: '', mt5_digits: '' })
  const newSymbol = ref('')
  const newPipValue = ref('')
  const newMt5Digits = ref('5')
  const showAddRow = ref(false)
  const errorMsg = ref('')

  /** Calcule le nombre de décimales depuis pip_value : -log10(v), arrondi */
  function computeDigits(pipStr: string): number {
    const v = parseFloat(pipStr)
    if (isNaN(v) || v <= 0 || v >= 1) return 0
    return Math.round(-Math.log10(v))
  }

  /** Génère un exemple concret : ratio = pip_value / point_size = pip_value * 10^mt5_digits */
  function computeExample(pipStr: string, mt5Str: string): string {
    const pv = parseFloat(pipStr)
    const mt5 = parseInt(mt5Str)
    if (isNaN(pv) || pv <= 0 || isNaN(mt5) || mt5 < 0) return '—'
    const ratio = Math.round(pv * Math.pow(10, mt5))
    if (ratio <= 1) return '1 pip = 1 pt'
    return `1 pip = ${ratio} pts`
  }

  onMounted(async () => { await loadConversions() })

  async function loadConversions() {
    loading.value = true
    try {
      conversions.value = await invoke<ConversionEntry[]>('get_all_conversions')
      // Mettre à jour le cache pipConverter pour que la heatmap et les analyses utilisent les bonnes valeurs
      setConversionCache(conversions.value)
    } catch (e) {
      errorMsg.value = `Erreur chargement: ${e}`
    } finally {
      loading.value = false
    }
  }

  function startEdit(entry: ConversionEntry) {
    editingSymbol.value = entry.symbol
    editRow.value = {
      symbol: entry.symbol,
      pip_value: entry.pip_value.toString(),
      unit: entry.unit,
      display_digits: entry.display_digits.toString(),
      mt5_digits: entry.mt5_digits.toString(),
    }
  }

  function cancelEdit() {
    editingSymbol.value = null
    errorMsg.value = ''
  }

  async function saveEdit() {
    saving.value = true
    errorMsg.value = ''
    try {
      const pv = parseFloat(editRow.value.pip_value)
      if (isNaN(pv) || pv <= 0) { errorMsg.value = 'pip_value doit être > 0'; return }
      const mt5d = parseInt(editRow.value.mt5_digits)
      if (isNaN(mt5d) || mt5d < 0) { errorMsg.value = 'MT5 digits doit être >= 0'; return }
      const digits = computeDigits(editRow.value.pip_value)
      await invoke('save_conversion', {
        symbol: editRow.value.symbol, pipValue: pv, unit: 'pips', displayDigits: digits, mt5Digits: mt5d,
      })
      editingSymbol.value = null
      await loadConversions()
    } catch (e) {
      errorMsg.value = `Erreur: ${e}`
    } finally {
      saving.value = false
    }
  }

  async function addNew() {
    saving.value = true
    errorMsg.value = ''
    try {
      const sym = newSymbol.value.trim().toUpperCase()
      const pv = parseFloat(newPipValue.value)
      if (!sym) { errorMsg.value = 'Symbole requis'; return }
      if (isNaN(pv) || pv <= 0) { errorMsg.value = 'pip_value doit être > 0'; return }
      const mt5d = parseInt(newMt5Digits.value)
      if (isNaN(mt5d) || mt5d < 0) { errorMsg.value = 'MT5 digits doit être >= 0'; return }
      const digits = computeDigits(newPipValue.value)
      await invoke('save_conversion', {
        symbol: sym, pipValue: pv, unit: 'pips', displayDigits: digits, mt5Digits: mt5d,
      })
      showAddRow.value = false
      newSymbol.value = ''
      newPipValue.value = ''
      newMt5Digits.value = '5'
      await loadConversions()
    } catch (e) {
      errorMsg.value = `Erreur: ${e}`
    } finally {
      saving.value = false
    }
  }

  async function resetToDefault(symbol: string) {
    try {
      await invoke('delete_conversion', { symbol })
      await loadConversions()
    } catch (e) {
      errorMsg.value = `Erreur reset: ${e}`
    }
  }

  return {
    conversions, loading, saving, editingSymbol, editRow,
    newSymbol, newPipValue, newMt5Digits, showAddRow, errorMsg,
    computeDigits, computeExample, startEdit, cancelEdit, saveEdit, addNew, resetToDefault,
  }
}
