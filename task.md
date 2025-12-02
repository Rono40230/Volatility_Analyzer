# 📋 TODO - PHASE 8 REFACTORING METRIQUES RETROSPECTIVES

## ✅ STATUS: PHASE 8 COMPLÉTÉE

**Durée réelle:** 1.5h (estimation: 2-3h)

---

## 🎯 RÉSULTAT FINAL

**Refactoriser les Métriques Retrospectives** pour les rendre vraiment utilisables en trading:
1. Supprimer les 3 onglets qui demandent saisie manuelle (inutiles)
2. Améliorer les 2 onglets restants avec filtrage par type d'événement
3. Résultats spécifiques au type d'événement (NFP vs Inflation vs autres)

**Raison:** L'app analyse des CSV, pas du backtesting manuel. Les moyennes globales sur 27,871 événements mélangés sont inutilisables en trading.

---

## 📊 PLAN D'EXÉCUTION

### **PHASE 8.1: SUPPRESSION (3 onglets inutiles)**

#### Backend Rust - Suppression Complète

**A. Commandes Tauri** (`src-tauri/src/commands/retrospective_analysis_commands.rs`)
- [ ] Supprimer fonction `analyze_entry_timing()`
- [ ] Supprimer fonction `analyze_directional_bias()`
- [ ] Supprimer fonction `analyze_whipsaw_root_cause()`
- [ ] Supprimer structs: `EntryTimingResult`, `DirectionalBiasResult`, `WhipsawRootCauseResult`
- [ ] Garder: `analyze_peak_delay()` et `analyze_decay_profile()` + structs correspondants

**B. Handler Registration** (`src-tauri/src/lib.rs`)
- [ ] Retirer du `generate_handler!` macro les 3 commandes supprimées
- [ ] Garder les 2 commandes restantes en place

**C. Services Volatility** (`src-tauri/src/services/volatility/`)
- [ ] Supprimer fichier: `win_rate_calculator.rs`
- [ ] Supprimer fichier: `directional_bias_analyzer.rs` (si existe)
- [ ] Supprimer fichier: `whipsaw_classifier.rs`
- [ ] Garder: `volatility_duration_analyzer.rs` (utilisé par peak_delay/decay)

**D. Service Exports** (`src-tauri/src/services/volatility/mod.rs`)
- [ ] Retirer: `pub use win_rate_calculator::*`
- [ ] Retirer: `pub use directional_bias_analyzer::*`
- [ ] Retirer: `pub use whipsaw_classifier::*`

#### Frontend Vue - Suppression Complète

**A. Composants à Supprimer**
- [ ] Supprimer fichier: `src/components/EntryTimingProfitability.vue`
- [ ] Supprimer fichier: `src/components/DirectionalBiasView.vue`
- [ ] Supprimer fichier: `src/components/WhipsawRootCauseView.vue`

**B. Composable Cleanup** (`src/composables/useRetrospectiveAnalysis.ts`)
- [ ] Retirer refs: `entryTimingLoading`, `entryTimingError`, `entryTimingResults`
- [ ] Retirer refs: `biasLoading`, `biasError`, `biasResults`
- [ ] Retirer refs: `whipsawLoading`, `whipsawError`, `whipsawResults`
- [ ] Retirer fonction: `analyzeEntryTiming()`
- [ ] Retirer fonction: `analyzeDirectionalBias()`
- [ ] Retirer fonction: `analyzeWhipsawRootCause()`
- [ ] Garder: peak delay et decay profile (tous les refs/fonctions)

**C. EventCorrelationView** (`src/components/EventCorrelationView.vue`)
- [ ] Retirer imports des 3 composants supprimés
- [ ] Retirer condition v-if pour mode 'entry-timing'
- [ ] Retirer condition v-if pour mode 'bias'
- [ ] Retirer condition v-if pour mode 'whipsaw'
- [ ] Garder: Logique pour peak-delay et decay

**D. Tab Component** (`src/components/RetrospectiveViewModeTabs.vue`)
- [ ] Retirer bouton: "📊 Fenêtres Entrée"
- [ ] Retirer bouton: "🎯 Biais Directionnel"
- [ ] Retirer bouton: "⚡ Causes Whipsaw"
- [ ] Garder: "⏰ Délai Peak" et "📉 Décroissance Vol."
- [ ] Mettre à jour type d'émission: `'peak-delay' | 'decay'` (au lieu de 5 modes)

---

### **PHASE 8.2: AMÉLIORATION (2 onglets rendus utilisables)**

#### Backend Rust - Modification des Commandes

**A. analyze_peak_delay()** - Ajouter filtrage par event_type
- [ ] Ajouter paramètre: `event_type: String` (ex: "NFP", "Inflation")
- [ ] Modifier requête SQL pour filtrer `calendar_events.event_type = event_type`
- [ ] Calculer moyenne **uniquement sur ce type d'événement**
- [ ] Retourner aussi: `event_count: usize` (combien d'événements de ce type)

**B. analyze_decay_profile()** - Ajouter filtrage par event_type
- [ ] Ajouter paramètre: `event_type: String`
- [ ] Modifier requête SQL pour filtrer par type
- [ ] Calculer moyenne **uniquement sur ce type d'événement**
- [ ] Retourner aussi: `event_count: usize`

**C. Nouvelle Commande** (optionnel mais utile)
- [ ] Créer `get_event_types()` pour lister tous les types disponibles
- [ ] Retourne: `Vec<String>` avec ["NFP", "Inflation", "Unemployment", ...]

#### Frontend Vue - Ajout Dropdowns

**A. PeakDelayAnalysis.vue** - Ajouter dropdown event_type
- [ ] Ajouter ref: `eventType` (initialisé à "NFP" ou premier de la liste)
- [ ] Ajouter dropdown HTML avec liste des types d'événements
- [ ] Passer `event_type` à la fonction `analyzePeakDelay(candles, eventType)`
- [ ] Afficher aussi: "Basé sur X événements de ce type"

**B. DecayProfileView.vue** - Ajouter dropdown event_type
- [ ] Ajouter ref: `eventType`
- [ ] Ajouter dropdown HTML
- [ ] Passer `event_type` à la fonction `analyzeDecayProfile(candles, eventType)`
- [ ] Afficher aussi: "Basé sur X événements de ce type"

**C. useRetrospectiveAnalysis.ts** - Mettre à jour composable
- [ ] Modifier signature: `analyzePeakDelay(candles, eventType)` au lieu de juste `candles`
- [ ] Modifier signature: `analyzeDecayProfile(candles, eventType)`
- [ ] Passer `eventType` à la commande Tauri via paramètre

**D. Charger Liste des Types** (optionnel)
- [ ] Au mount: appeler `get_event_types()` si possible
- [ ] Remplir dynamiquement le dropdown (au lieu de hardcoder)

---

### **PHASE 8.3: NETTOYAGE & TESTS**

#### Nettoyage Code
- [ ] `cargo fmt` et `cargo clippy`
- [ ] Vérifier zéro warnings Rust
- [ ] `npm run build` et `vue-tsc --noEmit`
- [ ] Vérifier zéro erreurs TypeScript

#### Tests
- [ ] `cargo test` - tous les tests passent
- [ ] Précommit checks - tous les seuils respectés (RÈGLE 15)
- [ ] App UI - vérifier dropdowns affichent les données correctes

#### Validation RÈGLE 15
- [ ] retrospective_analysis_commands.rs: < 200L
- [ ] PeakDelayAnalysis.vue: < 250L
- [ ] DecayProfileView.vue: < 250L
- [ ] useRetrospectiveAnalysis.ts: < 150L

---

## 📈 Résultat Final Attendu

**Avant (Inutilisable):**
```
Paire: BTCUSD
Délai Peak: 15 min (moyenne sur 27,871 événements mélangés)
```

**Après (Utilisable):**
```
Paire: BTCUSD
Événement: NFP
├─ Délai Peak: 3.2 min
├─ ATR Peak: 4500
├─ Confiance: 89%
└─ Basé sur: 248 événements NFP

Paire: BTCUSD
Événement: Inflation
├─ Délai Peak: 8.7 min
├─ ATR Peak: 3200
├─ Confiance: 76%
└─ Basé sur: 156 événements Inflation
```

---

## 🗂️ Fichiers Touchés

### À Supprimer (6 fichiers)
```
src-tauri/src/services/volatility/win_rate_calculator.rs
src-tauri/src/services/volatility/directional_bias_analyzer.rs
src-tauri/src/services/volatility/whipsaw_classifier.rs
src/components/EntryTimingProfitability.vue
src/components/DirectionalBiasView.vue
src/components/WhipsawRootCauseView.vue
```

### À Modifier (8 fichiers)
```
src-tauri/src/commands/retrospective_analysis_commands.rs (145L → 110L)
src-tauri/src/commands/mod.rs (clean exports)
src-tauri/src/lib.rs (clean handler)
src-tauri/src/services/volatility/mod.rs (clean exports)
src/composables/useRetrospectiveAnalysis.ts (140L → 90L)
src/components/EventCorrelationView.vue
src/components/RetrospectiveViewModeTabs.vue
src/components/PeakDelayAnalysis.vue (+event_type param)
src/components/DecayProfileView.vue (+event_type param)
```

---

## ✅ Checklist Finale

- [ ] Phase 8.1 complétée (suppression)
- [ ] Phase 8.2 complétée (amélioration)
- [ ] Phase 8.3 complétée (tests)
- [ ] `cargo check` 0 errors
- [ ] `npm run build` success
- [ ] Pre-commit checks PASSING
- [ ] Git commit réussi
- [ ] Git push réussi
- [ ] task.md mis à jour avec résultats

---

**Status:** 🔴 NOT STARTED - Prêt à commencer Phase 8.1

**Durée Estimée:** 2-3 heures (suppression + amélioration + tests)
