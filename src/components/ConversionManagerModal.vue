<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="fermer">
    <div class="modal-content conversion-manager">
      <!-- Header -->
      <div class="modal-header">
        <h2>⚙️ Gérer les Conversions Pips/Points</h2>
        <button class="close-btn" @click="fermer">✕</button>
      </div>

      <!-- Barre de recherche -->
      <div class="search-bar">
        <input 
          v-model="termeRecherche" 
          type="text"
          placeholder="Rechercher symbole (EUR, JPY, XAU...)"
          class="search-input"
        />
        <button @click="ouvrirAjoutRapide" class="btn-add-new" title="Ajouter une nouvelle paire">
          ➕ Ajouter
        </button>
        <button @click="toutEnPoints" class="btn-points-all" title="Passer toutes les paires en points">
          🎯 Tout en Points
        </button>
        <button @click="toutEnPourcentage" class="btn-percentage-all" title="Passer toutes les paires en pourcentage pour comparaison cross-asset">
          📊 Tout en %
        </button>
        <button @click="reinitialiserParDefaut" class="btn-secondary">
          🔄 Réinitialiser
        </button>
      </div>

      <!-- Tableau conversions -->
      <div class="table-wrapper">
        <table class="conversions-table">
          <thead>
            <tr>
              <th>Symbole</th>
              <th>Pip Value</th>
              <th>Unité</th>
              <th>Exemple</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr 
              v-for="conv in conversionsFiltrées" 
              :key="conv.symbol" 
              :class="{ 'is-edited': conv.is_custom }"
            >
              <td class="symbol-cell">{{ conv.symbol }}</td>
              <td class="editable">
                <input 
                  v-model.number="conv.pip_value" 
                  type="number" 
                  step="0.0001"
                  min="0.00001"
                  @input="marquerModif(conv.symbol)"
                  class="input-number"
                />
              </td>
              <td class="editable">
                <select 
                  v-model="conv.unit" 
                  @change="marquerModif(conv.symbol)"
                  class="input-select"
                >
                  <option value="pips">pips</option>
                  <option value="points">points</option>
                  <option value="$">$</option>
                  <option value="%">%</option>
                </select>
              </td>
              <td class="example-cell">
                {{ exemplesParSymbole[conv.symbol] }}
              </td>
              <td class="actions-cell">
                <button 
                  @click="sauvegarderLigne(conv.symbol)" 
                  class="btn-action btn-save"
                  title="Sauvegarder cette ligne"
                  :disabled="!modifieesSet.has(conv.symbol)"
                >
                  💾
                </button>
                <button 
                  @click="annulerModifications(conv.symbol)" 
                  v-if="modifieesSet.has(conv.symbol)"
                  class="btn-action btn-cancel"
                  title="Annuler les modifications"
                >
                  ↶
                </button>
                <button 
                  @click="revenirParDefaut(conv.symbol)" 
                  class="btn-action btn-reset"
                  title="Revenir à la valeur par défaut du broker"
                >
                  ↩️
                </button>
                <button 
                  @click="supprimerConversion(conv.symbol)" 
                  class="btn-action btn-delete"
                  title="Supprimer de la base de données"
                >
                  🗑️
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Footer -->
      <div class="modal-footer">
        <p v-if="compteurModifs > 0" class="info-message">
          📝 {{ compteurModifs }} changement(s) à enregistrer
        </p>
        <button @click="fermer" class="btn-secondary">Annuler</button>
        <button 
          v-if="compteurModifs > 0" 
          @click="sauvegarderEtRecalculer" 
          class="btn-primary"
          :disabled="enCoursDeSauvegarde"
        >
          {{ enCoursDeSauvegarde ? '⏳ Sauvegarde...' : '💾 Sauvegarder & Recalculer' }}
        </button>
        <button v-else @click="fermer" class="btn-primary">Fermer</button>
      </div>
    </div>

    <!-- Modale d'Ajout Rapide -->
    <div v-if="isAjoutRapideOuvert" class="confirm-overlay">
      <div class="confirm-content add-pair-modal">
        <div class="confirm-header">
          <h3>➕ Ajouter un nouveau symbole</h3>
        </div>
        <div class="confirm-body">
          <div class="form-group">
            <label>Symbole (ex: BTCUSD)</label>
            <input 
              v-model="nouveauSymbole.symbol" 
              type="text" 
              placeholder="NOMSYMBOLE"
              class="search-input"
              @input="nouveauSymbole.symbol = nouveauSymbole.symbol.toUpperCase()"
            />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Pip Value</label>
              <input 
                v-model.number="nouveauSymbole.pip_value" 
                type="number" 
                step="0.0001"
                class="search-input"
              />
            </div>
            <div class="form-group">
              <label>Unité</label>
              <select v-model="nouveauSymbole.unit" class="search-input">
                <option value="pips">pips</option>
                <option value="points">points</option>
                <option value="$">$</option>
              </select>
            </div>
          </div>
        </div>
        <div class="confirm-footer">
          <button @click="isAjoutRapideOuvert = false" class="btn-secondary">Annuler</button>
          <button 
            @click="validerAjoutRapide" 
            class="btn-primary" 
            :disabled="!nouveauSymbole.symbol || enCoursDajout"
          >
            {{ enCoursDajout ? '⏳ Ajout...' : 'Ajouter le symbole' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Toast de notification -->
    <Transition name="toast">
      <div v-if="notification" :class="['toast-notification', notification.type]">
        {{ notification.message }}
      </div>
    </Transition>

    <!-- Modale de Confirmation de Suppression -->
    <div v-if="isSuppressionEnCours" class="confirm-overlay">
      <div class="confirm-content">
        <div class="confirm-header">
          <h3>⚠️ Confirmation de suppression</h3>
        </div>
        <div class="confirm-body">
          <p>Êtes-vous sûr de vouloir supprimer <strong>{{ symboleASupprimer }}</strong> ?</p>
          <p class="warning-text">Cette action masquera la paire de la liste des analyses.</p>
        </div>
        <div class="confirm-footer">
          <button @click="annulerSuppression" class="btn-secondary">Annuler</button>
          <button @click="confirmerSuppression" class="btn-danger">
            {{ enCoursDeSuppression ? '⏳ Suppression...' : 'Confirmer la suppression' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useConversionStore } from '../stores/conversionStore'

interface ConversionEntry {
  symbol: string
  pip_value: number
  unit: string
  display_digits: number
  is_custom: boolean
}

const props = defineProps<{ 
  isOpen: boolean
  importedPairs?: string[]
}>()
const emit = defineEmits<{ 
  fermer: []
  sauvegarde: [] 
}>()

const conversions = ref<ConversionEntry[]>([])
const termeRecherche = ref('')
const modifieesSet = reactive(new Set<string>())
const etatInitial = new Map<string, ConversionEntry>()

// Store global pour synchroniser les conversions
const conversionStore = useConversionStore()
const enCoursDeSauvegarde = ref(false)

// États pour la suppression
const isSuppressionEnCours = ref(false)
const symboleASupprimer = ref('')
const enCoursDeSuppression = ref(false)

// États pour l'ajout
const isAjoutRapideOuvert = ref(false)
const enCoursDajout = ref(false)
const nouveauSymbole = ref({
  symbol: '',
  pip_value: 0.0001,
  unit: 'pips'
})

// Système de notification
const notification = ref<{ message: string, type: 'success' | 'error' } | null>(null)
let notificationTimeout: number | null = null

const conversionsFiltrées = computed(() => {
  let filtered = conversions.value.filter(c => 
    c.symbol.toUpperCase().includes(termeRecherche.value.toUpperCase())
  )
  
  // Si des paires importées sont fournies, afficher UNIQUEMENT celles-ci
  if (props.importedPairs && props.importedPairs.length > 0) {
    filtered = filtered.filter(c => 
      props.importedPairs!.some(p => p.toUpperCase() === c.symbol.toUpperCase())
    )
  }
  
  return filtered
})

const compteurModifs = computed(() => modifieesSet.size)

// Computed pour les exemples afin que Vue réactualise quand pip_value change
const exemplesParSymbole = computed(() => {
  const result: Record<string, string> = {}
  conversions.value.forEach(conv => {
    result[conv.symbol] = calculerExemple(conv.pip_value, conv.unit, conv.symbol)
  })
  return result
})

// Calcule automatiquement le nombre de chiffres d'affichage à partir du pip_value
function calculerChiffresAffichage(pipValue: number): number {
  if (pipValue >= 1.0) return 0
  if (pipValue <= 0) return 4 // Default
  let digits = 0
  let val = pipValue
  while (val < 1.0 && digits < 10) {
    val *= 10.0
    digits += 1
  }
  return digits
}

watch(() => props.isOpen, async (isOpen) => {
  if (isOpen) {
    await chargerConversions()
  }
})

// Recharger aussi quand les paires importées changent
watch(() => props.importedPairs, async () => {
  if (props.isOpen) {
    await chargerConversions()
  }
}, { deep: true })

async function chargerConversions() {
  try {
    let conversions_list = await invoke<ConversionEntry[]>('get_all_conversions')
    
    // Ajouter les paires importées manquantes avec conversions par défaut
    if (props.importedPairs && props.importedPairs.length > 0) {
      const existingSymbols = new Set(conversions_list.map(c => c.symbol.toUpperCase()))
      
      for (const pair of props.importedPairs) {
        if (!existingSymbols.has(pair.toUpperCase())) {
          // Créer une conversion par défaut basée sur le symbole
          const defaultConv = createDefaultConversion(pair)
          if (defaultConv) {
            conversions_list.push(defaultConv)
          }
        }
      }
    }
    
    conversions.value = conversions_list
    modifieesSet.clear()
    etatInitial.clear()
    // Sauvegarder l'état initial
    conversions.value.forEach(conv => {
      etatInitial.set(conv.symbol, JSON.parse(JSON.stringify(conv)))
    })
  } catch (e) {
    afficherErreur('Erreur lors du chargement des conversions')
  }
}

function createDefaultConversion(symbol: string): ConversionEntry | null {
  const s = symbol.toUpperCase()
  
  // Détecter le type d'asset et retourner conversion par défaut
  if (s.includes('JPY')) {
    return { symbol: s, pip_value: 0.01, unit: 'pips', display_digits: 2, is_custom: false }
  } else if (s.includes('XAU') || s.includes('GOLD')) {
    return { symbol: s, pip_value: 0.01, unit: '$', display_digits: 2, is_custom: false }
  } else if (s.includes('XAG') || s.includes('SILVER')) {
    return { symbol: s, pip_value: 0.01, unit: '$', display_digits: 2, is_custom: false }
  } else if (s.includes('BTC') || s.includes('ETH') || s.includes('CRYPTO')) {
    return { symbol: s, pip_value: 1.0, unit: '$', display_digits: 0, is_custom: false }
  } else if (s.includes('OIL') || s.includes('WTI') || s.includes('BRENT')) {
    return { symbol: s, pip_value: 0.01, unit: '$', display_digits: 2, is_custom: false }
  } else if (s.includes('NGAS')) {
    return { symbol: s, pip_value: 0.001, unit: '$', display_digits: 3, is_custom: false }
  } else if (s.includes('VIX') || s.includes('IDX') || s.includes('NAS') || s.includes('SP') || s.includes('DEU') || s.includes('US3') || s.includes('US')) {
    return { symbol: s, pip_value: 1.0, unit: 'points', display_digits: 0, is_custom: false }
  } else {
    // Par défaut: Forex major
    return { symbol: s, pip_value: 0.0001, unit: 'pips', display_digits: 4, is_custom: false }
  }
}

function marquerModif(symbol: string) {
  const conv = conversions.value.find(c => c.symbol === symbol)
  if (conv) {
    // Recalculer automatiquement display_digits
    conv.display_digits = calculerChiffresAffichage(conv.pip_value)
  }
  modifieesSet.add(symbol)
}

/**
 * Calcule un exemple d'affichage : "X pips = Y en unité finale"
 * Le nombre de pips varie selon le symbole pour des exemples réalistes
 * Affiche UNIQUEMENT des nombres entiers
 */
function calculerExemple(pipValue: number, unit: string, symbol: string): string {
  const sym = symbol.toUpperCase()
  
  // Déterminer le nombre de pips exemple selon le symbole
  let examplePips = 15 // Default pour Forex classique
  
  if (sym.includes('BTC') || sym.includes('ETH')) {
    examplePips = 350
  } else if (sym.includes('XAU') || sym.includes('GOLD')) {
    examplePips = 50
  } else if (sym.includes('XAG') || sym.includes('SILVER')) {
    examplePips = 15
  } else if (sym.includes('JPY') || sym.includes('HUF')) {
    examplePips = 20
  }
  
  const priceMovement = examplePips * pipValue
  
  if (unit === 'pips') {
    return `${examplePips} pips = ${Math.round(examplePips)} pips`
  } else if (unit === 'points') {
    return `${examplePips} pips = ${Math.round(priceMovement)} pts`
  } else if (unit === '$') {
    return `${examplePips} pips = $${Math.round(priceMovement)}`
  }
  return `${examplePips} × ${pipValue.toFixed(5)}`
}

function ouvrirAjoutRapide() {
  nouveauSymbole.value = {
    symbol: '',
    pip_value: 0.0001,
    unit: 'pips'
  }
  isAjoutRapideOuvert.value = true
}

async function validerAjoutRapide() {
  if (!nouveauSymbole.value.symbol) return
  
  try {
    enCoursDajout.value = true
    const s = nouveauSymbole.value
    
    // On utilise save_conversion pour l'ajout aussi
    await invoke('save_conversion', {
      symbol: s.symbol.toUpperCase(),
      pipValue: s.pip_value,
      unit: s.unit,
      displayDigits: calculerChiffresAffichage(s.pip_value)
    })
    
    await chargerConversions()
    afficherSucces(`✓ ${s.symbol} ajouté avec succès`)
    isAjoutRapideOuvert.value = false
  } catch (e) {
    afficherErreur(`Erreur lors de l'ajout: ${e}`)
  } finally {
    enCoursDajout.value = false
  }
}

async function sauvegarderLigne(symbol: string) {
  const conv = conversions.value.find(c => c.symbol === symbol)
  if (!conv) return
  
  try {
    await invoke('save_conversion', {
      symbol: conv.symbol,
      pipValue: conv.pip_value,
      unit: conv.unit,
      displayDigits: conv.display_digits
    })
    
    // Mettre à jour l'état initial pour que l'annulation revienne à cet état
    etatInitial.set(symbol, JSON.parse(JSON.stringify(conv)))
    
    // Invalider le cache pour que les changements soient pris en compte
    await invoke('invalidate_analysis_cache')
    
    // IMPORTANT: Recharger le store Pinia pour mettre à jour HeatmapTable
    await conversionStore.loadConversions()
    
    modifieesSet.delete(symbol)
    afficherSucces(`✓ ${symbol} sauvegardé`)
    
    // Notifier le parent pour rafraîchir les vues
    emit('sauvegarde')
  } catch (e) {
    afficherErreur(`Erreur: ${e}`)
  }
}

function annulerModifications(symbol: string) {
  const initial = etatInitial.get(symbol)
  const conv = conversions.value.find(c => c.symbol === symbol)
  if (initial && conv) {
    conv.pip_value = initial.pip_value
    conv.unit = initial.unit
    conv.display_digits = initial.display_digits
    conv.is_custom = initial.is_custom
    modifieesSet.delete(symbol)
  }
}

// Ouvrir la confirmation de suppression
function supprimerConversion(symbol: string) {
  symboleASupprimer.value = symbol
  isSuppressionEnCours.value = true
}

// Annuler la suppression
function annulerSuppression() {
  isSuppressionEnCours.value = false
  symboleASupprimer.value = ''
}

// Confirmer et exécuter la suppression
async function confirmerSuppression() {
  if (!symboleASupprimer.value) return
  
  try {
    enCoursDeSuppression.value = true
    await invoke('delete_conversion', { symbol: symboleASupprimer.value })
    
    // Invalider le cache car une paire a été supprimée
    await invoke('invalidate_analysis_cache')
    
    // Recharger les données
    await chargerConversions()
    
    afficherSucces(`✓ ${symboleASupprimer.value} supprimé de la base`)
    
    // Fermer la confirmation
    annulerSuppression()
    
    // Émettre un événement pour que le parent sache qu'il faut recalculer si besoin
    emit('sauvegarde')
    
  } catch (e) {
    afficherErreur(`Erreur suppression: ${e}`)
  } finally {
    enCoursDeSuppression.value = false
  }
}

function revenirParDefaut(symbol: string) {
  const conv = conversions.value.find(c => c.symbol === symbol)
  if (conv) {
    chargerDefaut(symbol).then((defaults) => {
      if (defaults) {
        conv.pip_value = defaults.pip_value
        conv.unit = defaults.unit
        conv.display_digits = defaults.display_digits
        conv.is_custom = false
        modifieesSet.delete(symbol)
      }
    })
  }
}

async function chargerDefaut(symbol: string): Promise<ConversionEntry | null> {
  try {
    return await invoke<ConversionEntry>('get_default_conversion', { symbol })
  } catch (e) {
    return null
  }
}

async function reinitialiserParDefaut() {
  if (!confirm('Êtes-vous sûr? Cela réinitialisera TOUTES les conversions.')) {
    return
  }

  for (const conv of conversions.value) {
    const defaults = await chargerDefaut(conv.symbol)
    if (defaults) {
      conv.pip_value = defaults.pip_value
      conv.unit = defaults.unit
      conv.display_digits = defaults.display_digits
      conv.is_custom = false
    }
  }
  modifieesSet.clear()
}

/**
 * Passe toutes les paires affichées en unité "%"
 * pour une comparaison cross-asset uniforme
 * IMPORTANT: Restaure les pip_value originaux pour que le calcul de % soit correct
 */
function toutEnPourcentage() {
  for (const conv of conversions.value) {
    conv.unit = '%'
    // Restaurer la pip_value originale depuis etatInitial pour que le calcul soit correct
    const initial = etatInitial.get(conv.symbol)
    if (initial) {
      conv.pip_value = initial.pip_value
      conv.display_digits = initial.display_digits
    }
    modifieesSet.add(conv.symbol)
  }
  afficherSucces("✅ Toutes les paires sont passées en % (Volatilité normalisée)")
}

/**
 * Passe toutes les paires affichées en unité "points"
 * avec des Pip Values adaptées pour une lecture granulaire (1 pip = 10 points)
 */
function toutEnPoints() {
  for (const conv of conversions.value) {
    const sym = conv.symbol.toUpperCase()
    conv.unit = 'points'

    // Logique de conversion : 1 pip = 10 points partout
    if (sym.includes('JPY') || sym.includes('HUF')) {
      conv.pip_value = 0.01     // 1 pip = 10 points
    } else if (sym.includes('XAU') || sym.includes('GOLD')) {
      conv.pip_value = 0.10     // 1 pip = 0.10$ = 10 points (0.01 chacun)
    } else if (sym.includes('XAG') || sym.includes('SILVER')) {
      conv.pip_value = 0.01     // 1 pip = 0.01$ = 10 points (0.001 chacun)
    } else if (sym.includes('BTC') || sym.includes('ETH') || sym.includes('US30') || sym.includes('NAS') || sym.includes('DAX')) {
      conv.pip_value = 1.0      // 1 point = 1$ ou 1 point d'indice
    } else if (sym.includes('NGAS')) {
      conv.pip_value = 0.001    // Gaz naturel
    } else {
      // Forex Standard (EURUSD, GBPUSD, etc., 5 décimales)
      conv.pip_value = 0.0001   // 1 pip = 10 points
    }

    conv.display_digits = calculerChiffresAffichage(conv.pip_value)
    conv.is_custom = true
    modifieesSet.add(conv.symbol)
  }
  afficherSucces("Toutes les paires sont passées en Points (Granulaire : 1 pip = 10 points)")
}

async function sauvegarderEtRecalculer() {
  enCoursDeSauvegarde.value = true

  try {
    // Sauvegarder chaque conversion modifiée
    for (const symbol of modifieesSet) {
      const conv = conversions.value.find(c => c.symbol === symbol)
      if (conv) {
        await invoke('save_conversion', {
          symbol: conv.symbol,
          pipValue: conv.pip_value,
          unit: conv.unit,
          displayDigits: conv.display_digits
        })
      }
    }

    // Invalider le cache
    await invoke('invalidate_analysis_cache')

    // IMPORTANT: Recharger le store Pinia pour mettre à jour HeatmapTable
    await conversionStore.loadConversions()

    // Reset
    modifieesSet.clear()
    afficherSucces('Conversions sauvegardées et analyses recalculées!')
    
    emit('sauvegarde')
    setTimeout(() => {
      emit('fermer')
    }, 500)
  } catch (e) {
    afficherErreur(`Erreur sauvegarde: ${e}`)
  } finally {
    enCoursDeSauvegarde.value = false
  }
}

function fermer() {
  if (compteurModifs.value > 0) {
    if (!confirm('Vous avez des changements non enregistrés. Fermer quand même?')) {
      return
    }
  }
  emit('fermer')
}

function afficherErreur(message: string) {
  if (notificationTimeout) clearTimeout(notificationTimeout)
  notification.value = { message: `❌ ${message}`, type: 'error' }
  notificationTimeout = window.setTimeout(() => {
    notification.value = null
  }, 4000)
}

function afficherSucces(message: string) {
  if (notificationTimeout) clearTimeout(notificationTimeout)
  notification.value = { message: `✓ ${message}`, type: 'success' }
  notificationTimeout = window.setTimeout(() => {
    notification.value = null
  }, 3000)
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.modal-content {
  background: #1e1e2e;
  border: 1px solid #30363d;
  border-radius: 8px;
  padding: 24px;
  max-width: 1100px;
  width: 95%;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.8);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  border-bottom: 2px solid #30363d;
  padding-bottom: 15px;
}

.modal-header h2 {
  margin: 0;
  color: #c9d1d9;
  font-size: 18px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: #8b949e;
  font-size: 24px;
  cursor: pointer;
  transition: color 0.2s;
}

.close-btn:hover {
  color: #c9d1d9;
}

.search-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.search-input {
  flex: 1;
  padding: 10px 14px;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 6px;
  color: #c9d1d9;
  font-size: 14px;
  transition: border-color 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 3px rgba(88, 166, 255, 0.1);
}

.table-wrapper {
  overflow-x: auto;
  margin-bottom: 20px;
  flex: 1;
  border: 1px solid #21262d;
  border-radius: 6px;
}

.conversions-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.conversions-table thead {
  background: #0d1117;
  position: sticky;
  top: 0;
}

.conversions-table th {
  padding: 12px 14px;
  text-align: left;
  border-bottom: 2px solid #30363d;
  color: #8b949e;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.conversions-table td {
  padding: 12px 14px;
  border-bottom: 1px solid #21262d;
  color: #c9d1d9;
}

.conversions-table th:nth-child(2),
.conversions-table td:nth-child(2) {
  width: 180px;
}

.conversions-table th:nth-child(3),
.conversions-table td:nth-child(3) {
  width: 140px;
}

.conversions-table th:nth-child(4),
.conversions-table td:nth-child(4) {
  width: 230px;
}

.conversions-table tr {
  transition: background-color 0.2s;
}

.conversions-table tr:hover {
  background-color: rgba(88, 166, 255, 0.05);
}

.conversions-table tr.is-edited {
  background: rgba(88, 166, 255, 0.1);
}

.symbol-cell {
  font-weight: 600;
  color: #58a6ff;
  font-family: 'Monaco', 'Courier New', monospace;
}

.example-cell {
  color: #8b949e;
  font-family: 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  padding: 12px 14px;
  border-bottom: 1px solid #21262d;
}

.editable input,
.editable select {
  width: 100%;
  padding: 8px 10px;
  background: #0d1117;
  border: 1px solid #30363d;
  border-radius: 4px;
  color: #c9d1d9;
  font-size: 13px;
  font-family: 'Monaco', 'Courier New', monospace;
  transition: all 0.2s;
}

.editable select {
  padding-right: 24px;
  appearance: none;
  background-image: url("data:image/svg+xml;charset=UTF-8,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2'%3e%3cpolyline points='6 9 12 15 18 9'%3e%3c/polyline%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 16px;
}

.editable select option {
  color: #c9d1d9;
  background: #0d1117;
  padding: 8px;
}

.editable input:focus,
.editable select:focus {
  outline: none;
  border-color: #58a6ff;
  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.2);
}

.badge-custom {
  color: #f0883e;
  font-weight: 500;
}

.badge-default {
  color: #238636;
  font-weight: 500;
}

.btn-action {
  padding: 6px 10px;
  background: #21262d;
  color: #58a6ff;
  border: 1px solid #58a6ff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
  flex-shrink: 0;
}

.btn-action:hover:not(:disabled) {
  background: #58a6ff;
  color: #0d1117;
  transform: scale(1.05);
}

.btn-action:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-cancel {
  color: #f0883e;
  border-color: #f0883e;
}

.btn-cancel:hover:not(:disabled) {
  background: #f0883e;
  color: #0d1117;
}

.btn-delete {
  color: #da3633;
  border-color: #da3633;
}

.btn-delete:hover:not(:disabled) {
  background: #da3633;
  color: #fff;
}

.actions-cell {
  display: flex;
  gap: 8px;
  align-items: center;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  align-items: center;
  border-top: 1px solid #30363d;
  padding-top: 20px;
  flex-wrap: wrap;
}

.info-message {
  margin-right: auto;
  color: #f0883e;
  font-size: 13px;
  margin: 0;
  font-weight: 500;
}

.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-primary {
  background: #238636;
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: #2ea043;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(35, 134, 54, 0.3);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: #1f6feb;
  color: white;
}

.btn-secondary:hover {
  background: #388bfd;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(31, 111, 235, 0.3);
}

.btn-add-new {
  background: #238636;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0 16px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-add-new:hover {
  background: #2ea043;
  transform: translateY(-1px);
}

.btn-points-all {
  background: #d29922;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 0 16px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.btn-points-all:hover {
  background: #e3b341;
  transform: translateY(-1px);
}

.form-group {
  margin-bottom: 16px;
  text-align: left;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  color: #8b949e;
  font-size: 12px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.add-pair-modal {
  border-color: #238636 !important;
  box-shadow: 0 0 30px rgba(35, 134, 54, 0.2) !important;
}

.confirm-overlay {
  position: absolute; /* Relative to modal-overlay */
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  border-radius: 8px;
  backdrop-filter: blur(4px);
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.confirm-content {
  background: #1e1e2e;
  border: 1px solid #da3633;
  border-radius: 12px;
  padding: 24px;
  max-width: 400px;
  width: 90%;
  box-shadow: 0 0 30px rgba(218, 54, 51, 0.2);
  transform: scale(1);
  animation: scaleUp 0.2s ease-out;
}

@keyframes scaleUp {
  from { transform: scale(0.9); }
  to { transform: scale(1); }
}

.confirm-header h3 {
  margin: 0 0 16px 0;
  color: #ff7b72;
  font-size: 18px;
  text-align: center;
}

.confirm-body {
  margin-bottom: 24px;
  text-align: center;
  color: #c9d1d9;
}

.warning-text {
  color: #8b949e;
  font-size: 13px;
  font-style: italic;
  margin-top: 12px;
}

.confirm-footer {
  display: flex;
  justify-content: center;
  gap: 12px;
}

.btn-danger {
  background: #da3633;
  color: white;
  border: none;
  border-radius: 6px;
  padding: 10px 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-danger:hover:not(:disabled) {
  background: #f85149;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(218, 54, 51, 0.4);
}

.btn-danger:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Toast System */
.toast-notification {
  position: fixed;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  border-radius: 8px;
  z-index: 2000;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.4);
  font-weight: 600;
  font-size: 14px;
  min-width: 200px;
  text-align: center;
}

.toast-notification.success {
  background: #238636;
  color: white;
  border: 1px solid #2ea043;
}

.toast-notification.error {
  background: #da3633;
  color: white;
  border: 1px solid #f85149;
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translate(-50%, -20px);
}

@media (max-width: 768px) {
  .modal-content {
    max-height: 90vh;
    padding: 16px;
  }

  .modal-header h2 {
    font-size: 16px;
  }

  .search-bar {
    flex-direction: column;
  }

  .modal-footer {
    flex-direction: column-reverse;
  }

  .info-message {
    margin-right: 0;
    margin-bottom: 8px;
    text-align: center;
  }

  .btn-primary,
  .btn-secondary {
    width: 100%;
  }
}
</style>
