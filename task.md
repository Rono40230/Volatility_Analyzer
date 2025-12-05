# 📋 Roadmap Analyses Historiques - IAnalyse Module

**Dernier update**: 5 décembre 2025  
**État**: Phase 1 (Accumulation) - 36 heures de travail planifiées  
**Responsable**: Rono40230

---

## 🎯 Vue d'ensemble

Ce document liste **toutes les tâches nécessaires** pour améliorer le module **IAnalyse** (analyse statistique globale par IA). Les tâches sont **triées par priorité et complexité**, avec estimations de temps réalistes.

**Objectif principal**: Transformer IAnalyse en un outil complet de prise de décision stratégique pour le Straddle trading, couvrant:
- ✅ Statistiques globales et fiabilité
- ✅ Recommandations intelligentes (Optimal/Good/Cautious/Risky)
- ✅ Tableau de bord des risques
- ✅ Analyse des "heures en or" (Golden Hours)
- ✅ Détection automatique des opportunités et outliers
- ✅ Heatmaps interactives corrélation paire/événement
- ✅ Analyse prédictive des tendances

---

## 🔴 PHASE 1: PRIORITÉ CRITIQUE (P1) - 8 heures

Tâches essentielles pour la fonctionnalité de base du module IAnalyse.

### P1.1 - Système de Recommandations Intelligentes (2h)

**Description**: Implémenter un système intelligent qui génère des recommandations (`Optimal | Good | Cautious | Risky`) basées sur:
- Confiance globale (> 75 = Optimal, 50-75 = Good, etc.)
- Volatilité vs événements (corrélation)
- Nombre minimum d'analyses (< 5 = Risky)

**Fichiers affectés**:
- `src-tauri/src/services/global_analysis_recommendation.rs` (NEW)
- `src-tauri/src/commands/global_analysis.rs` (modify)

**Critères de validation**:
- [ ] Enum `TradingRecommendation` implémenté
- [ ] Score de confiance intégré (0-100)
- [ ] Logique de recommandation testée
- [ ] Tests unitaires > 80% coverage

---

### P1.2 - Tableau de Bord des Risques (3h)

**Description**: Créer un dashboard visuel montrant les **risques détectés** (bruit élevé, volatilité extrême, événements non-corrélés, etc.).

**Composant Vue**: `src/components/global/RiskDashboard.vue`

**Métriques à afficher**:
- 🔴 Noise Ratio > 3.0 (trop de bruit)
- 🟡 Volatilité > 25% (risque élevé)
- 🔵 Événements sans corrélation (mauvaise tradabilité)
- 🟢 Opportunités détectées (combinaisons gagnantes)

**Intégration**: Dans GlobalAnalysisModal, nouvel onglet "🎯 Risques & Opportunités"

**Critères de validation**:
- [ ] Composant RiskDashboard créé (< 200 lignes)
- [ ] Métriques de risque calculées côté Rust
- [ ] UI montrant alertes visuelles
- [ ] Responsive sur Tauri 1400×900

---

### P1.3 - Analyse des Heures en Or (Golden Hours) - Améliorée (3h)

**Description**: Enrichir le système existant de Golden Hours avec:
- Score de fiabilité par heure (0-100%)
- Événements associés à chaque heure
- Performance réelle vs prédiction

**Fichiers affectés**:
- `src-tauri/src/services/golden_hours_analyzer.rs` (NEW)
- `src/components/global/DashboardGrid.vue` (enhance)

**Calcul**:
```
Fiabilité(h) = (Nombre de gains à h / Nombre d'opérations à h) × 100
Score(h) = Fiabilité(h) × Volatilité(h) × (1 - NoiseRatio(h)/5)
```

**Critères de validation**:
- [ ] Service `golden_hours_analyzer` implémenté
- [ ] Tests avec données réelles (30+ jours)
- [ ] Graphique des heures mises à jour
- [ ] Insight IA généré automatiquement

---

## 🟠 PHASE 2: PRIORITÉ HAUTE (P2) - 12 heures

Tâches importantes pour la profondeur analytique.

### P2.1 - Scoring Multi-Critères pour Paires (4h)

**Description**: Créer un **scoring global par paire** (0-100) combinant:
- Win rate du straddle
- Volatilité moyenne
- Stabilité (inverse du bruit)
- Corrélation événement/mouvement

**Fichiers affectés**:
- `src-tauri/src/services/pair_scoring.rs` (NEW)
- `src-tauri/src/commands/global_analysis.rs` (modify)

**Formula**:
```
Score(pair) = (WinRate × 0.4) + (VolatilityScore × 0.3) + (StabilityScore × 0.2) + (CorrelationScore × 0.1)
```

**Output**: Array `PairScore { symbol, score, reasoning }`

**Critères de validation**:
- [ ] Service `pair_scoring` testé (> 80% coverage)
- [ ] Scoring reproduisant les résultats attendus
- [ ] Raisons du score expliquées en français

---

### P2.2 - Matrice de Corrélation Interactive (4h)

**Description**: Créer une **heatmap interactive** montrant la corrélation entre paires et événements économiques.

**Composant Vue**: `src/components/global/EventCorrelationMatrix.vue`

**Données**:
```
Rows: Paires (EURUSD, GBPUSD, etc.)
Cols: Événements (NFP, CPI, BOE, etc.)
Cell: Score de corrélation (0-100%)
```

**Features**:
- Hover = tooltip avec détails
- Click sur cell = drill-down vers les analyses détaillées
- Filtrable par date/période
- Export CSV

**Critères de validation**:
- [ ] Heatmap affichée correctement
- [ ] Calcul de corrélation validé (Pearson ou Spearman)
- [ ] Performance < 2 secondes pour 50×20 cellules
- [ ] Responsive

---

### P2.3 - Détection Automatique des Outliers & Anomalies (4h)

**Description**: Implémenter un système qui détecte automatiquement les **valeurs aberrantes** (outliers) et anomalies dans les données.

**Fichiers affectés**:
- `src-tauri/src/services/anomaly_detection.rs` (NEW)
- `src/components/global/AnomalyAlerts.vue` (NEW)

**Détection**:
- Volatilité > 3σ (écart-type)
- Win rate anormal (trop haut/bas)
- Bruit extrême
- Sessions avec comportement différent

**Output**: Alertes visuelles + explications textuelles

**Critères de validation**:
- [ ] Détection d'outliers implémentée
- [ ] Seuils réalistes (3σ validé sur données réelles)
- [ ] Tests avec données synthétiques + réelles

---

## 🟡 PHASE 3: PRIORITÉ MOYENNE (P3) - 16 heures

Tâches d'amélioration et d'optimisation.

### P3.1 - Heatmap Avancée avec Filtres Temps (5h)

**Description**: Améliorer la heatmap avec filtres avancés:
- Filtrer par **jour de la semaine** (Mon-Sun)
- Filtrer par **session de trading** (Asie, Europe, NY)
- Filtrer par **type d'événement** (économique, géopolitique, etc.)
- Timeline interactive

**Fichiers affectés**:
- `src/components/global/EventCorrelationHeatmap.vue` (enhance)
- `src-tauri/src/services/heatmap_filters.rs` (NEW)

**Critères de validation**:
- [ ] Filtres intégrés et fonctionnels
- [ ] Performance maintenued < 1.5s
- [ ] UX claire avec labels explicitifs

---

### P3.2 - Analyse des Tendances à Long Terme (5h)

**Description**: Analyser les **tendances** des métriques clés sur le temps:
- Volatilité en hausse/baisse?
- Win rate s'améliore-t-il?
- Nouvelles corrélations apparaissent-elles?

**Composant Vue**: `src/components/global/TrendAnalysis.vue`

**Graphiques**:
- Line chart: Volatilité moyenne par mois
- Bar chart: Win rate tendance
- Sparklines: Corrélation événement/mouvement

**Critères de validation**:
- [ ] Graphiques tracés correctement
- [ ] Données lissées (moving average 7j / 30j)
- [ ] Prédictions simples (trend direction)

---

### P3.3 - Clustering de Sessions Similaires (6h)

**Description**: Utiliser un algorithme de clustering (K-means ou DBSCAN) pour **grouper les sessions similaires**.

**Fichiers affectés**:
- `src-tauri/src/services/session_clustering.rs` (NEW)
- `src/components/global/SessionClusterView.vue` (NEW)

**Groupes**:
- Haute volatilité + Haut bruit
- Basse volatilité + Stable
- Événement-driven + Corrélation forte
- etc.

**Output**: 
- Nombre de clusters (K auto-détecté)
- Membership par session
- Caractéristiques de chaque cluster

**Critères de validation**:
- [ ] Clustering implémenté (Polars + Rust)
- [ ] Silhouette score > 0.5 (qualité)
- [ ] Clusters interprétables

---

## 🔵 PHASE 4: PRIORITÉ BASSE (P4) - Nice-to-have

Tâches optionnelles / améliorations futures.

### P4.1 - Export Multi-Format (1.5h)
- [ ] PDF report (charts + tables)
- [ ] Excel workbook (multi-sheets)
- [ ] JSON API pour intégration Bidi robot

### P4.2 - Notifications & Alertes (2h)
- [ ] Alerte quand une paire franchit son score critique
- [ ] Notification changement Golden Hour
- [ ] Email digest quotidien

### P4.3 - Comparaison Historique (2h)
- [ ] Comparer 2 périodes (semaine A vs semaine B)
- [ ] Voir progression/régression

### P4.4 - Prédictions Simple (2.5h)
- [ ] Regress: Volatilité future basée sur tendance
- [ ] Recommended paires pour demain (basé sur pattern)

### P4.5 - Benchmarking (1h)
- [ ] Comparer vos metrics vs "moyenne du marché"
- [ ] Ranking: Où vous situez-vous?

---

## 📊 Estimations Globales

| Priorité | Heures | Nombre de tâches | Complexité |
|----------|--------|------------------|-----------|
| 🔴 CRITIQUE (P1) | 8 | 3 | Haute |
| 🟠 HAUTE (P2) | 12 | 3 | Haute |
| 🟡 MOYENNE (P3) | 16 | 3 | Moyenne |
| 🔵 BASSE (P4) | 9 | 5 | Basse |
| **TOTAL** | **45** | **14** | **Mixte** |

**Timeline réaliste**: ~1.5 mois (travail partiel, 2-3 sessions/semaine)

---

## ✅ Workflow d'exécution

### Phase 1 (CRITIQUE - 1 semaine)
1. **P1.1** Système de recommandations (2h)
2. **P1.2** Tableau de bord risques (3h)
3. **P1.3** Golden Hours enrichies (3h)
4. ✅ Validation + Tests + Commit

### Phase 2 (HAUTE - 2 semaines)
1. **P2.1** Scoring multi-critères (4h)
2. **P2.2** Matrice corrélation (4h)
3. **P2.3** Détection outliers (4h)
4. ✅ Validation + Tests + Commit

### Phase 3 (MOYENNE - 3 semaines)
1. **P3.1** Heatmap avancée (5h)
2. **P3.2** Analyse tendances (5h)
3. **P3.3** Clustering sessions (6h)
4. ✅ Validation + Tests + Commit

### Phase 4 (BASSE - Au-delà)
- Features optionnelles selon priorité produit

---

## 🔧 Standards Techniques

### Rust Backend (`src-tauri/src/`)
- ✅ Services < 300 lignes
- ✅ Erreurs = `Result<T, VolatilityError>`
- ✅ Tests: > 80% coverage
- ✅ Pas d'`unwrap()` (sauf tests)
- ✅ Nommage FRANÇAIS

### Vue Frontend (`src/components/`)
- ✅ Composants < 250 lignes (300 pour modals)
- ✅ Pas de `console.log()`, `alert()`, `any`
- ✅ TypeScript explicite
- ✅ Styles scoped + responsive (Tauri 1400×900)
- ✅ Nommage FRANÇAIS

### Tests & QA
- ✅ `cargo test` doit passer
- ✅ Audit: `make check`
- ✅ Validation Phase 2: `./scripts/impact-detection/validate-phase2.sh`

---

## 📝 Notes Importantes

1. **Pas de backtesting**: L'app analyse le **passé**, pas de simulation future
2. **Traçabilité**: Chaque métrique doit avoir une **raison explicable**
3. **Seuils réalistes**: Validés sur données réelles (30+ jours minimum)
4. **User-friendly**: UI simple, labels clairs, pas de jargon technique
5. **Performance**: Analyses < 5 secondes, import < 30 secondes

---

## 🎓 Ressources Utiles

| Ressource | Lien |
|-----------|------|
| .clinerules | Règles du projet |
| copilot-instructions.md | Guide Copilot |
| SYSTEM_PROMPT.md | Phase 1/2 workflow |
| projet.md | Objectifs business |

---

**Auteur**: AI (GitHub Copilot)  
**Maintenance**: Rono40230  
**Dernière mise à jour**: 5 décembre 2025
