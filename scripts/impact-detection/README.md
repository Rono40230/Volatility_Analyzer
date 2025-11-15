# 🎯 Système d'Impact Detection - Mode Opératoire

## Vue d'ensemble

Ce système gère les **2 phases** du développement avec verification d'impact :

```
PHASE 1 : ACCUMULATION (Tu codes, l'IA accumule)
  ├─ IA génère code
  ├─ IA teste chaque modification
  ├─ Accumule les changements
  └─ PAS de commit

PHASE 2 : VALIDATION (Quand tu dis "valide tout")
  ├─ Prend un snapshot du code
  ├─ Vérifie impact (qui a changé)
  ├─ Détecte régressions (teste tout)
  ├─ Génère rapport complet
  └─ OK ou BLOQUE le commit
```

---

## Phase 1 : Accumulation (Automatique)

**Durée** : Quelques changements/jours

### Ce que l'IA fait :
1. Génère code selon ta demande
2. Teste immédiatement (tests doivent passer)
3. **N'accumule pas en mémoire** (fichiers réels modifiés)
4. Attend ta validation avant Phase 2

### Ce que tu fais :
- Tu dis ce que tu veux
- L'IA code + teste
- Tu lui dis "continue" ou "valide tout"

---

## Phase 2 : Validation (À ton signal)

**Tu dis** : "Valide tout et prépare le commit"

**L'IA exécute** :
```bash
./scripts/impact-detection/validate-phase2.sh
```

### Étape 1 : Vérifier l'impact
**Script** : `verify-impact.sh`
- Compare snapshot initial vs code courant
- Détecte fichiers modifiés (attendus)
- Détecte fichiers affectés transitifs (attendus)
- Détecte fichiers cassés inattendus (ALERTE)

**Rapport** : `impact-report.txt`

### Étape 2 : Détecter les régressions
**Script** : `regression-detector.sh`
- Lance `cargo test --all`
- Compare avec baseline (tests avant changements)
- Détecte tests cassés (RÉGRESSION = ALERTE)
- Ignore tests cassés au départ

**Rapport** : `regression-report.txt`

### Étape 3 : Décision Finale
**Script** : `final-approval.sh`
- Agrège tous les rapports
- Lance vérifications .clinerules
- Affiche rapport complet

**Résultat** :
- ✅ APPROUVÉ : L'IA peut commit
- ❌ BLOQUÉ : Problèmes détectés

---

## Initialisation (Au départ)

**Une seule fois, avant de commencer** :

```bash
./scripts/impact-detection/init-impact-system.sh
```

Cela :
1. Crée le dossier `.git/.snapshots/`
2. Nettoie les snapshots anciens
3. Prend le snapshot initial (baseline)

---

## Workflow Exemple

### Jour 1 (Lundi) - PHASE 1 COMMENCE
```bash
# L'IA exécute (automatiquement au départ)
./scripts/impact-detection/init-impact-system.sh

Toi : "Ajoute une colonne 'last_sync' à la table pairs"
IA : Génère migration + tests
IA : cargo test → 150/150 ✅
IA : Accumule

Toi : "Continue, améliore le chargement CSV"
IA : Modifie csv_loader.rs
IA : cargo test → 150/150 ✅
IA : Accumule
```

### Jour 5 (Vendredi) - PHASE 2
```bash
Toi : "Valide tout et commit"
IA : Exécute :

  ./scripts/impact-detection/validate-phase2.sh
  
  ├─ verify-impact.sh
  │   → 3 fichiers modifiés ✅ (attendu)
  │   → 0 fichiers affectés inattendus ✅
  │
  ├─ regression-detector.sh
  │   → Tests avant : 150 passés, 0 échoués
  │   → Tests après : 150 passés, 0 échoués
  │   → Aucune régression ✅
  │
  └─ final-approval.sh
      → Tous les contrôles ✅
      → APPROUVÉ POUR COMMIT ✅

IA : Commit + push
Toi : Valide sur GitHub
```

---

## Fichiers & Dossiers

### Scripts
```
scripts/impact-detection/
├─ init-impact-system.sh      ← À lancer au départ
├─ snapshot-dependencies.sh   ← Prend un snapshot
├─ verify-impact.sh           ← Vérifie les changements
├─ regression-detector.sh     ← Détecte régressions
├─ final-approval.sh          ← Rapport final + décision
├─ validate-phase2.sh         ← Orchestre Phase 2 complète
└─ change-tracker.sh          ← Enregistre les changements (optionnel)
```

### Snapshots (dans `.git/.snapshots/`)
```
.git/.snapshots/
├─ pre-phase2-state-1731564000.json   ← Snapshot initial (json)
├─ impact-report.txt                  ← Rapport d'impact
└─ regression-report.txt              ← Rapport de régression
```

---

## Comprendre les Rapports

### Impact Report
```
Fichiers inchangés   : 250 ✅
Fichiers modifiés    : 3 (à tester)
Modules transitifs   : 5 (à vérifier)
```

### Regression Report
```
BASELINE (avant changements)
  Tests passés : 150
  Tests échoués : 0
  
RÉSULTATS ACTUELS
  Tests passés : 150
  Tests échoués : 0
  
ANALYSE
  Nouveaux cassés : 0 ✅
  Nouveaux réparés : 0
  
STATUS : ✅ AUCUNE RÉGRESSION
```

---

## Blocages Possibles

### ❌ Impact Détecte un Fichier Cassé Inattendu
```
Modules affectés (INATTENDUS) :
  - calendar_events.rs (n'aurait pas dû changer)
```
**Solution** : Revérifier les changements, peut-être une dépendance cachée

### ❌ Régression Détectée
```
STATUS : ❌ RÉGRESSION DÉTECTÉE
Test cassé : test_csv_loading ... FAILED
```
**Solution** : Corriger le code, re-tester, relancer Phase 2

### ❌ Vérification .clinerules Échouée
```
❌ Taille fichiers
❌ unwrap() détecté
```
**Solution** : Appliquer les corrections auto (format, dead code), corriger manuellement les unwrap()

---

## Cas Spéciaux

### Si tu veux relancer Phase 2 sans changer le code
```bash
./scripts/impact-detection/validate-phase2.sh
```

### Si tu veux voir juste le rapport d'impact
```bash
./scripts/impact-detection/verify-impact.sh
cat .git/.snapshots/impact-report.txt
```

### Si tu veux relancer juste les tests
```bash
cd src-tauri && cargo test --release
```

### Si tu veux réinitialiser (recommencer Phase 1)
```bash
./scripts/impact-detection/init-impact-system.sh
```

---

## Limitations & Notes

- ✅ Snapshot simple (une seule à la fois, écrasée après commit)
- ✅ Tests comparés avec baseline (ignore tests cassés au départ)
- ✅ Détecte modules transitifs affectés
- ⚠️ Ne détecte PAS les dépendances implicites (si mal documentées)
- ⚠️ Si >100 changements accumulés, mieux vaut committer plus souvent

---

## Résumé pour l'IA

**RÈGLES ABSOLUES** :

1. **PHASE 1** : Accumule, ne commit pas
2. **PHASE 2** : Exécute `validate-phase2.sh` quand l'utilisateur dit "valide"
3. **Blocages** : Si Phase 2 échoue, signale précisément pourquoi
4. **Commit** : Seulement si Phase 2 = ✅ APPROUVÉ
5. **Réinitialisation** : Si nouveau développement, re-exécute init

✅ Ce système assure **0 régression**, **0 dette technique**, **documentation d'impact complète**.
