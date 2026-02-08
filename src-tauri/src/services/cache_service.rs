// services/cache_service.rs - Service de caching pour résultats
// Préparé pour utilisation future (mise en cache des résultats d'analyse)
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Limite par défaut du nombre d'entrées en cache
const DEFAULT_MAX_ENTRIES: usize = 500;

/// Compteur global monotone pour ordonner les insertions (résolution sub-seconde)
static INSERTION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Entrée en cache avec timestamp d'expiration et ordre d'insertion (pour LRU)
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    expires_at: u64,       // UNIX timestamp
    insertion_order: u64,   // Compteur monotone pour éviction LRU
}

/// Service de cache global avec TTL (Time To Live) et limite d'entrées LRU
pub struct CacheService<T: Clone> {
    cache: Arc<Mutex<HashMap<String, CacheEntry<T>>>>,
    ttl_seconds: u64,
    max_entries: usize,
}

impl<T: Clone> CacheService<T> {
    /// Créer un nouveau service de cache
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl_seconds,
            max_entries: DEFAULT_MAX_ENTRIES,
        }
    }

    /// Créer un service de cache avec limite personnalisée
    pub fn with_max_entries(ttl_seconds: u64, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl_seconds,
            max_entries,
        }
    }
    
    /// Obtenir une valeur du cache (retourne None si expiré ou inexistant)
    pub fn get(&self, key: &str) -> Option<T> {
        let now = current_unix_timestamp();
        let cache = self.cache.lock().map_err(|_| ()).ok()?;
        
        if let Some(entry) = cache.get(key) {
            if now < entry.expires_at {
                return Some(entry.data.clone());
            }
        }
        None
    }
    
    /// Mettre une valeur en cache. Évince l'entrée la plus ancienne si max atteint.
    pub fn set(&self, key: String, value: T) {
        let now = current_unix_timestamp();
        let expires_at = now + self.ttl_seconds;
        let order = INSERTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let entry = CacheEntry {
            data: value,
            expires_at,
            insertion_order: order,
        };
        
        if let Ok(mut cache) = self.cache.lock() {
            // Éviction LRU si la limite est atteinte (et ce n'est pas une mise à jour)
            if cache.len() >= self.max_entries && !cache.contains_key(&key) {
                // D'abord supprimer les expirés
                cache.retain(|_, e| e.expires_at > now);

                // Si toujours plein, évincer le plus ancien (plus petit insertion_order)
                if cache.len() >= self.max_entries {
                    if let Some(oldest_key) = cache
                        .iter()
                        .min_by_key(|(_, e)| e.insertion_order)
                        .map(|(k, _)| k.clone())
                    {
                        cache.remove(&oldest_key);
                    }
                }
            }

            cache.insert(key, entry);
        }
    }
    
    /// Nettoyer les entrées expirées
    pub fn cleanup_expired(&self) {
        let now = current_unix_timestamp();
        if let Ok(mut cache) = self.cache.lock() {
            cache.retain(|_, entry| entry.expires_at > now);
        }
    }
    
    /// Vider le cache complètement
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    
    #[test]
    fn test_cache_get_set() {
        let cache = CacheService::new(10);
        
        cache.set("key1".to_string(), 42);
        assert_eq!(cache.get("key1"), Some(42));
        assert_eq!(cache.get("key2"), None);
    }
    
    #[test]
    fn test_cache_expiration() {
        let cache = CacheService::new(1);  // 1 second TTL
        
        cache.set("key1".to_string(), 42);
        assert_eq!(cache.get("key1"), Some(42));
        
        thread::sleep(Duration::from_millis(1100));
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = CacheService::with_max_entries(60, 3); // max 3 entrées

        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        cache.set("c".to_string(), 3);
        // Cache plein (3/3), ajout d'une 4e → évince la plus ancienne ("a")
        cache.set("d".to_string(), 4);

        assert_eq!(cache.get("a"), None);    // évincée
        assert_eq!(cache.get("b"), Some(2)); // conservée
        assert_eq!(cache.get("d"), Some(4)); // nouvelle
    }

    #[test]
    fn test_cache_update_no_eviction() {
        let cache = CacheService::with_max_entries(60, 2);

        cache.set("a".to_string(), 1);
        cache.set("b".to_string(), 2);
        // Mettre à jour "a" ne doit pas évincer
        cache.set("a".to_string(), 10);

        assert_eq!(cache.get("a"), Some(10));
        assert_eq!(cache.get("b"), Some(2));
    }
}
