# 🔍 Audit des Calculs de Volatilité Observée

**Date:** 14 novembre 2025  
**Fichier Source:** `src-tauri/src/commands/correlation/pair_correlation_helpers.rs`  
**Fonction:** `calculate_event_volatility_for_pair()` (lignes 39-165)

---

## 📊 Vue d'ensemble

Les trois métriques de volatilité observée sont calculées pour chaque occurrence d'un événement économique:

| Métrique | Fenêtre | Requête SQL | Unité |
|----------|---------|-------------|-------|
| **Volatilité -30mn** | 30min avant l'événement | `time <= event_time AND time > datetime(event_time, '-30 minutes')` | Pips |
| **Volatilité +30mn** | 30min après l'événement | `time > event_time AND time < datetime(event_time, '+30 minutes')` | Pips |
| **Volatilité 1h total** | 1h complète autour (±30min) | `time > datetime(event_time, '-30 minutes') AND time < datetime(event_time, '+30 minutes')` | Pips |

---

## ✅ Points de Cohérence (FIABLES)

### 1. **Fenêtres temporelles bien définies**
```
Timeline de l'événement:

        -30min                EVENT TIME              +30min
          |                      |                      |
    [BEFORE WINDOW]           ×                  [AFTER WINDOW]
    30 candles M1            OCCURS              30 candles M1
    
    [============ TOTAL WINDOW (60 candles M1) ============]
```

✅ **Logique cohérente:**
- Fenêtre BEFORE: Isolée AVANT l'événement (baseline de comparaison)
- Fenêtre AFTER: Isolée APRÈS l'événement (réaction du marché)
- Fenêtre TOTAL: Union des deux (comportement global)

### 2. **Calcul du volatilité en pips**
```sql
SELECT AVG(high - low) as avg_vol
FROM candle_data
WHERE ...
```

La volatilité est calculée comme:
- **Amplitude (high - low) de chaque M1** → averagée sur la fenêtre
- **Résultat direct = pips**
- **Multiplication × 100** appliquée ensuite pour normalisation

✅ **C'est la bonne approche:**
- `high - low` = amplitude réelle en prix
- Moyenne sur les 30 (ou 60) candles M1
- Unité naturelle = pips

### 3. **Agrégation par événement**
Pour chaque événement qui a **plusieurs occurrences** dans l'historique:

```rust
let mut volatilities_before = Vec::new();  // Accumule les volatilités de chaque occurrence
// ...
let avg_before = volatilities_before.iter().sum::<f64>() 
                 / volatilities_before.len() as f64;  // Moyenne des occurrences
```

✅ **C'est judicieux:**
- Un événement peut se reproduire 10, 20, 50 fois
- Afficher la moyenne des volatilités observées = comportement typique
- Évite d'être biaisé par une occurrence exceptionnelle

### 4. **Limite LIMIT 30/60**
```sql
LIMIT 30   -- Pour -30min ou +30min
LIMIT 60   -- Pour la fenêtre totale (±30min)
```

✅ **Cohérent avec M1:**
- 30 candles M1 = 30 minutes (1 minute par bougie)
- 60 candles M1 = 60 minutes (1 heure complète)

### 5. **Filtre des données nulles**
```rust
if let Some(vol) = vol_before {
    if vol > 0.0 {
        volatilities_before.push(vol * 100.0);  // Enregistre seulement si > 0
    }
}
```

✅ **Bonne pratique:**
- Évite les zéros problématiques (gap/fermé du marché)
- Moyenne = résultat fiable car basée sur vraies données
- Flag `has_data_found` indique si des données existent

### 6. **Vérification préalable des candles**
```rust
if !has_candles_for_event(candle_index, symbol, event_dt) {
    continue;  // Skip si aucune bougie disponible pour cet événement
}
```

✅ **Protection contre:**
- Événements sans données de trading
- Paires non disponibles à cette date
- Heures de fermeture/weekend

---

## ⚠️ Points Critiques à Examiner

### 1. **ATTENTION: Fenêtres chevauchantes**

```
Événement à 10:00:00

BEFORE:  09:30:00 → 10:00:00  (inclusive à 10:00)
         └─ `time <= event_time AND time > datetime('-30 min')`
         
AFTER:   10:00:00 → 10:30:00  (exclusive à 10:00)
         └─ `time > event_time AND time < datetime('+30 min')`
         
TOTAL:   09:30:00 → 10:30:00  (inclut 10:00)
         └─ `time > datetime('-30 min') AND time < datetime('+30 min')`
```

**Analyse:**
- ✅ BEFORE et AFTER **ne se chevauchent PAS** (BEFORE inclut 10:00, AFTER l'exclut)
- ✅ TOTAL = BEFORE + AFTER (plus une possible bougie exactement à 10:00)
- ⚠️ **Assertion à vérifier:** `AVG(BEFORE) + AVG(AFTER) ≈ AVG(TOTAL)` ?
  - Non exactement, car les LIMITS (30 vs 30) pourraient exclure/inclure différemment
  - Mais logiquement cohérent

### 2. **ATTENTION: Conversion × 100**

```rust
volatilities_before.push(vol * 100.0);  // vol est déjà en pips (high - low)
```

**Question:** Pourquoi multiplier par 100 ?

- La BD stocke volatilité en **format décimal** (ex: 0.0050 = 0.5 pips)
- `high - low` depuis les candles = valeur brute
- `× 100` = conversion pour affichage en "centimes de pips" ou normalisation

✅ **Valide SI:**
- Les données candles sont en format décimal (0.xxxx)
- On veut afficher en format "pips" standard (1 pip = 0.0001 pour paires majeurs)

⚠️ **À Vérifier:** 
- Quelle est l'**unité réelle** des colonnes `high` et `low` en DB ?
- Sont-elles en **prix brut** (1.2345) ou **decimal normalized** (0.00001) ?

### 3. **Limite LIMIT 30/60 peut être insuffisante**

```sql
WHERE ... LIMIT 30   -- Seulement 30 résultats
```

**Problème potentiel:**
- Si 150 candles M1 existent en -30min, seules les 30 **dernières** sont prises
- Biaise vers la fin de la fenêtre, pas vraiment une moyenne représentative

✅ **Mais peut-être intentionnel:**
- Privilégie les candles les **plus proches** de l'événement (plus pertinent)
- Évite un bruit excessif loin de l'événement

### 4. **Pas de zone de transition**

```
-30min AVANT         ×ÉVÉNEMENT         +30min APRÈS
[AVANT]              │                   [APRÈS]
```

**Observation:**
- Il n'y a **aucun délai** entre AVANT et APRÈS
- L'événement est souvent **annoncé à une seconde précise**
- Effet économique = réaction instantanée vs volatilité progessive ?

✅ **Logiquement OK si:**
- Données M1 sont suffisament granulaires pour capter l'impact immédiat
- Le marché réagit en < 1 minute pour annonces

⚠️ **Pourrait être amélioré avec:**
- Zone tampon de ±5 secondes (exclue des fenêtres)
- Fenêtre +5min plutôt que +30min pour mieux isoler l'effet

---

## 📐 Vérification Mathématique

Pour l'événement **USD Non-Farm Payroll** (exemple):

```
Occurrences: 12 (dernier an)

Occurrence #1 (2024-11-08 13:30):
  - vol_before = 8.5 pips
  - vol_after = 12.3 pips
  - vol_total = 10.4 pips

Occurrence #2 (2024-10-11 13:30):
  - vol_before = 7.2 pips
  - vol_after = 10.1 pips
  - vol_total = 8.6 pips

... (12 occurrences)

MOYENNE AFFICHÉE:
  - avg_before = SUM(all 12 befores) / 12
  - avg_after = SUM(all 12 afters) / 12
  - avg_total = SUM(all 12 totals) / 12
```

**Test de cohérence:**
```
Attendu: avg_total ≈ (avg_before + avg_after) / 2
Raison: 1 heure = 30min avant + 30min après

Exemple: 
  avg_before = 8.0
  avg_after = 11.0
  avg_total = ? 
  
  Logiquement: 9.0 ± 0.5 (avec chevauchement possible)
```

✅ **À vérifier:** Exporter un événement et calculer manuellement

---

## 🎯 Conclusion

### **Verdict: FIABLE ET LOGIQUE (avec réserves)**

| Aspect | Verdict | Confiance |
|--------|---------|-----------|
| Fenêtres temporelles | ✅ Bien séparées | 95% |
| Calcul en pips | ✅ Correct | 85% |
| Agrégation par événement | ✅ Moyenne valide | 90% |
| Unité de mesure | ⚠️ À confirmer | 70% |
| Limites de résultats | ⚠️ Peut biaiser | 75% |
| Absence de délai transition | ✅ OK pour M1 | 80% |

### **Recommandations d'Amélioration**

1. **Documente l'unité des colonnes `high`/`low` en DB**
   - Ajouter commentaire dans le code

2. **Teste la cohérence BEFORE + AFTER vs TOTAL**
   - Export logs en debug et vérifiez manuellement

3. **Envisage un paramètre configurable pour les fenêtres**
   - 30min peut ne pas être optimal pour tous les actifs

4. **Ajoute une validation sur les résultats**
   - Flag si `vol_total < (vol_before + vol_after) / 2` (incohérence)

5. **Trace les occurrences exclues**
   - Afficher en log combien d'occurrences ont été skippées (pas de données)

---

## 📝 Résumé pour l'UI

Les valeurs affichées dans le tableau "Par Paire" représentent:

- **Volatilité observée -30mn:** Amplitude moyenne en pips durant les 30 min **avant** l'événement
- **Volatilité observée +30mn:** Amplitude moyenne en pips durant les 30 min **après** l'événement  
- **Volatilité observée 1h total:** Amplitude moyenne en pips sur **l'heure complète** (±30min)

**Chaque valeur est la moyenne de TOUTES les occurrences** de cet événement dans l'historique.

Exemple: USD NFP s'est produit 12 fois → ces chiffres = moyenne des 12 fois.

