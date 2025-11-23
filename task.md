# 📋 TÂCHES RESTANTES - Optimisation Stratégie Straddle

**Date**: 23 novembre 2025  
**Status**: TÂCHE 1-4 ✅ COMPLÉTÉES  
**Objectif**: Implémenter les améliorations post-TÂCHE 4  
**Priorité**: Haute → Moyenne → Basse

---

## ✅ TÂCHES COMPLÉTÉES (À NE PAS REFAIRE)

### ✅ TÂCHE 1: Supprimer Volume Imbalance
**Status**: FAIT (commit cfac358)  
**Remplacée par**: Direction Strength = (body_range_mean × breakout_percentage) / 100

### ✅ TÂCHE 2: Corriger le Calcul du Stop Loss  
**Status**: FAIT (commit cfac358)  
**Implémentation**: SL adaptatif = ATR × noiseFactor, où noiseFactor = max(0.6, min(0.9, 1.0 - noise_ratio/10))

### ✅ TÂCHE 3: Remplacer Range par True Range
**Status**: FAIT (commit cfac358)  
**Implémentation**: Range → True Range + ATR Wilder's Smoothing

### ✅ TÂCHE 4: Implémenter le Calcul Réel de Durée de Volatilité
**Status**: FAIT (commits 9c508a7, e0551b2, 1010f4b)  
**Implémentation**: ATR Wilder's EMA decay analysis + affichage UI (Peak, Half-Life, Trade Exp)

---

## 🟠 PRIORITÉ HAUTE (Améliorations importantes)

### 🎯 TÂCHE 5: Ajouter Métriques Manquantes Critiques pour Straddle

**STATUS**: 🟡 PARTIELLEMENT RÉALISÉE (Phase 1 ✅ + Phase 2 ✅ + Phase 3 🟠 TODO)

#### Phase 1 ✅ COMPLÉTÉE: Backend Services (Commits: 1c22a29, 1bd0f52)

**5.1 - Offset Optimal** ✅
- Fichier: `src-tauri/src/services/volatility/offset_calculator.rs` (167L)
- Implémentation: P95 percentile des wicks + 10% marge
- Tests: Inclus + validés

**5.2 - Win Rate Simulé** ✅  
- Fichier: `src-tauri/src/services/volatility/win_rate_calculator.rs` (242L)
- Implémentation: Backtest Straddle 15min simulation
- Tests: Inclus + validés

**5.3 - Whipsaw Detection** ✅
- Fichier: `src-tauri/src/services/volatility/whipsaw_detector.rs` (254L)
- Implémentation: Détection double déclenchement (dual breach)
- Risk levels: Very Low/Low/Moderate/High/Very High avec colors
- Tests: Inclus + validés

#### Phase 2 ✅ COMPLÉTÉE: Tauri Commands (Commits: 1c22a29, 1bd0f52)

- Command: `analyze_straddle_metrics(symbol, hour, candles)` ✅
- Registrée dans `invoke_handler` ✅
- Combine les 3 services en un seul call ✅

#### Phase 3 ✅ COMPLÉTÉE: Frontend Composable (Commit: 1bd0f52)

- Fichier: `src/composables/useStraddleAnalysis.ts` (90L)
- Composable: `useStraddleAnalysis()` avec `analyzeStraddleMetrics()` ✅
- Exports: States + Computed colors ✅

#### Phase 4 ✅ COMPLÉTÉE: Frontend Integration (Commit: 1bd0f52)

- Fichier: `src/components/MetricsAnalysisModal.vue` (2145L)
- Sections UI: `.straddle-performance-section` ✅
- Display: 3 metric cards (Offset, Win Rate, Whipsaw) ✅
- Watcher: Appel à `analyzeStraddleMetrics` ✅

#### Phase 5 🟠 TODO: Charger VRAIES Candles

**Bloqueur**: Le watcher passe `emptyCandles[]` - besoin de charger depuis DB

**Options**:
1. Créer command `load_candles_for_hour(symbol, year, month, day, hour)` 
2. Charger via `load_pair_candles` + extraire l'heure
3. Passer depuis analysisResult si disponible

**Prochaine itération**: Implémenter chargement candles réelles (2-3h)
**Objectif**: Calculer la distance minimale pour éviter 95% des fausses mèches  
**Fichiers à créer/modifier**:
- `src-tauri/src/services/volatility/offset_calculator.rs` (nouveau)
- `src/utils/straddleAnalysis.ts` - Ajouter affichage

**Algorithme**:
```rust
pub fn calculate_optimal_offset(candles: &[Candle]) -> f64 {
    // 1. Calculer toutes les mèches (wick size)
    let wicks: Vec<f64> = candles
        .iter()
        .map(|c| {
            let body = (c.close - c.open).abs();
            let upper_wick = c.high - c.close.max(c.open);
            let lower_wick = c.open.min(c.close) - c.low;
            upper_wick.max(lower_wick)
        })
        .collect();
    
    // 2. Calculer le percentile 95
    let mut sorted_wicks = wicks.clone();
    sorted_wicks.sort_by(|a, b| a.partial_cmp(b).unwrap());
**Estimation**: 2-3 heures  
**Validation**: Vérifier que offset calculé > ATR × 0.75 actuel

---

#### 5.2 - Win Rate Simulé
**Objectif**: Backtester des Straddles avec différents offsets pour calculer le win rate réel  
**Status**: ✅ IMPLÉMENTÉ (voir `win_rate_calculator.rs`)

---

#### 5.3 - Fréquence Whipsaw  
**Objectif**: Mesurer le % de fois où les 2 ordres sont déclenchés (perte garantie)  
**Status**: ✅ IMPLÉMENTÉ (voir `whipsaw_detector.rs`)

---

## 🟡 PRIORITÉ MOYENNE (Optimisations)

### 🔄 TÂCHE 6: Fusionner Tick Quality et Body Range
**Problème**: Redondance conceptuelle  
**Impact**: Simplifie l'interface  
**Fichiers à modifier**:
- `src-tauri/src/services/volatility/hourly_stats.rs` - Retirer `tick_quality_mean`
- `src/components/HourlyTable.vue` - Retirer colonne
- `src/components/AnalysisPanel.vue` - Vérifier usage

**Alternative**: Renommer Tick Quality en "Body Size Moyen" si on veut garder la métrique absolue

**Estimation**: 1 heure  
**Validation**: Vérifier que Body Range % suffit pour les analyses

---

### 📐 TÂCHE 7: Améliorer la Formule de Trade Duration
**Problème**: Formule actuelle ignore event_type et hour_of_day  
**Impact**: Durée peut être sous-optimale  
**Fichiers à modifier**:
- `src/utils/straddleAnalysis.ts` - Fonction `calculateTradeDuration()`

**Nouvelle formule**:
```typescript
function calculateTradeDuration(
  atrMean: number,
  eventType: string,
  hourOfDay: number
): number {
  // Base duration from ATR
  let baseDuration = 240; // 4h default
  if (atrMean > 50) baseDuration = 120;
  else if (atrMean > 40) baseDuration = 150;
  else if (atrMean > 25) baseDuration = 180;
  
  // Adjust for event type
  const eventFactors: Record<string, number> = {
    'NFP': 0.5,           // Pic court, intense
    'CPI': 0.7,           // Pic moyen
    'Interest Rate': 0.8, // Pic long
    'GDP': 1.0,           // Pic très long
  };
  const eventFactor = eventFactors[eventType] || 1.0;
  
  // Adjust for hour of day
  const hourFactors: Record<number, number> = {
    8: 0.8,  // London open - pic court
    13: 0.6, // NY open - pic très court
    14: 0.7, // Overlap - pic court
    // Autres heures: 1.0 (normal)
  };
  const hourFactor = hourFactors[hourOfDay] || 1.0;
  
  return Math.round(baseDuration * eventFactor * hourFactor);
}
```

**Estimation**: 2-3 heures  
**Validation**: Comparer durées calculées avec observations empiriques

---

### 🎨 TÂCHE 8: Améliorer l'Affichage des Tooltips des Métriques
**Problème**: Certains tooltips manquent d'exemples concrets  
**Impact**: Utilisateur ne comprend pas bien les métriques  
**Fichiers à modifier**:
- `src/components/AnalysisPanel.vue` - Enrichir tooltips

**Exemple d'amélioration**:
```html
<template #usage>
  <div class="tooltip-section-title">📊 Interprétation</div>
  <div class="tooltip-section-text">
    <strong>Noise Ratio < 2.0</strong> : Excellent (peu de mèches)<br/>
    Exemple: Range 20 pips, Body 15 pips → Noise = 1.33 ✅<br/><br/>
    
    <strong>Noise Ratio 2.0-3.0</strong> : Acceptable<br/>
    Exemple: Range 20 pips, Body 8 pips → Noise = 2.5 ⚠️<br/><br/>
    
    <strong>Noise Ratio > 3.0</strong> : Danger (fausses mèches)<br/>
    Exemple: Range 20 pips, Body 4 pips → Noise = 5.0 ❌
  </div>
</template>
```

**Estimation**: 2 heures  
**Validation**: Tester avec utilisateur final

---

## 🔵 PRIORITÉ BASSE (Nice to have)

### 📊 TÂCHE 9: Ajouter Graphique de Décroissance de Volatilité
**Objectif**: Visualiser comment la volatilité décroît après le pic  
**Impact**: Aide à comprendre la durée optimale  
**Fichiers à créer/modifier**:
- `src/components/VolatilityDecayChart.vue` (nouveau)
- `src/components/MetricsAnalysisModal.vue` - Intégrer graphique

**Estimation**: 4-5 heures  
**Validation**: Graphique doit montrer clairement le pic et la décroissance

---

### 🔍 TÂCHE 10: Ajouter Filtre par Type d'Événement dans Heatmap
**Objectif**: Filtrer heatmap pour voir uniquement NFP, CPI, etc.  
**Impact**: Facilite l'analyse ciblée  
**Fichiers à modifier**:
- `src/components/EventCorrelationHeatmap.vue` - Ajouter dropdown filtre

**Estimation**: 2 heures  
**Validation**: Filtre fonctionne correctement

---

### 📈 TÂCHE 11: Exporter Paramètres Bidi en JSON
**Objectif**: Permettre export direct des paramètres pour le robot  
**Impact**: Automatisation complète  
**Fichiers à créer/modifier**:
- `src/utils/straddleAnalysis.ts` - Fonction `exportBidiConfig()`
- `src/components/BidiParametersModal.vue` - Bouton export

**Format JSON**:
```json
{
  "symbol": "EURUSD",
  "event_name": "NFP",
  "event_time": "2025-11-22T14:29:50Z",
  "entry_offset_pips": 12,
  "stop_loss_pips": 11,
  "take_profit_pips": 33,
  "trailing_stop_multiplier": 2.0,
  "trade_duration_minutes": 150,
  "confidence_score": 78.5,
  "win_rate_estimated": 0.65
}
```

**Estimation**: 2 heures  
**Validation**: JSON valide et importable par robot Bidi

---

## 📝 RÉCAPITULATIF DES PRIORITÉS

| Priorité | Tâches | Estimation Totale |
|----------|--------|-------------------|
| 🟠 **HAUTE** | 5.1, 5.2, 5.3 | 8-10 heures |
| 🟡 **MOYENNE** | 6, 7, 8 | 5-6 heures |
| 🔵 **BASSE** | 9, 10, 11 | 8-9 heures |

**TOTAL ESTIMÉ**: 21-25 heures de développement

---

## ✅ CRITÈRES DE VALIDATION GLOBALE

Avant de considérer l'application "production-ready" :

1. ✅ **Aucune métrique fictive** (Volume Imbalance supprimé)
2. ✅ **Stop Loss adaptatif** (prend en compte Noise Ratio)
3. ✅ **Durée calculée réellement** (pas d'heuristique)
4. ⏳ **Offset optimal calculé** (percentile 95 des mèches)
5. ⏳ **Win Rate affiché** (backtest réel)
6. ⏳ **Whipsaw détecté** (< 20% acceptable)
7. ✅ **Interface claire** (pas de doublons)
8. ⏳ **Export JSON** (paramètres Bidi)

---

## 🚀 ORDRE D'EXÉCUTION RECOMMANDÉ

### Sprint 1 (Haute - 2-3 jours)
- 5.1 Offset Optimal
- 5.2 Win Rate Simulé
- 5.3 Whipsaw Frequency

### Sprint 2 (Moyenne - 1 jour)
- 6 Fusionner Tick Quality
- 7 Améliorer Trade Duration
- 8 Enrichir Tooltips

### Sprint 3 (Basse - 1-2 jours)
- 9 Graphique décroissance
- 10 Filtre événements
- 11 Export JSON

**TOTAL**: 4-6 jours de développement
