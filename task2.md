# 🧪 Plan de Tests UI — Validation des Améliorations (Phases 1-8)

> **Objectif** : Valider visuellement et fonctionnellement toutes les améliorations.
> **Méthode** : Cocher ✅ chaque test réussi, ❌ si échec (noter le détail).
> **Prérequis** : Avoir au moins 1 paire importée (EURUSD recommandé) + des événements calendrier.
> **Ordre** : Du plus critique au moins critique. Faire dans l'ordre.

---

## 🔴 PRIORITÉ 1 — Blocants (si ça plante, rien d'autre ne marche)

### T1. L'application démarre sans erreur
- [x] Fermer et relancer l'application ✅
- [x] Pas d'écran blanc, pas de crash au démarrage ✅
- [x] L'onglet Accueil s'affiche correctement ✅
- [x] La navigation entre tous les onglets fonctionne (cliquer chaque onglet une fois) ✅

### T2. Pas de NaN / Infinity dans la Heatmap
- [x] Charger la heatmap (onglet Heatmap) → vérifier que les scores en points/pips sont des nombres valides ✅
- [x] Aucune cellule ne doit afficher `NaN`, `Infinity`, `-Infinity` ou `undefined` ✅
- [x] Aller dans **Volatilité** → sélectionner EURUSD → vérifier les colonnes ATR, Noise Ratio, Score ✅
- [x] Répéter pour une paire volatile (GBPJPY ou XAUUSD si disponible) ✅

### T3. Pas de division par zéro sur créneaux vides
- [x] Chercher un créneau avec `0 candles` dans la heatmap (ex: week-end, heure creuse) ✅
- [x] Vérifier qu'il affiche des valeurs à 0 ou "—" et non pas `NaN` ou une erreur ✅

---

## 🟠 PRIORITÉ 2 — Nouvelles features (les changements majeurs de cette session)

### T4. ⭐ KeepAlive des onglets
- [x] Aller dans **Heatmap**, charger des données (sélectionner paire + événement) ✅
- [x] Changer d'onglet → aller dans **Backtest** ✅
- [x] Revenir dans **Heatmap** ✅
- [x] **Résultat attendu** : les données sont toujours là, pas de rechargement, pas de spinner ✅
- [x] Répéter avec **Rétrospective** : charger une analyse, switch onglet, revenir → données préservées ✅
- [x] Répéter avec **Backtest** : configurer des paramètres (sans lancer), switch, revenir → les champs sont remplis ✅

### T5. ⭐ Bouton "Params Auto" dans le Backtest
- [ ] Aller dans l'onglet **Backtest**
- [ ] Sélectionner la paire **EURUSD**
- [ ] Cliquer sur **"📚 Params Auto"**
- [ ] Vérifier que les champs se remplissent automatiquement :
  - SL ≈ 13 pips
  - Spread ≈ 2.5
  - Slippage ≈ 1.0
  - Timeout ≈ 20 min
  - TP(R) ≈ 2.0
- [ ] Un badge vert **"✅ Paramètres recommandés appliqués"** doit apparaître
- [ ] Changer de paire (ex: GBPJPY) → le badge disparaît (mode redevient Manuel)
- [ ] Re-cliquer "📚 Params Auto" pour GBPJPY :
  - SL ≈ 33 pips
  - Spread ≈ 6.5
  - Slippage ≈ 3.0
- [ ] Modifier manuellement un champ (ex: SL) → pas de crash
- [ ] Lancer un backtest en mode Auto → les résultats s'affichent normalement
- [ ] **Sans sélectionner de paire** → le bouton doit être grisé/désactivé

### T6. ⭐ Ajustement temporel du Stop Loss
- [x] Ouvrir la heatmap et noter le SL recommandé pour un créneau **03h00 UTC** (heure calme) ✅
- [x] Noter le SL recommandé pour un créneau **14h30 UTC** (heure de news US) ✅
- [x] Le SL à 14h30 devrait être environ **2× plus grand** que celui de 03h00 (ratio 1.5/0.7 ≈ 2.14) ✅
- [x] Le SL à 10h00 UTC (calme) devrait être plus petit que celui de 12h00 UTC (critique) ✅

### T7. ~~⭐ Badge Monte Carlo (Intervalle de Confiance)~~ — SUPPRIMÉ
- [x] ~~Monte Carlo supprimé du code (backend + frontend) — fonctionnalité retirée par décision utilisateur~~

---

## 🟡 PRIORITÉ 3 — Cohérence visuelle & données

### T8. Unité affichée correcte (Pips vs Points vs $)
- [x] EURUSD → doit afficher "pips" partout
- [x] XAUUSD (si dispo) → doit afficher "$" ou "points"
- [x] L'unité doit être cohérente entre la heatmap, la modale d'analyse et le backtest
- [x] XAGUSD corrigé: 1000 → 10 pts/pip (vérifié MT5)

### T9. Noise Ratio réaliste
- [x] Aucun Noise Ratio ne doit être négatif
- [x] Un Noise Ratio > 3.0 est suspect mais possible (vérifier le visuel)
- [x] Les créneaux calmes (nuit) devraient avoir un Noise Ratio plus élevé que les créneaux actifs

### T10. Graduation de couleurs dans la heatmap (6 niveaux)
- [x] La heatmap affiche bien un dégradé de couleurs (pas juste 2-3 couleurs)
- [x] Les meilleures cellules sont clairement distinguées des pires

### T11. Compteur de candles (sample_count) visible
- [x] Chaque cellule de la heatmap devrait indiquer le nombre de candles analysées
- [x] Les cellules avec peu de données (< 10 candles) sont visuellement identifiables

### T12. Guard NaN sur l'affichage des unités
- [x] Aller dans une analyse de créneau
- [x] Vérifier que les valeurs numériques sont bien formatées (pas de `NaN pips`)
- [x] S'il n'y a pas de données, un message vide propre s'affiche

### T13. Détails Quarter (QuarterDetails)
- [x] Cliquer sur un créneau 15min dans la heatmap
- [x] Vérifier que le panneau de détail s'ouvre correctement
- [x] Les métriques affichées sont lisibles et cohérentes

---

## 🟢 PRIORITÉ 4 — Imports & Régression

### T14. Import de paire fonctionne
- [x] Importer (ou ré-importer) un fichier CSV pour une paire
- [x] Vérifier que le compteur de candles correspond
- [x] L'import ne doit pas créer de doublons (UPSERT ON CONFLICT)

### T15. Import calendrier fonctionne
- [x] Importer un fichier calendrier économique
- [x] Vérifier que les événements apparaissent dans le dropdown backtest

### T16. Sélecteur de symboles cohérent
- [x] Le sélecteur de paire apparaît dans : Volatilité, Backtest, Heatmap
- [x] La liste des paires disponibles est la même partout

### T17. Pas de régression backtest après nettoyage
- [x] Lancer un backtest (Event ou Time)
- [x] Les outcomes s'affichent correctement : TakeProfit, StopLoss, Timeout, NoEntry, Whipsaw
- [x] Il ne doit PAS y avoir de "RecoveryWin" ou "DoubleLoss" (supprimés)

### T18. Spread dynamique (validation indirecte)
- [x] Si tu as des screenshots/exports d'avant les changements : comparer les paramètres Straddle
- [x] Les coûts devraient être légèrement plus élevés qu'avant (spread ×3 au lieu de ×1 pendant les news)
- [x] Le score global peut être un peu moins bon (= plus réaliste)

---

## 🔵 PRIORITÉ 5 — Stress & Performance (optionnel)

### T19. Performance heatmap avec beaucoup de données
- [x] Charger une heatmap sur une paire avec > 1 an de données ✅
- [x] Le chargement ne devrait pas prendre plus de ~10 secondes ✅
- [x] Le scroll reste fluide ✅

### T20. Changement rapide d'onglets
- [x] Cliquer rapidement entre Heatmap → Backtest → Rétrospective → Heatmap (5x) ✅
- [x] Pas de crash, pas d'écran blanc, pas de données mélangées ✅

### T21. Backtest avec beaucoup d'événements
- [x] Lancer un backtest sur un événement fréquent (ex: CPI) avec 2+ ans de données ✅
- [x] Le backtest doit se terminer sans timeout ✅
- [x] Les résultats affichent un nombre raisonnable de trades ✅

---

## RÉSUMÉ D'EXÉCUTION

| Priorité | # | Test | Résultat | Notes |
|---|---|---|---|---|
| 🔴 P1 | T1 | Démarrage sans erreur | ✅ | |
| 🔴 P1 | T2 | Pas de NaN heatmap | ✅ | |
| 🔴 P1 | T3 | Division par zéro | ✅ | |
| 🟠 P2 | T4 | KeepAlive onglets ⭐ | ✅ | |
| 🟠 P2 | T5 | Params Auto backtest ⭐ | ⬜ | |
| 🟠 P2 | T6 | SL temporel ⭐ | ✅ | |
| 🟠 P2 | T7 | ~~Badge Monte Carlo~~ | 🗑️ | Supprimé |
| 🟡 P3 | T8 | Unités correctes | ✅ | XAGUSD fix 1000→10 |
| 🟡 P3 | T9 | Noise Ratio réaliste | ✅ | |
| 🟡 P3 | T10 | Graduation couleurs | ✅ | |
| 🟡 P3 | T11 | Sample count visible | ✅ | |
| 🟡 P3 | T12 | Guard NaN affichage | ✅ | |
| 🟡 P3 | T13 | Détails Quarter | ✅ | |
| 🟢 P4 | T14 | Import paire | ✅ | UPSERT anti-doublons OK |
| 🟢 P4 | T15 | Import calendrier | ✅ | |
| 🟢 P4 | T16 | Sélecteur symboles | ✅ | |
| 🟢 P4 | T17 | Régression backtest | ✅ | |
| 🟢 P4 | T18 | Spread dynamique | ✅ | |
| 🔵 P5 | T19 | Performance heatmap | ✅ | |
| 🔵 P5 | T20 | Switch rapide onglets | ✅ | |
| 🔵 P5 | T21 | Backtest gros volume | ✅ | |

**Score final** : 20/21 tests passés (T5 Params Auto = ⬜ deferred)

> 🔴 P1 = Si ça échoue, l'app est cassée → à corriger en urgence
> 🟠 P2 = Nouvelles features de cette session → les plus importants à valider
> 🟡 P3 = Qualité visuelle → peut être toléré temporairement
> 🟢 P4 = Fonctions existantes → régression unlikely mais à vérifier
> 🔵 P5 = Stress → bonus
