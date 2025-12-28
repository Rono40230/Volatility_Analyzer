# 🛠️ Plan de Finalisation : Intégration des Coûts Réels (Spread/Slippage)

Ce document détaille les étapes finales pour aligner la logique backend (Rust) avec le tableau de référence des coûts (Frontend).
**Objectif** : Remplacer les valeurs par défaut (+3.0 pips) par les vrais coûts par paire, sans casser l'existant.

## 🔴 Priorité 1 : Infrastructure des Coûts (Backend)
*Créer la source de vérité pour les Spreads et Slippages.*

- [ ] **Créer le module `TradingCosts`** (`src-tauri/src/models/trading_costs.rs`)
    - [ ] Définir une structure `TradingCostProfile` (spread_min, spread_max, slippage).
    - [ ] Implémenter une fonction `get_profile(symbol: &str)` qui retourne le profil selon le tableau de référence :
        - **Majors Liquides** (EURUSD, USDJPY): Spread 2.5, Slippage 1.0
        - **Majors Volatiles** (GBPUSD, AUDUSD): Spread 4.0, Slippage 2.0
        - **Minors/Crosses** (GBPJPY, EURJPY): Spread 6.5, Slippage 3.0
        - **Gold** (XAUUSD): Spread 4.0, Slippage 2.0
        - **Indices US** (US30, NAS100): Spread 7.5, Slippage 5.0
        - **Indices EU** (DAX40): Spread 6.0, Slippage 3.0
        - **Crypto** (BTCUSD): Spread 40.0, Slippage 20.0
        - **Défaut**: Spread 3.0, Slippage 1.0

## 🟠 Priorité 2 : Mise à jour du Calculateur (Service)
*Intégrer ces coûts dans les formules mathématiques.*

- [ ] **Modifier `StraddleParameterService::calculate_parameters`**
    - [ ] Ajouter un argument `symbol: &str` à la signature.
    - [ ] Supprimer l'argument `spread_margin: Option<f64>` (devenu obsolète).
    - [ ] Récupérer le profil via `TradingCosts::get_profile(symbol)`.
    - [ ] **Appliquer les formules corrigées** :
        - `Offset = (ATR * Multiplier) + Spread + Slippage` (Actuellement Slippage ignoré).
        - `Stop Loss = (ATR * Multiplier) + Slippage` (Pour garantir l'exécution réelle).
        - `SL Recovery = Stop Loss * 1.2` (Impacté indirectement).

## 🟡 Priorité 3 : Propagation dans les Commandes (API)
*Mettre à jour les appelants pour passer le symbole.*

- [ ] **Mettre à jour `volatility/straddle_analysis.rs`**
    - [ ] Passer le `symbol` lors de l'appel à `calculate_parameters`.
- [ ] **Mettre à jour `retrospective_analysis/bidi_calculator.rs`**
    - [ ] Passer le `symbol` (déjà disponible dans le contexte ou à ajouter).
- [ ] **Mettre à jour `volatility/quarterly_aggregator.rs`**
    - [ ] Passer le `symbol`.

## 🔵 Priorité 4 : Vérification & Non-Régression
*S'assurer que tout fonctionne comme avant, mais avec des valeurs plus justes.*

- [ ] **Vérifier la compilation** (`cargo check`).
- [ ] **Vérifier qu'aucune régression n'apparaît dans les tests unitaires**.
- [ ] **Validation Visuelle** : Comparer les résultats "Volatilité Brute" pour EURUSD vs GBPJPY (le GBPJPY doit avoir un Offset/SL mécaniquement plus large à cause du spread).
