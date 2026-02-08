# Plan de Corrections et Améliorations — Volatility Analyzer

> **Objectif** : Amener l'application à un niveau de confiance opérationnel pour qu'un trader puisse s'appuyer sur ses états sans vérification manuelle.
>
> **Fiabilité actuelle estimée** : ~60%
> **Fiabilité cible** : ≥ 95%
>
> **Méthodologie** : Les tâches sont classées par priorité décroissante. Chaque phase doit être validée (`cargo test` + `cargo clippy -- -D warnings`) avant de passer à la suivante.

---

## PHASE 1 — BUGS CRITIQUES (Confiance bloquée)

> Ces bugs faussent directement les résultats affichés au trader. Aucune autre amélioration n'a de sens tant qu'ils existent.

---

### 1.1 ❌ Prix hardcodé à 100000 dans `analyzeTop3Slices()`

**Fichier** : `src/utils/straddleAnalysis.ts`
**Ligne** : ~16 (`calculateTradingPlan(slice.stats, 100000, score)`)
**Impact** : CRITIQUE — Fausse tous les TP/SL en USD pour le Forex (prix réel EURUSD ~1.08, pas 100000)

**Correction** :
- Remplacer `100000` par un appel à `estimatePrice(slice.stats)` ou, mieux, utiliser le `point_value` renvoyé par le backend dans `AnalysisResult`
- Propager `point_value` et `unit` depuis le store jusqu'à `analyzeTop3Slices()`
- Valider que les TP/SL en USD sont cohérents pour EURUSD (~1.08), USDJPY (~150), XAUUSD (~2000)

**Test de validation** :
- Lancer une analyse sur EURUSD, USDJPY, XAUUSD et vérifier que les valeurs de TP/SL en USD sont réalistes

---

### 1.2 ❌ Formule ATR incorrecte dans l'analyse rétrospective

**Fichier** : `src-tauri/src/commands/retrospective_analysis/helpers.rs`
**Fonction** : `calculer_atr()`
**Impact** : HAUTE — Utilise `close.abs()` au lieu de `|H - prev_close|` et `|L - prev_close|` pour le True Range

**Correction** :
```
TR = max(high - low, |high - prev_close|, |low - prev_close|)
```
- Modifier la fonction pour prendre en paramètre `prev_close: Option<f64>`
- Utiliser la bonne formule TR standard (celle déjà correcte dans `slice_metrics_analyzer.rs`)
- Si `prev_close` n'est pas disponible, fallback sur `high - low`

**Test de validation** :
- Test unitaire avec des données connues (ex: gap overnight) vérifiant que le TR inclut le gap

---

### 1.3 ❌ Pénalité de confiance -15 morte dans le scorer

**Fichier** : `src-tauri/src/services/volatility/confidence_scorer.rs`
**Ligne** : ~139 (`mean_atr > 0.0002`)
**Impact** : HAUTE — La pénalité pour "haute volatilité + bruit" ne se déclenche jamais car `mean_atr` est en pips normalisés (ex: 2.5), pas en prix brut (0.0002)

**Correction** :
- Remplacer le seuil `0.0002` par un seuil en pips cohérent (ex: `2.0`)
- Vérifier les autres seuils du même fichier pour la même erreur d'unité
- Documenter en commentaire l'unité attendue pour chaque seuil

**Test de validation** :
- Test unitaire avec mean_atr=3.0 et noise_ratio=4.0 → la pénalité -15 doit s'appliquer

---

### 1.4 ❌ Biais TP-first dans le simulateur de backtest

**Fichier** : `src-tauri/src/services/backtest/simulator.rs`
**Impact** : HAUTE — Sur une bougie où H touche le TP ET L touche le SL, le simulateur prend toujours le TP pour les Longs (biais optimiste systématique)

**Correction** :
- Quand TP ET SL sont tous deux atteints sur la même bougie, appliquer une logique de priorité réaliste :
  - Option A (conservatrice, recommandée) : considérer comme SL touché en premier (worst case)
  - Option B (aléatoire) : 50/50
  - Option C (basée sur l'open) : si Open est plus proche du SL → SL d'abord, sinon TP d'abord
- Appliquer la même logique pour les Shorts (vérifier SL avant TP)
- Documenter le choix de priorité en commentaire

**Test de validation** :
- Test unitaire avec une bougie dont H = TP+1 et L = SL-1. Vérifier que le résultat est un SL (option A) ou cohérent avec la stratégie choisie

---

### 1.5 ❌ `win_rate_adjusted` toujours à 0.0 via `new_with_pair()`

**Fichier** : `src-tauri/src/services/straddle_adjustments.rs`
**Fonction** : `AdjustedMetrics::new_with_pair()`
**Ligne** : ~97
**Impact** : HAUTE — Le win rate ajusté affiché dans l'UI est toujours 0% quand on passe par cette fonction

**Correction** :
- Calculer un vrai win_rate ou ne pas appeler cette fonction quand on a besoin d'un win rate
- Si le win rate n'est pas calculable dans ce contexte, ne pas inclure `win_rate_adjusted` dans la réponse (utiliser `Option<f64>` au lieu de `f64`)
- Côté frontend, ne pas afficher un win rate de 0% comme s'il était réel — afficher "N/A" ou masquer le champ

**Test de validation** :
- Vérifier dans l'UI que le win rate ajusté n'affiche plus jamais "0.0%" sauf si c'est réellement 0

---

### 1.6 ❌ Format timestamp incompatible entre les deux importeurs

**Fichiers** :
- `src-tauri/src/commands/pair_importer.rs` → stocke en `%Y-%m-%d %H:%M:%S`
- `src-tauri/src/commands/pair_data/processor.rs` → stocke en RFC3339
- `src-tauri/src/services/database_loader.rs` → lit en RFC3339

**Impact** : CRITIQUE — Les candles importées par `pair_importer.rs` ne sont pas lisibles par le `DatabaseLoader` → **perte de données silencieuse**

**Correction** :
- Unifier sur RFC3339 (`2025-01-15T14:30:00+00:00`) partout
- Supprimer `pair_importer.rs` et ne garder que la version UPSERT de `processor.rs` (ou aligner les deux)
- Ajouter un script de migration pour convertir les timestamps existants en RFC3339
- Ajouter un test d'intégration : importer un CSV → relire les données → vérifier que les dates sont correctes

**Test de validation** :
- Import CSV puis `DatabaseLoader::load()` → les données doivent être retrouvées sans erreur

---

## PHASE 2 — INCOHÉRENCES DE CALCUL (Résultats non fiables)

> Ces incohérences font que deux modules donnent des réponses contradictoires pour la même question.

---

### 2.1 🔶 Unifier la méthode de calcul ATR

**Situation actuelle** : 3 implémentations ATR divergentes
| Module | Méthode | Période |
|--------|---------|---------|
| `straddle/implementation.rs` | EMA | 14 |
| `backtest/simulator.rs` | SMA | configurable |
| `volatility_duration_analyzer.rs` | EMA | 3 |
| `straddle_scoring.rs` | Simple H-L (pas un vrai TR) | N/A |

**Correction** :
- Créer un module `services/atr.rs` unique avec :
  - `calculate_true_range(candle, prev_close) -> f64`
  - `calculate_atr_sma(candles, period) -> f64`
  - `calculate_atr_ema(candles, period) -> f64`
- Faire pointer tous les modules vers ce module unique
- Corriger `straddle_scoring.rs` pour utiliser le vrai True Range
- Documenter dans le README quelle variante est utilisée où et pourquoi

**Test de validation** :
- Test unitaire vérifiant que les 3 fonctions donnent des résultats cohérents sur un même jeu de données
- Vérifier que `straddle_scoring.rs` n'utilise plus H-L simple

---

### 2.2 🔶 Unifier les 3 systèmes de timeout

**Situation actuelle** :
| Source | Range | Contexte |
|--------|-------|----------|
| `straddle_adjustments.rs` | [18, 32] min | Ajustement whipsaw |
| `straddle_parameter_service.rs` | [2, 12] min | Paramètres généraux |
| `simple_analyzers.rs` (decay) | 18/25/32 min | Profil de décroissance |

**Correction** :
- Définir un système unique de calcul du timeout avec une logique claire :
  1. Si decay profile disponible → utiliser le timeout dérivé du decay (le plus rigoureux)
  2. Sinon si half-life disponible → `clamp(half_life, min, max)`
  3. Sinon → fallback heuristique basé sur ATR + noise
- Harmoniser les ranges min/max : un timeout de 2 min est irréaliste pour un Straddle event-driven, un minimum de 10-15 min est plus raisonnable
- Documenter la cascade de priorité en commentaire

---

### 2.3 🔶 Unifier la définition de "Whipsaw"

**Situation actuelle** : 3 définitions incompatibles
| Module | Définition |
|--------|-----------|
| `global_analyzer_straddle_calc.rs` | Volatilité faible (< 30% de la moyenne) |
| `movement_analyzer.rs` | Deux mouvements directionnels consécutifs (sans vérifier la direction) |
| `whipsaw_detector.rs` | Simulation réelle SL/TP |

**Correction** :
- Adopter la définition standard trading : **un whipsaw est un mouvement qui déclenche un côté du straddle puis reverse pour toucher le SL** (c'est ce que fait `whipsaw_detector.rs`)
- Corriger `global_analyzer_straddle_calc.rs` : un mouvement faible n'est pas un whipsaw, c'est un "non-event". Renommer en `non_event_rate`
- Corriger `movement_analyzer.rs` : vérifier le **signe** du mouvement entre deux fenêtres consécutives — ne compter comme whipsaw que si la direction change

---

### 2.4 🔶 Unifier le scoring frontend/backend

**Situation actuelle** : 2 fonctions de scoring divergentes côté frontend
| Fonction | Fichier | Méthode |
|----------|---------|---------|
| `calculateStraddleScore()` | `straddleCalculators.ts` | Scoring en %, seuils relatifs |
| `calculateSliceScore()` | `hourlyTableUtils.ts` | Scoring en valeur absolue, seuils hardcodés Forex |

**Correction** :
- Supprimer `calculateSliceScore()` de `hourlyTableUtils.ts`
- Utiliser uniquement `calculateStraddleScore()` partout
- Ou mieux : utiliser le `quality_score()` calculé côté backend (Rust) et ne pas recalculer côté frontend
- S'assurer que les seuils sont relatifs (en %, pas en valeur absolue) pour fonctionner avec tous les actifs

---

### 2.5 🔶 Corriger le Noise Ratio pour les dojis (body=0)

**Fichier** : `src-tauri/src/services/slice_metrics_analyzer.rs`
**Impact** : MOYENNE — Un doji (body=0) retourne noise_ratio=0, alors qu'un doji est un signal de bruit maximal

**Correction** :
```rust
let noise_ratio = if body_size < f64::EPSILON {
    // Doji = 100% bruit
    range / pip_value  // ou une valeur conventionnelle haute (ex: 10.0)
} else {
    (upper_wick + lower_wick) / body_size
};
```

---

### 2.6 🔶 Corriger le Volume Imbalance (faux calcul)

**Fichier** : `src-tauri/src/services/slice_metrics_analyzer.rs`
**Ligne** : ~133
**Impact** : MOYENNE — `volume_imbalance` est un simple duplicata de `body_range / 100`. Ce n'est pas un vrai indicateur de flux

**Correction** :
- Option A (honnête) : Renommer en `direction_strength` ou `body_ratio` et supprimer la prétention de "volume imbalance"
- Option B (complète) : Implémenter un vrai volume imbalance si les données volume sont disponibles : `(vol_up - vol_down) / (vol_up + vol_down)`
- Mettre à jour les labels UI correspondants
- Mettre à jour `FormulasModal` et `SpreadCostTable` si la formule y est documentée

---

### 2.7 🔶 Corriger le ratio Risk/Reward trompeur

**Fichier** : `src-tauri/src/services/straddle_parameter_service.rs`
**Fonction** : Calcul R/R
**Impact** : MOYENNE — Calculé comme `ATR / SL`, sera toujours < 1.0 (puisque SL > 2×ATR). Le Hard TP = 2×SL, donc le R/R réel est ≈ 2.0, pas < 1.0

**Correction** :
```rust
// R/R devrait refléter le vrai potentiel
let risk_reward_ratio = hard_tp_pips / stop_loss_pips; // = 2.0 par construction
```
- Ou si on veut un R/R dynamique, calculer `expected_move / SL` avec `expected_move` basé sur le profil de volatilité

---

## PHASE 3 — INTÉGRITÉ DES DONNÉES (Fondations solides)

> Sans données fiables, tous les calculs sont bâtis sur du sable.

---

### 3.1 🔧 DatabaseLoader doit utiliser le pool Diesel

**Fichier** : `src-tauri/src/services/database_loader.rs`
**Impact** : HAUTE — Le loader ouvre des connexions rusqlite ad-hoc à chaque appel, ignorant le pool r2d2 et ses PRAGMA configurés

**Correction** :
- Utiliser le `DbPool` reçu au constructeur pour exécuter les requêtes
- Supprimer l'ouverture manuelle de connexion `rusqlite::Connection::open()`
- Si des requêtes raw SQL sont nécessaires, utiliser `diesel::sql_query()` ou `diesel::RunQueryDsl`
- Vérifier que WAL mode et busy_timeout ne sont plus redondamment configurés à chaque requête

---

### 3.2 🔧 Supprimer le doublon `pair_importer.rs`

**Fichiers** :
- `src-tauri/src/commands/pair_importer.rs` — INSERT simple, crash sur doublon
- `src-tauri/src/commands/pair_data/processor.rs` — UPSERT, robuste

**Correction** :
- Supprimer `pair_importer.rs`
- S'assurer que toutes les commandes Tauri pointent vers `processor.rs`
- Supprimer l'enregistrement de l'ancienne commande dans `lib.rs`
- Vérifier que `pair_metadata.row_count` utilise `SELECT COUNT(*)` (version processor) et non le cumul incorrect

---

### 3.3 🔧 Corriger `calendar_import_id = 0` hardcodé

**Fichier** : `src-tauri/src/services/economic_event_loader.rs`
**Impact** : L'intégrité référentielle est violée si `foreign_keys = ON`

**Correction** :
- Créer ou récupérer un `calendar_import` valide avant l'import
- Passer le vrai `calendar_import_id` lors de l'insertion des événements
- Ou, si l'import est "libre" (sans calendrier parent), désigner un import par défaut en base (id=1, name="default")

---

### 3.4 🔧 Nettoyer le schéma Diesel fantôme

**Fichier à supprimer** : `src-tauri/src/db/schema.rs` (déclare `ohlc_data`, table inexistante)
**Fichier à garder** : `src-tauri/src/schema.rs` (vrai schéma)

**Correction** :
- Supprimer `src-tauri/src/db/schema.rs` ou le vider
- Vérifier qu'aucun import ne pointe vers l'ancien
- Lancer `diesel print-schema` et comparer avec `schema.rs` pour détecter d'autres divergences

---

### 3.5 🔧 Optimiser le chargement des archives

**Fichier** : `src-tauri/src/services/archive_service.rs`
**Impact** : `list_archives()` charge tous les `data_json` blob en mémoire

**Correction** :
- Créer une variante `list_archives_light()` qui exclut `data_json` (projection SELECT sur les colonnes nécessaires)
- Charger `data_json` uniquement quand l'utilisateur ouvre une archive spécifique
- Ajouter une pagination si le nombre d'archives devient élevé (> 100)

---

### 3.6 🔧 Borner le cache en mémoire

**Fichier** : `src-tauri/src/services/cache_service.rs`
**Impact** : Le `HashMap` du cache croît sans limite → fuite mémoire sur sessions longues

**Correction** :
- Ajouter un `max_entries: usize` au `CacheService`
- À chaque `set()`, si la taille dépasse le max, évincer l'entrée la plus ancienne (LRU simple)
- Valeur par défaut recommandée : 500 entrées

---

### 3.7 🔧 Unifier le système de migrations

**Situation** : Diesel CLI migrations + `db/migrations.rs` avec des `CREATE TABLE IF NOT EXISTS` manuels → doublons et conflits

**Correction** :
- Choisir UN système : Diesel CLI (recommandé pour le long terme)
- Migrer les `CREATE TABLE IF NOT EXISTS` de `migrations.rs` vers de vraies migrations Diesel
- Garder `migrations.rs` uniquement pour les `ALTER TABLE ADD COLUMN` de compatibilité (anciennes bases)
- Documenter la procédure de migration dans le README

---

## PHASE 4 — DUPLICATION FRONTEND/BACKEND (Source de vérité unique)

> Le frontend ne doit PAS recalculer ce que le backend fournit déjà.

---

### 4.1 🔄 Supprimer les recalculs frontend redondants

**Duplications identifiées** :
| Logique | Frontend (à supprimer) | Backend (source de vérité) |
|---------|----------------------|--------------------------|
| Scoring straddle | `calculateStraddleScore()`, `calculateSliceScore()` | `quality_score()` dans `Stats15Min` |
| Plan de trading (SL/TP) | `calculateTradingPlan()` | `straddle_parameters` dans `Stats15Min` |
| Estimation de prix | `estimatePrice()`, `obtenirPrixEstime()` | `point_value` dans `AnalysisResult` |
| Durée de trade | `calculateTradeDuration()` | `recommended_trade_expiration_minutes` |
| Conversion pts/pips | `pipConverter.ts` (partiel) | `AssetProperties::normalize()` |

**Correction** :
- Utiliser les valeurs du backend (`straddle_parameters`, `quality_score`, `point_value`) telles quelles dans l'UI
- Supprimer `calculateTradingPlan()`, `calculateTradeDuration()`, `calculateSliceScore()`, `estimatePrice()`
- Garder `pipConverter.ts` uniquement pour le formatage d'affichage (pas pour le calcul)
- Adapter les composants Vue pour lire depuis le store au lieu d'appeler les fonctions utilitaires

---

### 4.2 🔄 Corriger `estimatePrice()` pour les paires JPY (en attendant 4.1)

**Fichier** : `src/utils/straddleCalculators.helpers.ts`
**Impact** : HAUTE — Retourne `1.0` pour TOUT le Forex, erreur de facteur 150× pour USDJPY

**Correction provisoire** (si 4.1 n'est pas encore fait) :
```ts
function estimatePrice(slice: Stats15Min, symbol?: string): number {
  if (symbol?.includes('JPY')) return 150.0
  if (symbol?.includes('XAU') || symbol?.includes('GOLD')) return 2000.0
  if (slice.atr_mean > 1000) return 100000 // crypto
  if (slice.atr_mean > 10) return 10000    // indices
  return 1.10 // forex majors
}
```
**Correction définitive** : utiliser `point_value` du backend (cf. 4.1)

---

### 4.3 🔄 Supprimer le type `CalendarEvent` dupliqué

**Fichiers** : `src/types/cleanup.ts` et `src/stores/volatilityTypes.ts` déclarent chacun un type `CalendarEvent`

**Correction** :
- Garder une seule définition dans `src/stores/volatilityTypes.ts`
- Importer depuis ce fichier dans `cleanup.ts`

---

## PHASE 5 — AMÉLIORATION DU SCORING ET DE LA CONFIANCE

> Rendre les scores et recommandations réellement exploitables.

---

### 5.1 📊 Adapter les seuils de scoring aux classes d'actifs

**Problème** : Les seuils absolus du HourlyTable (`range > 0.0025`, `ATR > 0.002`) sont calibrés pour EURUSD uniquement. Pour XAUUSD (range ~1.5) ou BTCUSD (range ~500), les TOP 3 sont faux.

**Correction** :
- Normaliser en pips AVANT le scoring (c'est déjà fait côté backend, mais pas côté frontend `calculateSliceScore`)
- Ou exprimer tous les seuils en % du prix (`atr_pct > 0.15%` au lieu de `atr > 0.002`)
- Ou déléguer entièrement le scoring au backend (cf. Phase 4.1)

---

### 5.2 📊 Rendre le TOP 3 réactif au changement de symbole

**Fichier** : `src/components/HourlyTable.vue`
**Impact** : Le TOP 3 est calculé une fois au `onMounted` et jamais mis à jour

**Correction** :
- Transformer le TOP 3 en `computed` ou ajouter un `watch` sur `props.stats15min` / `props.symbol`
- Ou le calculer côté backend et l'envoyer dans `AnalysisResult`

---

### 5.3 📊 Ajouter le nombre d'occurrences dans la heatmap

**Fichier** : `src/components/HeatmapTable.vue`
**Impact** : L'utilisateur voit une moyenne de volatilité sans savoir si elle est basée sur 2 ou 200 occurrences

**Correction** :
- Afficher `N=X` en petit sous la valeur de chaque cellule
- Ou en tooltip au hover
- Griser ou hachurer les cellules avec N < 5 (statistiquement non fiable)

---

### 5.4 📊 Implémenter les fonctionnalités "fantômes" du GlobalAnalyzer

**Fichier** : `src-tauri/src/services/global_analyzer.rs`
**Champs hardcodés à implémenter** :
- `total_days_analyzed: 0` → calculer la vraie durée couverte par les archives
- `event_impacts: vec![]` → peupler avec les impacts des événements depuis les archives
- `most_frequent_recommendation: "Scalp Prudent"` → calculer dynamiquement depuis les archives
- `avg_volatility: 0.0` dans Golden Hours → calculer la vraie moyenne de volatilité par heure

---

### 5.5 📊 Documenter toutes les constantes magiques

**Constantes non documentées identifiées** :
| Constante | Valeur | Fichier | À documenter |
|-----------|--------|---------|-------------|
| Diviseur score volatilité | `0.8` | `straddle_scoring.rs` | Pourquoi 80% = score max ? |
| Coefficient trailing stop | `1.59` | `straddle_adjustments.rs` | Origine ? Calibrage ? |
| Normalisation ATR timeout | `8.0` | `straddle_adjustments.rs` | Pourquoi 8 pips = max ? |
| Floor directionnalité | `0.4 + 0.6 ×` | `straddle_scoring.rs` | Pourquoi garder 40% min ? |
| P95 margin | `× 1.1` | `straddle_parameter_service.rs` | Pourquoi 10% de marge ? |
| Noise simultaneous boost | `× 1.2` | `straddle_simultane_calculator.rs` | Pourquoi +20% ? |
| Giant Doji ATR | `15.0 pips` | `volatility_heuristics.rs` | Calibré sur quoi ? |

**Correction** :
- Pour chaque constante, ajouter un commentaire `// Calibré sur X données / Justification : Y`
- Idéalement, les extraire dans un fichier `services/constants.rs` centralisé
- Ajouter des tests de régression pour vérifier que modifier une constante a l'effet attendu

---

## PHASE 6 — AMÉLIORATIONS UX (Fiabilité perçue)

> Un outil fiable qui affiche mal ses données perd la confiance du trader.

---

### 6.1 🎨 Corriger `UnitDisplay` : gestion NaN et robustesse

**Fichier** : `src/components/UnitDisplay.vue`

**Corrections** :
- Ajouter un garde-fou `isNaN(value) || !isFinite(value)` → afficher "N/A"
- Supprimer le fallback `pointsPerPip = 10` quand le symbole est absent → afficher en unité brute sans conversion
- Ajouter une assertion en dev mode si la valeur reçue n'est pas en pips (ex: valeur > 10000 pour du Forex)

---

### 6.2 🎨 Corriger `isOpenRef` non réactif dans MetricsAnalysisModal

**Fichier** : `src/components/MetricsAnalysisModal.vue`
**Impact** : `ref(props.isOpen)` capture la valeur initiale mais ne suit pas les changements

**Correction** :
```ts
// Remplacer
const isOpenRef = ref(props.isOpen)
// Par
const isOpenRef = computed(() => props.isOpen)
// Ou utiliser toRef
const isOpenRef = toRef(props, 'isOpen')
```

---

### 6.3 🎨 Enrichir la graduation de la heatmap

**Fichier** : `src/components/HeatmapTable.vue`
**Impact** : Seulement 3 niveaux de couleur → discrimination insuffisante

**Correction** :
- Passer à 5-6 niveaux : `very-low`, `low`, `medium`, `high`, `very-high`, `extreme`
- Ou utiliser un gradient continu CSS basé sur la valeur numérique (plus précis)

---

### 6.4 🎨 Corriger les problèmes mineurs UI

| # | Problème | Fichier | Correction |
|---|----------|---------|-----------|
| 1 | Typo `"✨ IAnalyse"` | `GlobalAnalysisModal.vue` | Supprimer le "I" |
| 2 | Code mort `showImportHub`, `showEventCorrelation` | `App.vue` L48-49 | Supprimer les refs inutilisées |
| 3 | Code mort `formatNumber`, `isBestHour` | `HourlyTable.vue` | Supprimer |
| 4 | Imports morts `MetricsGrid`, `VolatilityDurationSection` | `MetricsAnalysisModal.vue` | Supprimer |
| 5 | Spinners incohérents (emoji vs CSS) | Plusieurs | Uniformiser sur un composant `<Spinner>` |
| 6 | Labels mixtes FR/EN dans le backtest | `BacktestConfigPanel.vue` | Tout mettre en français |
| 7 | ConversionTable : valeurs `$` à "0" | `ConversionTable.vue` | Remplir les vraies valeurs |
| 8 | Titre vide de MetricsAnalysisModal | `MetricsAnalysisModal.vue` | Ajouter un titre descriptif |

---

### 6.5 🎨 Scinder les composants trop volumineux

| Composant | Lignes actuelles | Limite | Action |
|-----------|-----------------|--------|--------|
| `HourlyTable.vue` | ~732 | 250 | Extraire `HourlyRow.vue`, `QuarterRow.vue`, `Top3Badge.vue` |
| `RetroactiveAnalysisView.vue` | ~344 | 250 | Extraire la logique `resolveEventType()` dans un composable |
| `straddle/implementation.rs` | ~424 | 300 | Extraire `simulate_trade_outcome()` dans un fichier séparé |

---

## PHASE 7 — AMÉLIORATIONS DE FOND (Excellence)

> Passer de "fiable" à "excellent" pour un usage professionnel.

---

### 7.1 🚀 Ajouter une validation Monte Carlo

**Objectif** : Tester la robustesse statistique des paramètres recommandés

**Implémentation** :
- Pour chaque créneau identifié, simuler N=1000 tirages aléatoires avec bootstrap sur les données historiques
- Calculer l'intervalle de confiance à 95% pour le win rate et le P&L
- Afficher l'IC dans l'UI : "Win Rate: 62% [55%-69%]"
- Si l'IC est trop large (> 20pp), afficher un warning

---

### 7.2 🚀 Gérer le spread dynamique en conditions de news

**Problème** : Le spread est constant dans les simulations. En réalité, il triple ou quintuple pendant les publications de news.

**Implémentation** :
- Ajouter un `spread_multiplier_event: f64` dans `TradingCostProfile` (ex: 3.0× pour les majeures, 5.0× pour les exotiques)
- Appliquer le multiplicateur pendant la fenêtre [T-1min, T+5min] dans le backtest
- Recalculer le P&L net avec le spread réaliste

---

### 7.3 🚀 Modéliser le slippage asymétrique

**Problème** : Le slippage est symétrique et constant. En réalité, il est beaucoup plus élevé à l'ouverture (event-driven) qu'à la fermeture.

**Implémentation** :
- `entry_slippage` = 2× à 3× le slippage normal pendant les 2 premières minutes post-event
- `exit_slippage` = slippage normal
- Appliquer ces valeurs dans le simulateur de Straddle et de backtest

---

### 7.4 🚀 Ajouter la corrélation inter-paires

**Problème** : Si un trader trade EURUSD + GBPUSD sur le même événement, l'exposition est doublée (corrélation ~0.85)

**Implémentation** :
- Calculer la matrice de corrélation glissante (20 jours) entre les paires
- Afficher un warning si deux paires dans le plan de trading sont corrélées > 0.7
- Optionnel : proposer un "diversification score" pour le portefeuille de Straddles

---

### 7.5 🚀 Implémenter `apply_time_adjustment()` pour les multiplicateurs SL

**Fichier** : `src-tauri/src/services/straddle_multipliers.rs`
**Situation** : La fonction existe avec des fuseaux horaires définis mais n'est jamais appelée (tests `#[ignore]`)

**Implémentation** :
- Activer l'ajustement du SL en fonction de l'heure UTC (sessions Tokyo/London/NY)
- Connecter au `StraddleParameterService`
- Écrire et activer les tests

---

### 7.6 🚀 Ajouter `<KeepAlive>` sur les onglets principaux

**Fichier** : `src/App.vue`
**Impact** : Les vues lourdes (Heatmap, Backtest, Rétrospective) sont détruites à chaque changement d'onglet

**Correction** :
```vue
<KeepAlive :include="['HeatmapView', 'BacktestView', 'RetroactiveView']">
  <component :is="currentTabComponent" />
</KeepAlive>
```

---

### 7.7 🚀 Backtester les paramètres recommandés (pas seulement des params fixes)

**Problème** : Le backtest utilise des paramètres configurés manuellement. Il ne teste pas les paramètres dynamiques recommandés par le système.

**Implémentation** :
- Ajouter un mode "Auto" au backtest qui utilise les `straddle_parameters` calculés pour le créneau choisi
- Comparer les résultats "Auto" vs "Manuel" pour valider la pertinence des recommandations
- Afficher un score de fiabilité : "Les paramètres recommandés auraient donné +X% vs vos paramètres manuels"

---

## PHASE 8 — NETTOYAGE TECHNIQUE (Dette à zéro)

---

### 8.1 🧹 Supprimer tout le code mort identifié

- `showImportHub`, `showEventCorrelation` dans `App.vue`
- `formatNumber`, `isBestHour` dans `HourlyTable.vue`
- Imports `MetricsGrid`, `VolatilityDurationSection` dans `MetricsAnalysisModal.vue`
- `TradeOutcome::RecoveryWin` et `DoubleLoss` dans `backtest/models.rs` (jamais produits)
- `bidi_calculator.rs` (fichier vide marqué "remplacé")
- `get_upcoming_events` (retourne toujours un vecteur vide)
- `db/schema.rs` (schéma fantôme `ohlc_data`)

---

### 8.2 🧹 Passer tous les checks qualité

```bash
# Tous ces scripts doivent passer au vert
cargo test
cargo clippy -- -D warnings
./scripts/check-file-size.sh
./scripts/check-unwrap.sh
./scripts/check-architecture.sh
./scripts/check-dead-code.sh
./scripts/check-vue-quality.sh
./scripts/check-typescript-quality.sh
./scripts/check-frontend-quality.sh
```

---

### 8.3 🧹 Dédupliquer `MetricsAnalysisModal`

**Fichiers** : instanciée dans `App.vue` ET `AnalysisPanel.vue`

**Correction** : Ne garder qu'une seule instance (dans App.vue) et communiquer via un event bus ou le store pour l'ouvrir depuis AnalysisPanel

---

## RÉSUMÉ EXÉCUTIF

| Phase | Items | Impact sur fiabilité | Effort estimé |
|-------|-------|---------------------|---------------|
| **Phase 1** — Bugs critiques | 6 | +15% → ~75% | 2-3 jours |
| **Phase 2** — Incohérences calcul | 7 | +10% → ~85% | 3-4 jours |
| **Phase 3** — Intégrité données | 7 | +5% → ~90% | 2-3 jours |
| **Phase 4** — Duplication FE/BE | 3 | +3% → ~93% | 2 jours |
| **Phase 5** — Scoring & confiance | 5 | +2% → ~95% | 2-3 jours |
| **Phase 6** — UX | 5 | Confiance perçue | 2 jours |
| **Phase 7** — Excellence | 7 | +3% → ~98% | 5-7 jours |
| **Phase 8** — Nettoyage | 3 | Maintenabilité | 1 jour |
| **TOTAL** | **43 items** | **60% → 98%** | **~20-25 jours** |
