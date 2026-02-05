#+ Liste priorisée d'améliorations (objectif: excellence & confiance totale)

## P0 — Fiabilité mathématique & cohérence des moteurs (bloquant pour confiance)
1. **Unifier les moteurs Straddle** : un seul moteur de simulation/résultats pour `offset`, `win_rate`, `whipsaw`, `SL/TP`, coûts et timeout, utilisé par toutes les vues (volatilité, rétrospective, backtest). Éliminer les divergences de logique entre simulateurs.
2. **Supprimer les hypothèses contradictoires** : standardiser les fenêtres temporelles (ex: 15 min vs 60 min), la définition d’un trade, et les règles de déclenchement/timeout.
3. **Normaliser toutes les mesures en pips/points** : garantir que chaque métrique est calculée dans une unité unique, puis convertie pour l’affichage uniquement.
4. **Aligner l’offset “optimal”** : choisir une seule méthode (ATR+noise ou P95 wicks) et justifier par validation empirique. Documenter la formule exacte.
5. **Corriger les cas morts** : supprimer les branches logiques jamais atteintes (ex: états de trade non générés) et aligner les résultats aux sorties possibles.
6. **Uniformiser l’intégration des coûts** (spread/slippage) dans tous les calculs (offset, win rate, whipsaw, backtest) avec un profil de coûts unique.

### Ajouts P0 actés (stratégie & timeout)
- **Supprimer la “stratégie directionnelle”** en tant que mode d’exécution : conserver uniquement la stratégie **simultanée**.
- **Conserver la directionnalité comme métrique de qualité** (diagnostic), mais supprimer tout calcul/affichage des paramètres Bidi associés.
- **Backtest uniquement en simultané** : supprimer toute voie “directionnelle” dans le backtest.
- **Timeout non figé** : le dériver dynamiquement de la demi‑vie/volatilité (avec bornes min/max), pas de valeur en dur unique.
- **Conversion pips/points** : **conserver la règle actuelle pour l’instant** (stabilité à court terme).

## P1 — Validation scientifique & robustesse (nécessaire pour confiance élevée)
7. **Mettre en place une validation out‑of‑sample** : séparation stricte training/validation par périodes (ex: 70/30 ou rolling walk‑forward).
8. **Ajouter des tests de stabilité** : variabilité des résultats selon fenêtre, période et granularité.
9. **Introduire des métriques d’incertitude** : intervalle de confiance, erreur standard, taille d’échantillon minimale par événement.
10. **Définir des critères d’arrêt** : refuser de produire un score/reco si l’échantillon est insuffisant ou instable.
11. **Backtest aligné “event‑driven”** : backtest basé sur timestamps réels des annonces (pas uniquement bougies régulières).

## P2 — Qualité des données (prérequis pour résultats fiables)
12. **Validation stricte des imports** : détection des trous de données, doublons, outliers, UTC vs local, et anomalies de timeframe.
13. **Détection automatique d’événements incohérents** : évènements sans bougies associées, doublons d’événements, ou événements trop rares.
14. **Audit de complétude** : rapport de couverture par paire, par année, par événement.
15. **Versioning des données** : tracer la version des fichiers importés et répercuter dans les archives.

## P3 — Architecture & cohérence logicielle (excellence du code)
16. **Uniformiser l’accès DB** : éviter le mélange Diesel / rusqlite non contrôlé. Utiliser un seul mode avec pool unique.
17. **Tracer les dépendances d’analyse** : un “pipeline” unique documenté (imports → cleaning → metrics → décisions).
18. **Centraliser les constantes métier** : seuils (noise ratio, breakout, ATR, etc.) dans une configuration versionnée.
19. **Supprimer la logique métier dans l’UI** : toute logique de calcul doit être côté Rust, l’UI ne fait que l’affichage.
20. **Logs déterministes** : journaliser versions, paramètres, et hypothèses utilisées pour chaque analyse.

## P4 — UX & lisibilité analytique (confiance utilisateur)
21. **Afficher la provenance des métriques** : dans chaque panneau, indiquer formule/paramètres/période.
22. **Indicateurs de fiabilité visuels** : badges “Échantillon faible”, “Hors stabilité”, “Non validé”.
23. **Comparer plusieurs périodes** : afficher la variabilité des résultats entre années pour déceler les changements de régime.
24. **Unifier les libellés et glossaire** : éviter les termes ambigus (risk, confidence, whipsaw, etc.).

## P5 — Tests & assurance qualité (excellence technique)
25. **Tests unitaires systématiques sur métriques clés** (ATR, noise ratio, whipsaw, win rate, offset).
26. **Tests d’intégration end‑to‑end** : import CSV → analyse → archive → export PDF.
27. **Tests de non‑régression** : snapshots des résultats pour un set de données connu.
28. **Benchmarks de performance** : mesurer coût des analyses volumineuses (pairs multi‑années).

## P6 — Documentation & gouvernance (durabilité)
29. **Documenter chaque métrique** : définition, raison d’être, limites, exemples.
30. **Guide de validation trader** : procédure claire pour valider un setup avant trading réel.
31. **Journal des changements** : suivre chaque modification de formule et son impact.

---

### Critère “Confiance totale” (définition)
L’application peut être jugée “fiable” uniquement si :
- un seul moteur de calcul est utilisé partout,
- les résultats sont stables out‑of‑sample,
- les données sont validées et complètes,
- chaque métrique fournit son niveau d’incertitude,
- la décision finale peut être auditée avec la traçabilité complète.

---

## Avancement — Itération 1 (Straddle simultané uniquement)
- Suppression des affichages et composants directionnels (UI + rétrospectif).
- Backtest basculé en simultané unique (frontend + backend), suppression du mode.
- Rapports PDF “Bidi” simplifiés au simultané.
- Paramètres rétrospectifs directionnels retirés des types et du payload.
- Nettoyage des types/utilitaires Bidi obsolètes.

## Avancement — Itération 2 (Timeout dérivé)
- Timeout Straddle dérivé de la demi‑vie si disponible, sinon basé sur ATR/Noise.
- Paramètres rétrospectifs alignés sur le simultané uniquement.

## Avancement — Itération 3 (Terminologie & exports)
- Terminologie “Bidi” remplacée par “Straddle Simultané” dans les PDF.
- Nettoyage des mentions Bidi résiduelles côté UI.

## Avancement — Itération 4 (Exports Straddle)
- Exports PDF et UI d’export alignés sur Straddle Simultané.

## Avancement — Itération 5 (Unification moteur Straddle)
- Win rate Straddle calculé via le moteur `simulate_straddle` (offset fixé possible).
- Nettoyage du module `services/volatility/win_rate_calculator` (supprimé).
- Références `simulate_straddle` mises à jour côté tests.

## Avancement — Itération 6 (Formules Straddle)
- Formules et notes alignées sur le mode simultané uniquement.

## Avancement — Itération 7 (Nettoyage terminologie)
- Commentaires backend alignés sur “Straddle simultané”.

## Avancement — Itération 8 (Rétrospectif)
- Calculateur renommé en `StraddleSimultaneCalculator`.

## Avancement — Itération 9 (Coûts backtest)
- Slippage intégré au backtest (config + simulation + PDF).

## Avancement — Itération 10 (Offset unifié)
- Offset Straddle unifié sur le P95 des mèches récentes (+ coûts), fallback ATR si P95 indisponible.

## Avancement — Itération 11 (Fenêtre de simulation)
- Fenêtre de simulation Straddle alignée sur le timeout ajusté (ATR).

## Avancement — Itération 12 (Backtest déclenchement)
- Backtest aligné sur un déclenchement par offset (NoEntry/Timeout cohérents).

## Avancement — Itération 13 (Règles de sortie)
- Backtest aligné sur TP/SL (TP = 2x offset), trailing stop retiré pour cohérence.

## Avancement — Itération 14 (Coûts/Whipsaw)
- Coûts et perte whipsaw du simulateur alignés sur la logique backtest (2 jambes).

## Avancement — Itération 15 (Règles de sortie simulateur)
- Simulateur Straddle aligné sur TP/SL (gestion LOSS explicite).

## Avancement — Itération 16 (Timeout PnL)
- PnL timeout du simulateur aligné sur un exit prix (coûts inclus).

## Avancement — Itération 17 (Backtest offset/TP)
- Offset backtest reconfigurable, trailing stop retiré (TP = 2x offset).

## P0 — Statut
- ✅ P0 terminé (unification moteur, règles de sortie, coûts, timeout, offset).

