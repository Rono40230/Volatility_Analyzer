BLOC 1 : Header + Recommandation (BestSliceCard)
Éléments affichés :

⭐ Rang du créneau (rank 1 = meilleur)
🕒 Heure du meilleur créneau (ex: 14:45-15:00)
📊 Score Straddle (0-100)
🎯 Recommandation textuelle avec position size recommandée
Calcul du Score Straddle :


Score = combinaison pondérée de:  - ATR Mean (volatilité)  - Body Range % (pureté du signal)  - Noise Ratio (stabilité du mouvement)  - Breakout % (cassures)  - Volume Imbalance (force directionnelle)
Seuils d'interprétation :

≥75 + confiance ≥50% → ✅ EXCELLENTES (position 75-100%)
60-75 + confiance ≥30% → ⚠️ ACCEPTABLES (position 50-75%)
<60 → ❌ INSUFFISANTES (skip)
BLOC 2 : Métriques détaillées (MetricsGrid)
6 métriques de base affichées côte à côte :

ATR Moyen (Average True Range 14 périodes)

Valeur: {atr_mean} converti en % du prix estimé
Mesure la volatilité moyenne du créneau
Calcul: Moyenne des ATR historiques pour ce créneau horaire
Volatilité Moyenne

Valeur: {volatility_mean}
Écart-type empirique des mouvements intra-period
Basée sur les closes historiques du créneau
Range Moyen

Valeur: {range_mean} (high - low)
Distance totale parcourue par les bougies
Calculé sur tout l'historique du créneau
Body Range %

Valeur: {body_range_mean} en pourcentage
(Close - Open) / Range × 100
Indique la pureté directionnelle (sans les queues)
45% = très pur, <25% = très bruité

Noise Ratio (ShadowRatio)

Valeur: {noise_ratio_mean}
(High-Low) / (Close-Open) ou équivalent
Mesure les rejets et fausses touches
<1.5 = stable, >2.5 = chaotique
Breakout %

Valeur: {breakout_percentage}
Pourcentage de fois où le price casse un support/résistance
Indique la force du mouvement
Chaque métrique affiche :

Valeur du créneau 15min analysé
Valeur globale moyenne (comparatif)
Code couleur : 🟢 Excellent / 🔵 Bon / 🟡 Acceptable / 🔴 Pauvre
BLOC 3 : Qualité du Mouvement (MovementQualitySection)
Éléments affichés :

📊 Score Qualité (0-100)

Notation globale combinant: Trend Score + Smoothness + Consistance des bougies
80 = Excellent | 60-80 = Bon | <40 = Faible

🏷️ Label Qualité (texte descriptif)

Ex: "EXCELLENT", "BON", "FAIBLE"
Appréciation qualitative du mouvement
Calcul : Issu du command Rust analyze_slice_metrics qui retourne movement_quality_score et movement_quality_label

BLOC 4 : Durée de Volatilité (VolatilityDurationSection)
4 métriques temps-dépendantes :

Durée du Pic ({peak_duration_minutes} min)

Temps où la volatilité > 80% du pic observé
Calcul: Empirique basé sur ATR, Range, Body Range
Exemple: NFP = 90-150min, données faibles = 150-270min
Demi-Vie de Volatilité ({volatility_half_life_minutes} min)

Temps pour décroissance exponentielle à 50% du pic
Basée sur: Noise Ratio + stabilité du mouvement
NR<1.5 (stable) → demi-vie longue (70% du pic)
NR>2.5 (chaotique) → demi-vie courte (30-40% du pic)
Durée du Trade ({recommended_trade_expiration_minutes} min)

Formule: max(peak_duration, half_life × 2)
Temps maximum avant fermeture pour éviter les whipsaws
Critique pour le trailing stop
Score de Confiance ({confidence_score} %)

Fiabilité basée sur le sample_size du créneau
Scoring : ≥100 occ = 100% | 50-99 = 90% | 30-49 = 75% | 15-29 = 60% | <15 = 50%
Influence position size et stop loss
BLOC 5 : Paramètres BIDI Optimisés (BidiParametersSection)
5 paramètres de trading calculés :

Meilleur Moment

Affiche l'heure exacte du créneau optimal
Sélectionné parmi top 3 créneau avec taux succès ≥55%
Taux de Succès Entry Window ({optimal_win_rate} %)

Pourcentage de trades gagnants sur le créneau (atteint TP avant SL)
Calculé sur historique complet avec ajustement volatilité/range/body-range
Minimum 55% pour biais positif, >65% = excellent
Stop Loss ({slPips} pips)

Formule: (Score/100) × (ATR × 2.5)
Distance entre entrée et SL
Exemple: score 60 = ±20 pips
Win Rate ({winProbability} %)

Basé sur histogramme des mouvements du créneau
Combinaison de: success_rate + volatility_score + body_range_score
55% = profitable

Avg Gain / Avg Loss / Risk-Reward Ratio

Gain moyen historique vs perte moyenne
Ratio = Avg_Gain / Avg_Loss
Objectif: ratio > 2.0 pour viabilité
BLOC 6 : Performance Straddle Simulée (StraddlePerformanceSection)
3 métriques de backtesting :

Win Rate Simulé ({win_rate_percentage} %)

Nombre de wins / total trades (ex: 125/200)
Calculé en simulant le straddle historiquement
Fréquence Whipsaw ({whipsaw_frequency_percentage} %)

% de fausses sorties (atteint SL + TP sans profit net)
Indique la qualité du créneau
Risk Level color: 🟢 Low / 🟡 Medium / 🔴 High
Calcul du SL Optimal ({offset_pips} pips)

Basé sur P95 des wicks (percentile 95 des rejets)
Statitique: 95% des mouvements restent dans ce range
Exemple: "12.5 pips (P95: 15.8)"
BLOC 7 : Observations & Conseils (ObservationsSection)
5 observations synthétiques :

📊 Range

Affichage en pips avec status
0.0025 = ✅ Excellent | >0.0015 = ⚠️ Bon | <0.0015 = ❌ Faible

⚡ ATR

Affichage en pips avec status
0.0020 = ✅ Excellent | >0.0010 = ⚠️ Bon | <0.0010 = ❌ Faible

📈 Body Range

Pourcentage avec status
45% = ✅ Très Pur | >25% = ⚠️ Acceptable | <25% = ❌ Très Bruité

💫 Qualité Mouvement

Score /10 issu du movement_quality_score
≥8 = Excellent | ≥6 = Bon | <6 = Faible
💡 Conseils contextuels

Recommandations personnalisées basées sur tous les paramètres
Ex: "Range excellent mais volatilité basse → réduire position", etc.
BLOC 8 : Graphique de Décroissance (VolatilityDecayChart)
Visualisation de la dégradation de volatilité dans le temps :

Courbe exponentielle affichée :

X = Temps (minutes) depuis l'entrée
Y = Volatilité résiduelle (%)
Trois zones marquées:
Zone Pic : 0 à peak_duration_minutes (volatilité > 80%)
Zone Décroissance : Courbe exponentielle
Zone Mort : Après recommended_trade_expiration_minutes (fermer)
Légende :

Peak Volatility: {peakVolatility}% du mouvement initial
Half-Life: {halfLifeMinutes} min
Recommended Close Time: {recommendedDuration} min après entrée
Heure de départ: {startHour}:{startMinute}
Résumé des Incohérences Potentielles
📌 Points à vérifier/corriger :

Source des données conflictuelles ?

Les métriques viennent de 2 sources: stats_15min agrégés MOINS analyze_slice_metrics brut du créneau
Possible divergence entre valeurs "moyennes" (quarterly_aggregator) et "réelles" (commande Rust)
Cohérence entre blocs?

Body Range % vs Noise Ratio : relation inverse souvent
Win Rate (bloc 6) vs Taux Succès Entry (bloc 5) : sont-ils basés sur les mêmes données?
Formules de calcul imprécises?

Score Straddle : pondérations exactes non documentées
SL/TP : formules varient entre blocs (BidiParametersSection vs VolatilityDurationSection)
Confidence Score vs Sample Size?

Relation linéaire ou logarithmique?
Même formule utilisée partout?