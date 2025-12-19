# 📋 Tâches : Correction Critique Conversion Points/Pips

## CONTEXTE
Un audit a révélé une incohérence majeure entre le Backend (qui envoie des données normalisées en **PIPS**) et le Frontend (qui les affiche comme des **POINTS** sans conversion). Cela entraîne une erreur d'un facteur 10 sur les paires Forex (ex: afficher "10 pts" au lieu de "100 pts").

**Objectif** : Harmoniser l'affichage pour garantir que les valeurs en Points sont correctes (x10 pour Forex) afin d'éviter des erreurs de configuration de robot fatales.

---

## 📅 PLAN D'ACTION PRIORISÉ

### 🔴 PRIORITÉ 1 : Cœur du Système d'Affichage
Le composant `UnitDisplay` est le point central de l'erreur. Il doit savoir que la valeur entrante est toujours normalisée (Pips).

- [ ] **Refactor `src/components/UnitDisplay.vue`**
    - [ ] Modifier la logique : Considérer la prop `value` comme étant **toujours** en Pips (source Backend).
    - [ ] Si `unit` est 'pts'/'points' : Calculer `displayValue = value * pointsPerPip`.
    - [ ] Si `unit` est 'pips' : Calculer `displayValue = value`.
    - [ ] Mettre à jour le template pour afficher "X pts" ou "X pips" correctement.

### 🔴 PRIORITÉ 2 : Paramètres de Trading (Bidi)
Les paramètres calculés (Offset, SL, TP) sont critiques pour le robot. Ils doivent être affichés en Points MT5.

- [ ] **Audit & Fix `src/components/metrics/BidiParametersSection.vue`**
    - [ ] Vérifier les valeurs passées aux cartes (`StraddleDirectionalCard`, `StraddleSimultaneousCard`).
    - [ ] S'assurer que les valeurs (Offset, SL, TP) sont converties en Points avant affichage.
- [ ] **Fix `src/components/trading/StraddleDirectionalCard.vue`**
    - [ ] Vérifier l'utilisation de `UnitDisplay` ou le formatage manuel.
    - [ ] Garantir l'affichage "xxx Points".

### 🟠 PRIORITÉ 3 : Tableaux de Données
Vérifier que la correction de `UnitDisplay` se propage correctement sans double conversion.

- [ ] **Vérification `src/components/HourlyTable.vue`**
    - [ ] S'assurer que `atr_mean`, `max_true_range` utilisent bien `UnitDisplay`.
    - [ ] Vérifier l'affichage des colonnes ATR et Max Spike.
- [ ] **Vérification `src/components/metrics/MetricsGrid.vue`**
    - [ ] Vérifier l'affichage des métriques globales.

### 🟡 PRIORITÉ 4 : Archives
Les archives stockent des snapshots JSON. Il faut s'assurer qu'à la relecture, les unités sont respectées.

- [ ] **Fix `src/composables/useArchiveParsers.ts`**
    - [ ] S'assurer que lors du parsing, on ne dénormalise pas accidentellement les valeurs si elles sont déjà stockées en Pips.
    - [ ] Harmoniser l'unité par défaut (`unit: 'pts'` vs `unit: 'pips'`).

### 🟢 PRIORITÉ 5 : Validation Finale
- [ ] **Test Manuel (Scénario EURUSD)**
    - [ ] Charger EURUSD.
    - [ ] Vérifier ATR : Doit être ~10-20 Pips -> Affichage **100-200 pts**.
    - [ ] Vérifier Heatmap : Doit rester cohérente (déjà correcte).
    - [ ] Vérifier Paramètres Bidi : Offset ~15 Pips -> Affichage **150 pts**.

---

## 📝 NOTES TECHNIQUES
- **Backend** : Envoie toujours des PIPS (`AssetProperties::normalize` divise par 0.0001 pour Forex).
- **Frontend** : Doit multiplier par `pointsPerPip` (10 pour Forex) pour obtenir les POINTS.
- **Règle d'Or** : "Afficher en Points, Calculer en Pips".
