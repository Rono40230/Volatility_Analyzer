# Tâche : Mise à jour des Formules (Bidi V2)

## Contexte
Les calculs du backend (Rust) ont évolué vers la logique "Bidi V2" (paramètres adaptatifs basés sur le Noise Ratio), mais la documentation frontend (`src/data/formules.ts`) affiche encore les anciennes formules statiques.

## Objectifs
1.  Mettre à jour `src/data/formules.ts` pour refléter exactement le code Rust.
2.  Ajouter les nouvelles métriques (SL Recovery, Trailing Stop adaptatif).
3.  Corriger les définitions existantes (Noise Ratio utilise True Range, pas High-Low).

## Étapes

### 1. Mise à jour des définitions de base
- [ ] **Noise Ratio** : Corriger la formule pour utiliser `True Range / |Close - Open|` (au lieu de `Range / Body`).
- [ ] **ATR** : Confirmer la méthode de lissage (Wilder's Smoothing) dans la description.

### 2. Mise à jour des Paramètres Straddle (Bidi V2)
- [ ] **Offset** : Passer de `ATR * 1.75` à la logique adaptative :
    - Base : `ATR * 1.2`
    - Si Noise > 2.0 : `ATR * 1.5`
- [ ] **Stop Loss (SL)** : Ajouter la logique adaptative complète :
    - Base : `ATR * 1.5`
    - Paliers : 1.75x (>1.5), 2.0x (>2.0), 2.5x (>2.5), 3.0x (>3.0).
- [ ] **Trailing Stop** : Ajouter la formule adaptative :
    - Base : `ATR * 0.6`
    - Paliers : 0.8x (>1.5), 1.0x (>2.0), 1.2x (>3.0).
- [ ] **SL Recovery** : Ajouter la nouvelle formule `max(SL, Offset * 3.0)`.
- [ ] **Take Profit** : Mettre à jour pour indiquer qu'il est géré dynamiquement par le Trailing Stop ou définir un "Target Théorique" (Risk:Reward).

### 3. Vérification
- [ ] Vérifier que la modale "Formules" affiche correctement les nouvelles descriptions et exemples.
- [ ] S'assurer que les unités (points, pips) sont cohérentes.

## Références Code (Source de Vérité)
- `src-tauri/src/services/metrics/calculator.rs` (ATR, Noise, Volatility)
- `src-tauri/src/services/straddle_parameter_service.rs` (Offset, SL, TS, Recovery)
