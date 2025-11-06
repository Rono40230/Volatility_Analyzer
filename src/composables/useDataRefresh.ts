// src/composables/useDataRefresh.ts
// Système d'événements pour rafraîchir automatiquement les listes après import
import { ref } from 'vue'

type RefreshCallback = () => Promise<void> | void

const pairDataRefreshListeners = new Set<RefreshCallback>()

export function useDataRefresh() {
  const isRefreshing = ref(false)

  // S'abonner aux événements de rafraîchissement
  function onPairDataRefresh(callback: RefreshCallback) {
    pairDataRefreshListeners.add(callback)
    
    // Retourner une fonction de désabonnement
    return () => {
      pairDataRefreshListeners.delete(callback)
    }
  }

  // Notifier tous les abonnés qu'un rafraîchissement est nécessaire
  async function triggerPairDataRefresh() {
    if (isRefreshing.value) return
    
    isRefreshing.value = true
    try {
      const promises = Array.from(pairDataRefreshListeners).map(callback => callback())
      await Promise.all(promises)
    } finally {
      isRefreshing.value = false
    }
  }

  return {
    isRefreshing,
    onPairDataRefresh,
    triggerPairDataRefresh
  }
}
