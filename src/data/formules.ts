/**
 * data/formules.ts - Catalogue exhaustif de TOUTES les formules
 * Organisé par catégories pour la modal "Formules"
 */

export interface Formule {
  id: string
  titre: string
  definition: string
  explication_litterale?: string
  formule: string
  inputs: string[]
  output: {
    type: string
    range: string
    unite: string
  }
  exemple: string
  notes: string[]
  categorieId: string
}

export interface Categorie {
  id: string
  titre: string
  emoji: string
  description: string
  formules: string[] // IDs des formules
}

// CATÉGORIES
export const categories: Categorie[] = [
  {
    id: 'volatilite',
    titre: 'Volatilité & ATR',
    emoji: '📊',
    description: 'Mesures de l\'amplitude et de la volatilité',
    formules: ['atr', 'range_moyen', 'volatilite_percent', 'body_percent']
  },
  {
    id: 'whipsaw',
    titre: 'Whipsaw Analysis',
    emoji: '⚡',
    description: 'Analyse des retournements rapides',
    formules: ['whipsaw_freq', 'whipsaw_risk_level', 'total_trades_simules']
  },
  {
    id: 'timing',
    titre: 'Timing & Durée',
    emoji: '⏱️',
    description: 'Analyse temporelle et durée de volatilité',
    formules: ['peak_duration', 'half_life', 'trade_expiration', 'confidence']
  },
  {
    id: 'mouvement',
    titre: 'Mouvement & Qualité',
    emoji: '📈',
    description: 'Qualité et directionalité du mouvement',
    formules: ['noise_ratio', 'shadow_ratio', 'volume_imbalance', 'breakout_percent']
  },
  {
    id: 'scores',
    titre: 'Scores & Recommandations',
    emoji: '🔢',
    description: 'Scores finaux et recommandations',
    formules: ['score_brut', 'score_ajuste', 'recommendation', 'meilleure_heure']
  },
  {
    id: 'retrospectif',
    titre: 'Analyse Rétrospective',
    emoji: '📊',
    description: 'Métriques d\'analyse rétrospective pour backtesting',
    formules: [
      'peak_delay',
      'whipsaw_root_cause',
      'entry_timing_profitability',
      'volatility_decay_profile',
      'directional_bias_score'
    ]
  },
  {
    id: 'backtest',
    titre: 'Backtest & Performance',
    emoji: '🧪',
    description: 'Métriques de performance issues des simulations et de l\'analyse avancée',
    formules: [
      'win_rate', 'profit_factor', 'max_drawdown', 'average_pips',
      'mfe', 'mae', 'mfe_mae_ratio',
      'tp_potential_rate', 'tp_miss_rate',
      'be_hit_rate', 'trailing_exit_rate',
      'no_entry_rate', 'consecutive_losses',
      'quick_win_rate', 'quick_loss_rate',
      'cost_ratio', 'profitable_months_ratio',
      'recommended_sl_p75', 'recommended_tpr'
    ]
  },
  {
    id: 'spread_cost',
    titre: 'Coûts Spread & Slippage',
    emoji: '💸',
    description: 'Impact des coûts cachés en News Trading',
    formules: ['spread_impact']
  }
]

// FORMULES DÉTAILLÉES
export const formules: Record<string, Formule> = {
  // === VOLATILITÉ & ATR ===
  atr: {
    id: 'atr',
    titre: 'ATR (Average True Range)',
    categorieId: 'volatilite',
    definition: 'Mesure de la volatilité réelle incluant les gaps. Le True Range est le max de 3 valeurs, puis lissé avec une EMA Wilder sur 14 périodes.',
    explication_litterale: 'Cette formule mesure à quel point le marché bouge vraiment. Elle regarde la plus grande variation entre le haut et le bas d\'une chandelle, puis elle moyenne ces variations sur 14 chandelles. Plus l\'ATR est grand, plus le marché est volatil (bouge beaucoup). Plus l\'ATR est petit, plus le marché est calme (bouge peu).',
    formule: 'TR = max(H-L, |H-Cₚₚₚ|, |L-Cₚₚₚ|)\nATR = EMA(TR, 14)',
    inputs: ['High', 'Low', 'Close (précédent)', 'Période: 14 candles'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points MetaTrader 5'
    },
    exemple: 'EURUSD M1: ATR = 12.5 points (volatilité moyenne)',
    notes: [
      'Sensible aux gaps (inclus dans TR)',
      'Wilder\'s EMA plus fluide que SMA',
      'Minimum 2 candles requis'
    ]
  },

  range_moyen: {
    id: 'range_moyen',
    titre: 'Range Moyen',
    categorieId: 'volatilite',
    definition: 'Amplitude moyenne des candles pour une période donnée (heure ou 15min).',
    explication_litterale: 'Cette formule mesure la distance moyenne entre le haut et le bas des chandelles. C\'est simple: on prend chaque chandelle, on regarde sa hauteur (haut - bas), puis on en fait la moyenne. Plus le range est grand, plus les chandelles sont grosses (marché actif). Plus le range est petit, plus les chandelles sont minces (marché calme).',
    formule: 'Range = Σ(High - Low) / n',
    inputs: ['High (n candles)', 'Low (n candles)', 'n = nombre candles'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points MetaTrader 5'
    },
    exemple: 'Hour 12:00-12:59: Range = 45 points (moyenne de 60 candles M1)',
    notes: [
      'Plus simple que ATR, ne compte pas les gaps',
      'Utile pour normaliser les TP/SL'
    ]
  },

  volatilite_percent: {
    id: 'volatilite_percent',
    titre: 'Volatilité %',
    categorieId: 'volatilite',
    definition: 'ATR normalisé en pourcentage du prix. Permet de comparer volatilité Forex vs Crypto vs Indices.',
    explication_litterale: 'Cette formule convertit l\'ATR en pourcentage du prix pour pouvoir comparer des marchés différents. Par exemple, si l\'ATR=30 points et le prix=1.1000, on divise pour obtenir un pourcentage comparable. Utile pour voir: "est-ce que l\'EUR est plus volatil que l\'OR?"',
    formule: 'Vol% = (ATR / Close) × 100',
    inputs: ['ATR moyen', 'Close price estimé'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: '%'
    },
    exemple: 'EURUSD: ATR=12.5 points, Close≈1.1000 → Vol% = (12.5/11000)×100 ≈ 0.11%',
    notes: [
      'Prix estimé: Forex=1.0, Indices=10000, Crypto=100000',
      'Permet comparaison multi-assets'
    ]
  },

  body_percent: {
    id: 'body_percent',
    titre: 'Body % (Directionalité)',
    categorieId: 'volatilite',
    definition: 'Ratio du corps de la candle par rapport au range total. Mesure la directionalité: >50% = fort mouvement directionnel.',
    explication_litterale: 'Cette formule regarde la force d\'une chandelle. Elle mesure: combien du mouvement total a-t-il été "concluant"? Si la chandelle monte de 100 points du bas au haut (range=100), mais le corps (ouverture à fermeture) ne bouge que de 10 points, alors Body%=10% (peu directionnel). Si le corps=90 points, alors Body%=90% (très directionnel).',
    formule: 'Body% = |Close - Open| / (High - Low) × 100',
    inputs: ['Open', 'Close', 'High', 'Low'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'Candle: O=1.1050, C=1.1070, H=1.1075, L=1.1045 → Body% = 20/30 × 100 = 66%',
    notes: [
      '0% = candle indécise (mèches longues)',
      '100% = candle directionnelle parfaite',
      'Filtre Straddle: Body% < 20% recommandé'
    ]
  },

  noise_ratio: {
    id: 'noise_ratio',
    titre: 'Noise Ratio',
    categorieId: 'mouvement',
    definition: 'Ratio True Range / Mouvement Net. Mesure le "bruit" (mèches/gaps) vs signal (direction). >3 = trop de bruit.',
    explication_litterale: 'Cette formule regarde si une chandelle a beaucoup de "bruit" (mèches, gaps) par rapport à son mouvement net (Close à Close). Si beaucoup de bruit = marché chaotique. Si peu de bruit = marché directionnel propre. Ratio > 3 = très bruyant (mauvais pour trader). Ratio < 1.5 = très directionnel (bon pour trader).',
    formule: 'Noise = True Range / |Close - Open|',
    inputs: ['True Range', 'Close', 'Open'],
    output: {
      type: 'float',
      range: '1.0 - ∞',
      unite: 'ratio'
    },
    exemple: 'TR=30 points, Body=10 points → Noise = 3.0 (bruit modéré)',
    notes: [
      '< 2.0 = Excellent (directionnel)',
      '2.0-3.0 = Bon',
      '> 3.0 = À éviter (trop chaotique)',
      'Utilisé pour adapter Offset et SL'
    ]
  },

  shadow_ratio: {
    id: 'shadow_ratio',
    titre: 'Shadow Ratio (Mèches)',
    categorieId: 'mouvement',
    definition: 'Ratio des mèches (wicks) par rapport au range. Mesure l\'indécision du marché.',
    explication_litterale: 'Cette formule mesure les "mèches" (queues) des chandelles. Si une chandelle a de longues mèches en haut et en bas, c\'est que le marché a changé d\'avis plusieurs fois = indécision. Plus les mèches sont longues, plus le marché est indécis. Moins de mèches = décision claire.',
    formule: 'Upper_wick = High - max(Close, Open)\nLower_wick = min(Close, Open) - Low\nShadow = (Upper + Lower) / Range × 100',
    inputs: ['High', 'Low', 'Open', 'Close'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'Range=30, Upper_wick=8, Lower_wick=10 → Shadow = 18/30 × 100 = 60%',
    notes: [
      'Élevé = marché indécis',
      'Bas = direction claire'
    ]
  },

  volume_imbalance: {
    id: 'volume_imbalance',
    titre: 'Direction Strength',
    categorieId: 'mouvement',
    definition: 'Force directionnelle de la bougie : ratio body/range normalisé. Mesure la conviction du mouvement (pas un vrai volume imbalance bid/ask).',
    explication_litterale: 'Cette formule mesure quelle part du mouvement total (high-low) est due au corps réel de la bougie (open-close). Plus le corps est grand par rapport au mouvement total, plus la direction est claire et convaincante. C\'est un proxy de la force directionnelle, pas une mesure de flux acheteurs/vendeurs.',
    formule: 'Direction Strength = Body / Range = |Close - Open| / (High - Low)',
    inputs: ['Open', 'High', 'Low', 'Close'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'Bid=1000, Ask=600 → Imbalance = 400/1600 × 100 = 25%',
    notes: [
      '> 0 = Plus d\'acheteurs (hausse probable)',
      '< 0 = Plus de vendeurs (baisse probable)',
      'Absent si volume implicite non disponible'
    ]
  },

  breakout_percent: {
    id: 'breakout_percent',
    titre: 'Breakout %',
    categorieId: 'mouvement',
    definition: 'Pourcentage de candles cassant la moyenne mobile. Mesure l\'agressivité du mouvement.',
    explication_litterale: 'Cette formule compte combien de chandelles "sortent des sentiers battus" (sortent de la moyenne mobile). Si 80% des chandelles sont au-dessus de la moyenne, le marché pousse fort vers le haut. Si c\'est seulement 20%, le marché hésite. Plus de breakouts = plus d\'agressivité dans une direction.',
    formule: 'Breakout% = (Nombre candles > MA) / Total × 100',
    inputs: ['Close', 'MA (période 20)'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '45 candles/60 cassent la MA → Breakout% = 75%',
    notes: [
      'Élevé = tendance forte',
      'Bas = consolidation'
    ]
  },

  // === WHIPSAW ===
  whipsaw_freq: {
    id: 'whipsaw_freq',
    titre: 'Whipsaw Frequency %',
    categorieId: 'whipsaw',
    definition: 'Pourcentage de trades où BOTH Buy Stop ET Sell Stop se déclenchent dans 15min (perte garantie).',
    explication_litterale: 'Whipsaw = ton Straddle se déclenche dans DEUX directions en même temps = perte garantie. Cette formule compte combien de fois ça arrive. Si 5% des trades sont whipsaws = excellent. Si 30% = problématique. Un Straddle fiable doit avoir peu de whipsaws.',
    formule: 'Whipsaw% = (whipsaw_count / total_trades) × 100',
    inputs: ['Nombre whipsaws détectés', 'Total trades simulés'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '45 whipsaws / 1000 trades → 4.5% whipsaw',
    notes: [
      'Whipsaw = Buy+Sell déclenchés 15min après entry',
      '< 5% = Excellent',
      '5-10% = Bon',
      '10-20% = Acceptable',
      '> 30% = À éviter'
    ]
  },

  whipsaw_risk_level: {
    id: 'whipsaw_risk_level',
    titre: 'Whipsaw Risk Level (Giant Doji)',
    categorieId: 'whipsaw',
    definition: 'Détection des "Giant Dojis" : forte volatilité mais clôture proche de l\'ouverture. Signe de retournement violent (Whipsaw).',
    explication_litterale: 'Je cherche les bougies "pièges". Si une bougie est très grande (ATR > 15 pts) mais qu\'elle finit presque où elle a commencé (Corps < 35% du range), c\'est un "Giant Doji". Ça veut dire que le prix a explosé dans les deux sens puis est revenu. C\'est le pire scénario pour un Straddle.',
    formule: 'Risk = TRUE si (ATR > 15.0) ET (|Body| / Range < 0.35)',
    inputs: ['ATR', 'Body Range %'],
    output: {
      type: 'boolean',
      range: '{True, False}',
      unite: 'flag'
    },
    exemple: 'ATR=30 pts, Body=10% → Giant Doji détecté (Risque Whipsaw élevé)',
    notes: [
      'ATR > 15 pts : filtre les dojis de nuit (calmes)',
      'Body < 35% : signe d\'indécision majeure',
      'Indicateur clé pour éviter les pertes'
    ]
  },

  total_trades_simules: {
    id: 'total_trades_simules',
    titre: 'Total Trades Simulés',
    categorieId: 'whipsaw',
    definition: 'Nombre total d\'entrées potentielles analysées pour déterminer whipsaw frequency.',
    explication_litterale: 'C\'est le nombre de Straddles que je "teste" virtuellement dans les données historiques. Avec 100 données = 85 possibilités d\'entrée (100 - 15min fenêtre). Plus de données testées = plus confiance dans mes calculs de whipsaw. 1000+ trades = données solides. 50 trades = données faibles.',
    formule: 'Total = Nombre de candles - 15',
    inputs: ['Candles analysées (60 min minimum)'],
    output: {
      type: 'integer',
      range: '45 - ∞',
      unite: 'trades'
    },
    exemple: '60 candles M1 → 60-15 = 45 trades potentiels',
    notes: [
      '15 minutes nécessaires post-entry pour déterminer whipsaw',
      'Plus grand sample = plus fiable'
    ]
  },

  // === TIMING & DURÉE ===
  peak_duration: {
    id: 'peak_duration',
    titre: 'Peak Duration',
    categorieId: 'timing',
    definition: 'Estimation heuristique de la durée du pic de volatilité basée sur l\'ATR, la directionalité (Body%) et l\'impact des événements.',
    explication_litterale: 'Cette formule estime combien de temps la volatilité reste élevée. Elle part d\'une base (100-240 min) selon si le marché est volatil (ATR) et directionnel (Body%). Ensuite, elle ajuste selon l\'impact des événements: HIGH prolonge la durée (x1.5), LOW la réduit (x0.7).',
    formule: 'Base = f(ATR, Body%)\nDuration = Base × ImpactMultiplier (High=1.5, Low=0.7)\nClamped: [60, 300] min',
    inputs: ['ATR', 'Body Range %', 'Event Impact'],
    output: {
      type: 'integer',
      range: '60 - 300',
      unite: 'minutes'
    },
    exemple: 'ATR élevé + Body > 50% → Base 100 min. Event HIGH → 100 × 1.5 = 150 min.',
    notes: [
      'Base: 100, 140, 180 ou 240 min selon intensité',
      'Ajustement dynamique selon l\'annonce éco',
      'Minimum 1h, Maximum 5h'
    ]
  },

  half_life: {
    id: 'half_life',
    titre: 'Volatility Half-life',
    categorieId: 'timing',
    definition: 'Temps estimé pour que la volatilité retombe à 50% de son pic. Dépend fortement du "bruit" (Noise Ratio).',
    explication_litterale: 'Cette formule calcule la vitesse de retour au calme. Si le marché est "propre" (Noise < 1.5), la volatilité persiste longtemps (65% du pic). Si le marché est "bruyant" (Noise > 2.5), la volatilité s\'effondre vite (35% du pic).',
    formule: 'Ratio = f(NoiseRatio) → {0.65, 0.50, 0.35}\nHalf_life = PeakDuration × Ratio',
    inputs: ['Peak Duration', 'Noise Ratio'],
    output: {
      type: 'integer',
      range: '30 - 270',
      unite: 'minutes'
    },
    exemple: 'Peak=100 min, Noise=1.2 (propre) → Half-life = 65 min.\nPeak=100 min, Noise=3.0 (sale) → Half-life = 35 min.',
    notes: [
      'Noise < 1.5 → Ratio 0.65 (Décroissance lente)',
      'Noise < 2.5 → Ratio 0.50 (Décroissance normale)',
      'Noise > 2.5 → Ratio 0.35 (Décroissance rapide)'
    ]
  },

  trade_expiration: {
    id: 'trade_expiration',
    titre: 'Recommended Trade Expiration',
    categorieId: 'timing',
    definition: 'Durée totale recommandée du trade. Basée sur peak_duration avec buffer.',
    explication_litterale: 'C\'est le temps total que tu devrais garder ouvert ton Straddle. Si le pic arrive à 5 minutes et que la volatilité dure 10 minutes, je te recommande de fermer à 15 minutes pour attraper le mouvement mais avant que la volatilité ne disparaisse. Trop long = pertes. Trop court = pas assez de profit.',
    formule: 'Expiration = peak_duration × 1.5 (approximatif)\nOu: peak + 2 × half_life',
    inputs: ['Peak Duration', 'Half-life'],
    output: {
      type: 'integer',
      range: '10 - 30',
      unite: 'minutes'
    },
    exemple: 'Peak=12min, Half-life=6min → Expiration = 12 × 1.5 = 18 min',
    notes: [
      'Buffer: attendre décroissance volatilité',
      'Clamped à timeout maximum'
    ]
  },

  confidence: {
    id: 'confidence',
    titre: 'Confidence Score',
    categorieId: 'timing',
    definition: 'Score de confiance (0-100%) basé sur sample size et variance des mesures.',
    explication_litterale: 'Cette formule dit à quel point je suis "sûr" de mes calculs. Si j\'ai analysé 10 ans de données avec peu de variation, ma confiance est très haute (90%). Si j\'ai peu de données ou beaucoup de variation, ma confiance est basse (30%). Plus la confiance est haute, plus tu peux faire confiance à mes recommandations.',
    formule: 'Confidence = min(100, (sample_size / min_required) × 100 × variance_factor)',
    inputs: ['Sample size (jours analysés)', 'Variance ATR'],
    output: {
      type: 'integer',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'Sample=100 jours, Variance=faible → Confidence=95%',
    notes: [
      '> 80% = Haute confiance',
      '50-80% = Moyenne',
      '< 50% = Données insuffisantes'
    ]
  },

  // === SCORES & RECOMMANDATIONS ===
  score_brut: {
    id: 'score_brut',
    titre: 'Straddle Score (Brut)',
    categorieId: 'scores',
    definition: 'Score pondéré avant ajustement whipsaw. Agrège 5 métriques clés pour viabilité Straddle.',
    explication_litterale: 'Cette formule combiner 5 points importants pour le Straddle: volatilité, range, directionalité, bruit, et agressivité. Elle donne un note de 0 à 100. Score élevé (80+) = conditions excellentes. Score bas (20-) = mauvaises conditions.',
    formule: 'Score = (w1×ATR_norm + w2×Range_norm + w3×Body% + w4×Noise + w5×Breakout) / sum(weights)',
    inputs: ['ATR%', 'Range%', 'Body%', 'Noise Ratio', 'Breakout%'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: 'points'
    },
    exemple: 'Score brut = 78/100 (avant whipsaw)',
    notes: [
      'Poids: emphasis on volatilité et directionalité',
      'Base pour ajustement whipsaw'
    ]
  },

  score_ajuste: {
    id: 'score_ajuste',
    titre: 'Straddle Score (Ajusté)',
    categorieId: 'scores',
    definition: 'Score final après pénalité whipsaw. C\'est le vrai score de viabilité.',
    explication_litterale: 'C\'est le score réel après correction pour les faux signaux (whipsaw). Si le score brut est 80 mais il y a 30% de whipsaw, la note finale baisse pour être plus réaliste (56 au lieu de 80). Ce score te dit vraiment à quel point tu peux compter sur le Straddle à cette heure.',
    formule: 'Score_ajusté = Score_brut × (1 - whipsaw_freq / 100)',
    inputs: ['Score brut', 'Whipsaw frequency %'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: 'points'
    },
    exemple: 'Score=78, Whipsaw=20% → Score_ajusté = 78 × 0.8 = 62.4',
    notes: [
      '≥ 75 = Optimal',
      '60-74 = Good',
      '45-59 = Cautious',
      '< 45 = Risky'
    ]
  },

  recommendation: {
    id: 'recommendation',
    titre: 'Trading Recommendation',
    categorieId: 'scores',
    definition: 'Recommandation finale basée sur le score ajusté et conditions additionnelles.',
    explication_litterale: 'Voilà mon conseil final: dois-tu faire un Straddle à cette heure? "Optimal" = oui, conditions parfaites. "Good" = oui, conditions correctes. "Cautious" = oui, mais sois prudent. "Risky" = non, attends une meilleure heure. Ma recommandation se base sur tous mes autres calculs.',
    formule: 'IF score ≥ 75 AND whipsaw < 10 → "Optimal"\nELSE IF score ≥ 60 → "Good"\nELSE IF score ≥ 45 → "Cautious"\nELSE → "Risky"',
    inputs: ['Score ajusté', 'Whipsaw frequency', 'Win rate ajusté'],
    output: {
      type: 'enum',
      range: '{Optimal, Good, Cautious, Risky}',
      unite: 'recommendation'
    },
    exemple: 'Score=65, Whipsaw=8% → "Good"',
    notes: [
      'Decision tree: priorité score puis whipsaw',
      'Affichée avec emoji et couleur correspondante'
    ]
  },

  meilleure_heure: {
    id: 'meilleure_heure',
    titre: 'Meilleure Heure (Best Hour Ranking)',
    categorieId: 'scores',
    definition: 'Classement des 24 heures pour déterminer laquelle offre les meilleures conditions de trading Straddle. Basée sur 3 critères pondérés.',
    explication_litterale: 'Cette formule classe les 24 heures de la journée pour trouver les meilleures pour trader. Elle combine 3 éléments: la confiance dans les données (plus c\'est fiable, mieux c\'est), le taux de gain (plus on gagne souvent, mieux), et les faux déclenchements (moins il y en a, mieux). Elle additionne confiance + gain, puis soustrait les faux déclenchements. L\'heure avec le score le plus élevé est la meilleure.',
    formule: 'Score_heure = Confidence_Score + Win_Rate_ajusté - Whipsaw_Frequency\n\nRanking: ARGSORT(descending, Score_heure)',
    inputs: [
      'Confidence Score (0-100) - qualité des données',
      'Win Rate ajusté (%) - probabilité de profit',
      'Whipsaw Frequency (%) - fréquence des faux déclenchements'
    ],
    output: {
      type: 'ranking',
      range: '1-24',
      unite: 'heure (0-23)'
    },
    exemple: 'Heure 08:00 → Conf=78 + WR=45 - Whipsaw=15 = 108 ✅ 1st\nHeure 09:00 → Conf=68 + WR=40 - Whipsaw=22 = 86 (2nd)\nHeure 10:00 → Conf=55 + WR=35 - Whipsaw=30 = 60 (3rd)',
    notes: [
      'Calcul INDÉPENDANT du SL (SL ne change pas le ranking)',
      'Calcul INDÉPENDANT des arrondis .ceil()',
      'Meilleure heure = celle avec SCORE LE PLUS ÉLEVÉ',
      'Confidence = ATR + Body% + Volatilité + Noise + Breakout (0-100)',
      'Win Rate ajusté = WR brut × (1 - Whipsaw%)',
      'Whipsaw impact: Freq=0% → pas pénalité, Freq=33% → perte 33 points'
    ]
  },

  // === ANALYSE RÉTROSPECTIVE (PHASE 7) ===
  peak_delay: {
    id: 'peak_delay',
    titre: 'Peak Delay (Minutes)',
    categorieId: 'retrospectif',
    definition:
      'Délai en minutes entre l\'annonce d\'un événement et le pic de volatilité réel.',
    explication_litterale:
      'Cette formule mesure QUAND arrive le vrai mouvement. Si Peak Delay = +2.3 min, ça signifie qu\'après l\'annonce, il faut attendre 2.3 minutes pour voir le mouvement maximum. Utile pour savoir: "Quand est-ce que je dois être attentif?"',
    formule: 'Peak_Delay = Time(max_ATR) - Time(event_announcement)',
    inputs: ['Time of announcement', 'ATR timeseries'],
    output: { type: 'integer', range: '-5 to +15', unite: 'minutes' },
    exemple:
      'NFP annoncé à 13:30:00 → Peak ATR à 13:32:18 → Delay = +2.3 min',
    notes: [
      'Négatif = pic avant l\'annonce (rare)',
      'Positif = pic après l\'annonce (habituel)',
      'Variance importante selon paires et types d\'événements'
    ]
  },

  whipsaw_root_cause: {
    id: 'whipsaw_root_cause',
    titre: 'Whipsaw Root Cause',
    categorieId: 'retrospectif',
    definition:
      'Analyse des whipsaws: combien avant peak vs après peak.',
    explication_litterale:
      'Cette formule sépare les faux déclenchements en deux: ceux qui arrivent AVANT le pic (mauvaise chance) et ceux qui arrivent APRÈS (mauvais SL). Si beaucoup de whipsaws "late", tu dois agrandir ton SL. Si beaucoup de "early", c\'est juste de la malchance.',
    formule:
      'Whipsaw_Early% = (whipsaws_before_peak / total_whipsaws) × 100\nWhipsaw_Late% = (whipsaws_after_peak / total_whipsaws) × 100',
    inputs: ['Whipsaw events', 'Peak duration'],
    output: { type: 'float', range: '0-100', unite: '%' },
    exemple:
      '8% early (before peak), 20% late (after peak) → SL issue → Increase SL',
    notes: [
      'Early whipsaw = avant le pic = malchance du timing',
      'Late whipsaw = après le pic = SL trop serré',
      'Ajuster SL si late% > 15%'
    ]
  },

  entry_timing_profitability: {
    id: 'entry_timing_profitability',
    titre: 'Entry Timing Profitability',
    categorieId: 'retrospectif',
    definition:
      'Profitabilité stratifiée par offset d\'entrée (T-10, T-5, T-0, T+3).',
    explication_litterale:
      'Cette formule te montre: "Si j\'étais entré 5 minutes avant l\'annonce, quel aurait été mon win rate?" Compare 4 moments d\'entrée différents pour trouver le meilleur. Le moment idéal change selon l\'événement.',
    formule:
      'For each entry_offset in [-10, -5, 0, +3]:\n  Win_Rate(offset) = (wins / total) × 100\n  P&L(offset) = sum(profits) / total',
    inputs: ['Backtest results', 'Entry time offsets'],
    output: { type: 'matrix', range: '4 rows × 5 cols', unite: 'win%, P&L' },
    exemple:
      'T-5 min: 52% win, +18p avg ← BEST\nT-0 min: 50% win, +8p avg\nT+3 min: 45% win, -5p avg',
    notes: [
      'Meilleur offset varie par type d\'événement',
      'NFP optimal = T-5 min',
      'Jobless optimal = T-3 min'
    ]
  },

  volatility_decay_profile: {
    id: 'volatility_decay_profile',
    titre: 'Volatility Decay Profile',
    categorieId: 'retrospectif',
    definition:
      'Taux de décroissance de la volatilité après le pic (points/minute).',
    explication_litterale:
      'Cette formule mesure: "Comment vite la volatilité baisse après le mouvement?" Si la volatilité baisse très vite (3 points/minute), le mouvement est court, donc timeout court. Si elle baisse lentement (0.8 points/minute), le mouvement dure longtemps, donc timeout long.',
    formule:
      'Decay_Rate = (Peak_ATR - ATR_at_T+10) / 10 min\nDecay_Speed = FAST (>3) | MEDIUM (1.5-3) | SLOW (<1.5)',
    inputs: ['ATR timeseries', 'Peak ATR value'],
    output: { type: 'float', range: '0.5 to 5.0', unite: 'points/minute' },
    exemple:
      'Peak 45p → 18p at T+10 → Decay = 2.7p/min = MEDIUM → Timeout = 25 min',
    notes: [
      'FAST decay = mouvement court = short timeout (18 min)',
      'MEDIUM decay = équilibré = medium timeout (25 min)',
      'SLOW decay = mouvement long = long timeout (32 min)'
    ]
  },

  directional_bias_score: {
    id: 'directional_bias_score',
    titre: 'Directional Bias Score',
    categorieId: 'retrospectif',
    definition:
      'Asymétrie UP vs DOWN des gagnants: mesure la tendance inhérente.',
    explication_litterale:
      'Cette formule regarde: "Les achats gagnent-ils plus que les ventes pour cet événement?" Si oui = événement biaisé HAUT. Si non = biaisé BAS. Si égal = neutre. Un Straddle fonctionne mieux sur événements neutres.',
    formule:
      'UP_Bias = (Up_Wins - Down_Wins) / Total_Wins\nAsymmetry% = |UP_Bias| × 100\nClassification: |Bias| > 0.3 = BIASED, else NEUTRAL',
    inputs: ['Backtest results (buy/sell side)'],
    output: {
      type: 'enum',
      range: '{UP_BIASED, DOWN_BIASED, NEUTRAL}',
      unite: 'classification'
    },
    exemple: 'NFP: 67% buy wins, 33% sell wins → Bias = +0.7 → UP_BIASED',
    notes: [
      'Straddle fonctionne mal sur événements biaisés',
      'Meilleur sur événements NEUTRAL',
      'Si biaisé, éviter le Straddle simultané'
    ]
  },

  // === BACKTEST & PERFORMANCE ===
  win_rate: {
    id: 'win_rate',
    titre: 'Win Rate (Taux de réussite)',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades gagnants par rapport au nombre total de trades exécutés. Inclut les issues TakeProfit ET TrailingStop comme gagnantes.',
    explication_litterale: 'Cette formule calcule combien de fois la stratégie a gagné. Un trade est "gagnant" s\'il se termine en TakeProfit ou en TrailingStop avec un gain positif. Les trades Timeout peuvent être gagnants ou perdants selon le PnL final.',
    formule: 'Win Rate = (Winning Trades / Total Trades) × 100\n\nWinning = outcome ∈ {TakeProfit, TrailingStop} ou pips_net > 0',
    inputs: ['Winning Trades (TP + Trailing + Timeout positifs)', 'Total Trades exécutés'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '60 gagnants / 100 total = 60% (inclut 45 TP + 15 TrailingStop)',
    notes: [
      '> 50% est généralement requis pour être profitable',
      'Inclut TakeProfit + TrailingStop comme issues gagnantes',
      'Timeout peut être gagnant ou perdant selon le PnL final'
    ]
  },

  profit_factor: {
    id: 'profit_factor',
    titre: 'Profit Factor',
    categorieId: 'backtest',
    definition: 'Ratio entre les gains bruts et les pertes brutes.',
    explication_litterale: 'Le Profit Factor nous dit combien on gagne pour chaque dollar perdu. Un PF de 1.5 signifie qu\'on gagne 1.50$ pour chaque 1.00$ perdu. C\'est la mesure ultime de la rentabilité.',
    formule: 'Profit Factor = Gross Profit / Gross Loss',
    inputs: ['Gross Profit (somme gains)', 'Gross Loss (somme pertes)'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'ratio'
    },
    exemple: 'Gains totaux 1500, Pertes totales 1000 → PF = 1.5',
    notes: [
      '< 1.0 = Stratégie perdante',
      '> 1.5 = Stratégie solide',
      '> 2.0 = Stratégie excellente'
    ]
  },

  max_drawdown: {
    id: 'max_drawdown',
    titre: 'Max Drawdown',
    categorieId: 'backtest',
    definition: 'La plus grande baisse de capital (du pic au creux) durant la période de test.',
    explication_litterale: 'C\'est le "pire scénario" historique. Si tu avais commencé au pire moment, combien aurais-tu perdu avant de remonter? Ça mesure le risque psychologique et financier.',
    formule: 'MDD = Max(Peak - Current_PnL)',
    inputs: ['Equity Curve'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'Compte monte à +100, descend à +60 → Drawdown = 40 pips',
    notes: [
      'Doit être acceptable par rapport au capital',
      'Un drawdown trop élevé tue le compte même si la stratégie est gagnante à long terme'
    ]
  },

  average_pips: {
    id: 'average_pips',
    titre: 'Average Pips per Trade',
    categorieId: 'backtest',
    definition: 'Gain moyen (ou perte) par trade en pips.',
    explication_litterale: 'En moyenne, combien chaque trade rapporte-t-il? C\'est l\'espérance mathématique par trade. Si c\'est positif, la stratégie est gagnante. Si c\'est négatif, elle est perdante.',
    formule: 'Avg Pips = Total Net Pips / Total Trades',
    inputs: ['Total Net Pips', 'Total Trades'],
    output: {
      type: 'float',
      range: '-∞ - +∞',
      unite: 'pips'
    },
    exemple: 'Total +500 pips / 100 trades = +5 pips/trade',
    notes: [
      'Doit couvrir le spread et les commissions',
      'Si < Spread, la stratégie perdra en réel'
    ]
  },

  spread_impact: {
    id: 'spread_impact',
    titre: 'Impact du Spread',
    categorieId: 'spread_cost',
    definition: 'Pourcentage du mouvement de volatilité "mangé" par le spread.',
    explication_litterale: 'Si le marché bouge de 20 pips mais que le spread est de 2 pips, le spread représente 10% du mouvement. Plus ce chiffre est bas, mieux c\'est. Au-dessus de 20-30%, trader devient très risqué car le coût d\'entrée est trop élevé par rapport au gain potentiel.',
    formule: 'Impact = (Spread / Volatilité_Attendue) * 100',
    inputs: ['Spread Moyen', 'Volatilité (ATR ou Mouvement)'],
    output: {
      type: 'percentage',
      range: '0% - 100%',
      unite: '%'
    },
    exemple: 'Spread 2 pips / Volatilité 10 pips = 20% Impact',
    notes: [
      'Critique pour le Scalping et News Trading',
      'Si > 30%, éviter de trader',
      'Le spread s\'élargit souvent pendant les news'
    ]
  },

  // === NOUVELLES MÉTRIQUES D'ANALYSE AVANCÉE ===
  mfe: {
    id: 'mfe',
    titre: 'MFE (Max Favorable Excursion)',
    categorieId: 'backtest',
    definition: 'Excursion maximale favorable : le plus grand gain latent atteint avant la clôture du trade.',
    explication_litterale: 'Le MFE mesure "jusqu\'où le trade est allé en ma faveur" avant de se fermer. Si j\'achète à 100 et que le prix monte à 120 puis redescend à 110 où je ferme, mon MFE est 20. C\'est le potentiel réel du trade. Un MFE élevé par rapport au gain final signifie qu\'on "laisse de l\'argent sur la table".',
    formule: 'Long: MFE = highest_price - entry_price\nShort: MFE = entry_price - lowest_price\n\nMFE moyen = Σ(MFE) / n',
    inputs: ['Entry price', 'Highest/Lowest pendant le trade', 'Point value'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'Entry=1.1000, Highest=1.1050 → MFE = 50 pips\nMoyenne sur 100 trades = 35 pips',
    notes: [
      'Toujours ≥ 0 (par définition)',
      'Sert à évaluer si le TP est bien calibré',
      'Si MFE moyen >> gain moyen → TP trop serré ou trailing trop agressif'
    ]
  },

  mae: {
    id: 'mae',
    titre: 'MAE (Max Adverse Excursion)',
    categorieId: 'backtest',
    definition: 'Excursion maximale adverse : la plus grande perte latente subie avant la clôture du trade.',
    explication_litterale: 'Le MAE mesure "jusqu\'où le trade est allé contre moi" avant de se fermer. Si j\'achète à 100 et que le prix descend à 85 puis remonte à 105 où je ferme, mon MAE est 15. C\'est le risque réel subi. Un MAE élevé indique une exposition au risque importante.',
    formule: 'Long: MAE = entry_price - lowest_price\nShort: MAE = highest_price - entry_price\n\nMAE moyen = Σ(MAE) / n',
    inputs: ['Entry price', 'Highest/Lowest pendant le trade', 'Point value'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'Entry=1.1000, Lowest=1.0960 → MAE = 40 pips\nMoyenne sur 100 trades = 25 pips',
    notes: [
      'Toujours ≥ 0 (par définition)',
      'Sert de base pour calibrer le SL optimal (P75 MAE)',
      'Si MAE moyen > SL → beaucoup de stops touchés "juste avant" le rebond'
    ]
  },

  mfe_mae_ratio: {
    id: 'mfe_mae_ratio',
    titre: 'Ratio MFE/MAE',
    categorieId: 'backtest',
    definition: 'Rapport entre le potentiel moyen (MFE) et le risque moyen (MAE). Mesure l\'efficacité de la stratégie.',
    explication_litterale: 'Ce ratio dit si on gagne plus qu\'on ne risque. Si MFE/MAE = 2.0, les trades vont 2× plus loin en faveur qu\'en défaveur. C\'est un indicateur de qualité pure : plus le ratio est élevé, meilleure est la stratégie.',
    formule: 'Ratio = MFE moyen / MAE moyen',
    inputs: ['MFE moyen (pips)', 'MAE moyen (pips)'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'ratio'
    },
    exemple: 'MFE moyen=35 pips, MAE moyen=20 pips → Ratio = 1.75',
    notes: [
      '≥ 1.5 = Excellent (vert)',
      '1.0 - 1.5 = Correct (orange)',
      '< 1.0 = Les trades vont plus en défaveur qu\'en faveur (rouge)'
    ]
  },

  tp_potential_rate: {
    id: 'tp_potential_rate',
    titre: 'TP Potentiel Atteint',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades dont le MFE a atteint ou dépassé le TP cible, indépendamment de l\'issue réelle.',
    explication_litterale: 'Cette formule répond à : "Combien de trades auraient pu atteindre le Take Profit ?" On regarde si le MFE (meilleur moment du trade) dépasse la cible TP. Si 60% des trades y arrivent mais seulement 30% finissent en TP réel, ça montre un problème de trailing ou de timing.',
    formule: 'TP_cible = SL × TP(R)\nTP_potential% = (trades où MFE ≥ TP_cible) / total × 100',
    inputs: ['MFE par trade', 'SL (pips)', 'TP(R) ratio'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'SL=20, TP(R)=3 → cible=60 pips\n45 trades sur 100 ont MFE ≥ 60 → 45%',
    notes: [
      '≥ 50% = Excellent potentiel (vert)',
      '30-50% = Potentiel correct (orange)',
      '< 30% = Cible trop ambitieuse (rouge)',
      'Si très haut mais TP rate bas → trailing ferme trop tôt'
    ]
  },

  tp_miss_rate: {
    id: 'tp_miss_rate',
    titre: 'TP Manqué',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades qui auraient pu atteindre le TP (MFE ≥ cible) mais n\'ont pas été clôturés en TakeProfit.',
    explication_litterale: 'C\'est la mesure de "l\'argent laissé sur la table". Si un trade monte jusqu\'au TP puis redescend et se ferme en trailing ou timeout, c\'est un TP manqué. Un taux élevé indique que le trailing stop est trop agressif ou que le TP n\'est pas atteint à cause du spread/timing.',
    formule: 'TP_miss% = (trades où MFE ≥ TP_cible ET outcome ≠ TakeProfit) / total × 100',
    inputs: ['MFE par trade', 'TP cible', 'Outcome par trade'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '45 trades MFE ≥ TP, mais seulement 30 en TP → 15 manqués = 15%',
    notes: [
      '< 30% = Acceptable (vert)',
      '30-50% = Trailing trop serré (orange)',
      '> 50% = Problème de configuration (rouge)'
    ]
  },

  be_hit_rate: {
    id: 'be_hit_rate',
    titre: 'Breakeven (BE) Atteint',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades où le Breakeven a été activé (détecté via les logs de simulation).',
    explication_litterale: 'Le Breakeven se déclenche quand le prix a bougé suffisamment en faveur. Une fois le BE atteint, le trailing stop s\'active. Un taux élevé signifie que beaucoup de trades atteignent un état "protégé" (pas de perte possible).',
    formule: 'BE% = (trades avec log "BE Long" ou "BE Short") / total × 100',
    inputs: ['Logs de simulation par trade', 'Total trades exécutés'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '65 trades sur 100 déclenchent le BE → 65%',
    notes: [
      '≥ 50% = Bon, la majorité des trades atteignent la sécurité (vert)',
      '30-50% = Moyen (orange)',
      '< 30% = Le prix ne va pas assez loin pour activer le BE (rouge)'
    ]
  },

  trailing_exit_rate: {
    id: 'trailing_exit_rate',
    titre: 'Sorties Trailing Stop',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades clôturés par le trailing stop (outcome = TrailingStop).',
    explication_litterale: 'Mesure la fréquence à laquelle le trailing stop ferme les positions. Si trop fréquent (>40%), le trailing est trop serré et coupe les gains. Si trop rare (<15%), le trailing n\'a presque pas d\'effet et peut être resserré.',
    formule: 'Trailing% = (trades avec log "TS Long" ou "TS Short") / total × 100',
    inputs: ['Logs de simulation', 'Total trades exécutés'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '25 sorties trailing / 100 trades → 25%',
    notes: [
      '> 40% = Trailing trop serré, relâcher le coefficient',
      '15-40% = Bon équilibre',
      '< 15% = Trailing peu actif, envisager de resserrer',
      'Utilisé pour recommander le coefficient optimal'
    ]
  },

  no_entry_rate: {
    id: 'no_entry_rate',
    titre: 'Taux de Non-Déclenchement',
    categorieId: 'backtest',
    definition: 'Pourcentage d\'événements où aucun trade n\'a été déclenché (pas de mouvement suffisant pour activer le straddle).',
    explication_litterale: 'Certains événements ne provoquent pas assez de mouvement pour déclencher les ordres Buy Stop / Sell Stop. Ça signifie que la volatilité n\'a pas été suffisante. Un taux élevé de non-entrée peut indiquer un offset trop élevé ou des données M1 manquantes autour de T0.',
    formule: 'NoEntry% = no_entries / (total_trades + no_entries) × 100',
    inputs: ['No entries (events sans trade)', 'Total trades', 'Total events'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '15 non-entry / 100 events → 15%',
    notes: [
      '< 10% = Normal en straddle immédiat',
      '10-50% = Vérifier les données ou l\'offset',
      '> 60% = Problème de données ou offset trop large'
    ]
  },

  consecutive_losses: {
    id: 'consecutive_losses',
    titre: 'Pertes Consécutives Max',
    categorieId: 'backtest',
    definition: 'Plus longue série de trades perdants consécutifs. Mesure le "pire scénario psychologique".',
    explication_litterale: 'Combien de trades d\'affilée peut-on perdre au maximum ? C\'est crucial pour le money management : si on a eu 8 pertes d\'affilée, il faut s\'assurer que le capital survit à cette série. Plus cette valeur est élevée, plus il faut un money management conservateur.',
    formule: 'MaxStreak = max sequence de trades où pips_net < 0',
    inputs: ['Liste ordonnée des trades', 'PnL par trade'],
    output: {
      type: 'integer',
      range: '0 - ∞',
      unite: 'trades'
    },
    exemple: 'Séquence: +10, -5, -3, -8, -2, +15 → streak = 4 pertes consécutives',
    notes: [
      '≤ 3 = Acceptable',
      '4-6 = Attention au sizing',
      '> 6 = Revoir la stratégie ou filtrer les événements'
    ]
  },

  quick_win_rate: {
    id: 'quick_win_rate',
    titre: 'Wins Rapides (≤ 1 min)',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades gagnants clôturés en 1 minute ou moins, parmi tous les gagnants.',
    explication_litterale: 'Mesure les "victoires éclair". Un TP touché en moins d\'une minute indique un mouvement très violent et directionnel. Un taux élevé est positif car le capital est exposé très peu de temps.',
    formule: 'QuickWin% = (gagnants avec durée ≤ 1 min) / total gagnants × 100',
    inputs: ['Durée par trade gagnant'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '12 wins rapides / 60 gagnants → 20%',
    notes: [
      '≥ 40% = Mouvements très impulsifs (vert)',
      '20-40% = Normal (orange)',
      '< 20% = Mouvements graduels (neutre)'
    ]
  },

  quick_loss_rate: {
    id: 'quick_loss_rate',
    titre: 'Loss Rapides (≤ 1 min)',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades perdants clôturés en 1 minute ou moins, parmi tous les perdants.',
    explication_litterale: 'Mesure les "pertes instantanées". Un SL touché en moins d\'une minute indique un mouvement violent contre nous (souvent un whipsaw). Un taux élevé est préoccupant car le stop n\'a pas eu le temps de protéger.',
    formule: 'QuickLoss% = (perdants avec durée ≤ 1 min) / total perdants × 100',
    inputs: ['Durée par trade perdant'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '8 losses rapides / 40 perdants → 20%',
    notes: [
      '< 10% = Pertes graduelles (vert)',
      '10-25% = Attention aux spikes (orange)',
      '> 25% = Trop de whipsaws instantanés (rouge)'
    ]
  },

  cost_ratio: {
    id: 'cost_ratio',
    titre: 'Poids des Frais',
    categorieId: 'backtest',
    definition: 'Pourcentage du PnL total absorbé par les frais estimés (spread + slippage).',
    explication_litterale: 'Les frais (spread à l\'ouverture et fermeture, plus le slippage) grignotent les profits. Cette formule montre combien ils "coûtent" par rapport au résultat total. Si les frais représentent 30% du PnL, la stratégie est très sensible au spread.',
    formule: 'Coût/trade = (spread × 2) + (slippage × 2)\nCoût total = coût/trade × nombre trades\nRatio = coût_total / |PnL_total| × 100',
    inputs: ['Spread (pips)', 'Slippage (pips)', 'Nombre de trades', 'PnL total (pips)'],
    output: {
      type: 'float',
      range: '0 - ∞',
      unite: '%'
    },
    exemple: 'Spread=1.5, Slip=0.5, 100 trades → Coût=400 pips\nPnL=+1200 pips → Ratio = 33%',
    notes: [
      '< 10% = Frais négligeables (vert)',
      '10-25% = Impact modéré (orange)',
      '> 25% = Stratégie très sensible au spread (rouge)',
      'Le ×2 car spread/slippage s\'appliquent à l\'ouverture ET fermeture'
    ]
  },

  profitable_months_ratio: {
    id: 'profitable_months_ratio',
    titre: 'Mois Profitables',
    categorieId: 'backtest',
    definition: 'Ratio de mois avec un PnL net positif par rapport au total de mois. Mesure la stabilité temporelle.',
    explication_litterale: 'Avoir un bon Profit Factor ne suffit pas si tous les gains viennent d\'un seul mois. Cette métrique montre la régularité : si 8 mois sur 12 sont positifs, la stratégie est stable. Si seulement 3/12, elle dépend de quelques events chanceux.',
    formule: 'Stats par mois : PnL_mensuel = Σ(pips par trade du mois)\nRatio = mois_positifs / total_mois × 100',
    inputs: ['PnL par trade', 'Date de chaque trade'],
    output: {
      type: 'string',
      range: '0/0 - n/n',
      unite: 'mois positifs / total mois'
    },
    exemple: '8 mois positifs / 12 → 67% de stabilité',
    notes: [
      '≥ 60% = Stratégie stable (vert)',
      '40-60% = Instable (orange)',
      '< 40% = Dépendant de quelques événements (rouge)',
      'Chaque mois inclut aussi le Profit Factor et le PnL net'
    ]
  },

  recommended_sl_p75: {
    id: 'recommended_sl_p75',
    titre: 'SL Optimal (P75 MAE)',
    categorieId: 'backtest',
    definition: 'Stop Loss recommandé basé sur le 75ème percentile de la distribution MAE. Couvre 75% des excursions adverses.',
    explication_litterale: 'Au lieu de deviner le SL, on le calcule statistiquement. On prend tous les MAE (pire moment de chaque trade), on les trie, et on prend le P75 (75% des trades ont un MAE inférieur). Ce SL protège contre la majorité des excursions sans être trop large.',
    formule: 'MAE_values = [MAE₁, MAE₂, ..., MAEₙ]\nP75 = percentile(MAE_values, 75)\nSL_optimal = max(ceil(P75), 1)',
    inputs: ['Liste MAE de tous les trades exécutés'],
    output: {
      type: 'float',
      range: '1 - ∞',
      unite: 'pips'
    },
    exemple: 'MAE distribution: [5, 8, 12, 15, 18, 22, 30, 45]\nP75 = 22 → SL recommandé = 22 pips',
    notes: [
      'P75 = compromis couverture/distance',
      'P50 serait trop serré (50% des trades toucheraient le SL)',
      'P90 serait trop large (capital exposé inutilement)',
      'Minimum 1 pip (sécurité)'
    ]
  },

  recommended_tpr: {
    id: 'recommended_tpr',
    titre: 'TP(R) Optimal',
    categorieId: 'backtest',
    definition: 'Ratio TP/SL recommandé basé sur la MFE médiane divisée par le SL optimal.',
    explication_litterale: 'On calcule le TP idéal en regardant la médiane des MFE (potentiel réel). On divise par le SL optimal (P75 MAE) pour obtenir un ratio R:R. Si la MFE médiane est 3× le SL, on peut viser un TP(R) de 3. Si elle est seulement 1.5×, on vise TP(R) = 1.5.',
    formule: 'MFE_median = median(MFE_values)\nRaw_TPR = MFE_median / SL_optimal\nTP(R) = max(round(Raw_TPR × 2) / 2, 1)',
    inputs: ['Liste MFE de tous les trades', 'SL optimal (P75 MAE)'],
    output: {
      type: 'float',
      range: '1.0 - ∞',
      unite: 'ratio (arrondi au 0.5)'
    },
    exemple: 'MFE médiane = 45 pips, SL optimal = 20 pips\nRaw = 2.25 → arrondi = 2.5R',
    notes: [
      'Médiane (pas moyenne) pour éviter l\'influence des outliers',
      'Arrondi au 0.5 supérieur pour garder des valeurs pratiques',
      'Minimum 1.0R (sinon le risk:reward est trop défavorable)'
    ]
  }
}
