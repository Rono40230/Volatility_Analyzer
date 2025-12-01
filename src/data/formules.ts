/**
 * data/formules.ts - Catalogue exhaustif de TOUTES les formules
 * Organisé par catégories pour la modal "Formules"
 */

export interface Formule {
  id: string
  titre: string
  definition: string
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
    id: 'straddle',
    titre: 'Paramètres Straddle',
    emoji: '🎯',
    description: 'Configuration optimale du Straddle',
    formules: ['offset', 'offset_ajuste', 'meilleur_moment', 'win_rate_ajuste', 'trailing_stop', 'timeout']
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
    formules: ['score_brut', 'score_ajuste', 'recommendation']
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
    formule: 'TR = max(H-L, |H-Cₚₚₚ|, |L-Cₚₚₚ|)\nATR = EMA(TR, 14)',
    inputs: ['High', 'Low', 'Close (précédent)', 'Période: 14 candles'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'EURUSD M1: ATR = 12.5 pips (volatilité moyenne)',
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
    formule: 'Range = Σ(High - Low) / n',
    inputs: ['High (n candles)', 'Low (n candles)', 'n = nombre candles'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'Hour 12:00-12:59: Range = 45 pips (moyenne de 60 candles M1)',
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
    formule: 'Vol% = (ATR / Close) × 100',
    inputs: ['ATR moyen', 'Close price estimé'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: '%'
    },
    exemple: 'EURUSD: ATR=12.5 pips, Close≈1.1000 → Vol% = (12.5/11000)×100 ≈ 0.11%',
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
    definition: 'Ratio Range / Body. Mesure le "bruit" (mèches) vs signal (direction). >3 = trop de bruit.',
    formule: 'Noise = (High - Low) / |Close - Open|',
    inputs: ['High', 'Low', 'Close', 'Open'],
    output: {
      type: 'float',
      range: '1.0 - ∞',
      unite: 'ratio'
    },
    exemple: 'Range=30 pips, Body=10 pips → Noise = 3.0 (bruit modéré)',
    notes: [
      '< 2.0 = Excellent (directionnel)',
      '2.0-3.0 = Bon',
      '> 3.0 = À éviter (trop chaotique)',
      'Anti-pattern Straddle: filter si > 3.0'
    ]
  },

  shadow_ratio: {
    id: 'shadow_ratio',
    titre: 'Shadow Ratio (Mèches)',
    categorieId: 'mouvement',
    definition: 'Ratio des mèches (wicks) par rapport au range. Mesure l\'indécision du marché.',
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
    titre: 'Volume Imbalance',
    categorieId: 'mouvement',
    definition: 'Déséquilibre entre acheteurs et vendeurs (Bid/Ask). Prédit la direction du prochain mouvement.',
    formule: 'Imbalance = (Bid_Volume - Ask_Volume) / Total × 100',
    inputs: ['Bid Volume', 'Ask Volume'],
    output: {
      type: 'float',
      range: '-100 - 100',
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

  // === STRADDLE PARAMETERS ===
  offset: {
    id: 'offset',
    titre: 'Offset (Distance ordres)',
    categorieId: 'straddle',
    definition: 'Distance des ordres Buy Stop et Sell Stop par rapport au prix actuel. Basé sur ATR pour adapter à la volatilité.',
    formule: 'Offset = ATR × Multiplicateur\nMultiplicateur = 1.5-2.0 (adaptatif)',
    inputs: ['ATR local', 'Volatilité du quarter'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'ATR=12.5 pips, Multiplicateur=1.75 → Offset = 21.9 pips',
    notes: [
      'ATR faible → Offset réduit (moins de mouvement attendu)',
      'ATR élevé → Offset augmenté (volatilité haute)',
      'Fondation pour SL, TP, entrée'
    ]
  },

  offset_ajuste: {
    id: 'offset_ajuste',
    titre: 'SL Ajusté (Stop Loss)',
    categorieId: 'straddle',
    definition: 'Stop Loss augmenté pour compenser l\'impact du whipsaw. Plus la fréquence whipsaw est élevée, plus le SL doit être large.',
    formule: 'SL_ajusté = SL_brut × (1 + whipsaw_freq × 0.3)',
    inputs: ['SL brut (= Offset)', 'Whipsaw frequency %'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'pips'
    },
    exemple: 'SL=20 pips, Whipsaw=25% → SL_ajusté = 20 × (1 + 0.25 × 0.3) = 21.5 pips',
    notes: [
      'Whipsaw nul → SL = SL brut',
      'Whipsaw 50% → +15% sur SL',
      'Logique: whipsaw = faux déclenchements → besoin plus d\'espace'
    ]
  },

  meilleur_moment: {
    id: 'meilleur_moment',
    titre: 'Meilleur Moment (Entry Timing)',
    categorieId: 'straddle',
    definition: 'Minutes avant le début du quarter pour entrer. Basé sur analyse des moments de déclenchement whipsaw (trigger_minutes).',
    formule: 'Optimal = mean(whipsaw_trigger_minutes) × 0.6\nClamped: [0, quarter_end]',
    inputs: ['Whipsaw trigger times (par jour)', 'Quarter boundaries'],
    output: {
      type: 'integer',
      range: '0 - 25',
      unite: 'minutes'
    },
    exemple: 'Whipsaws déclenchent à: [5, 8, 12, 6] min → Mean=7.75 × 0.6 = 4.65 → arrondi 5 min',
    notes: [
      'Logique: entrer 60% avant whipsaw = sécurité',
      'Clamped pour rester dans le quarter',
      'Example: Quarter 12:30-12:45, entry=5min → 12:35'
    ]
  },

  win_rate_ajuste: {
    id: 'win_rate_ajuste',
    titre: 'Win Rate Ajusté',
    categorieId: 'straddle',
    definition: 'Taux de gain réaliste après pénalité whipsaw. Reflète la vraie probabilité de profit.',
    formule: 'WR_ajusté = WR_brut × (1 - whipsaw_freq)',
    inputs: ['Win Rate brut (simulation)', 'Whipsaw frequency %'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: 'WR=55%, Whipsaw=20% → WR_ajusté = 55 × 0.8 = 44%',
    notes: [
      'Whipsaw = réductions de gain direct',
      'À 50%+ = trade viable (espérance positive)',
      'À <50% = risqué'
    ]
  },

  trailing_stop: {
    id: 'trailing_stop',
    titre: 'Trailing Stop (Coefficient)',
    categorieId: 'straddle',
    definition: 'Multiplicateur du SL pour stop dynamique. Ajusté selon whipsaw pour adapter la traîne.',
    formule: 'TS = 1.59 × (1 - whipsaw_freq / 2)',
    inputs: ['Baseline: 1.59', 'Whipsaw frequency %'],
    output: {
      type: 'float',
      range: '0.8 - 1.59',
      unite: 'x SL'
    },
    exemple: 'Baseline=1.59, Whipsaw=30% → TS = 1.59 × 0.85 = 1.35x SL',
    notes: [
      'Whipsaw nul → TS = 1.59x',
      'Whipsaw élevé → TS réduit (moins de traîne)',
      'Réduit les faux déclenchements'
    ]
  },

  timeout: {
    id: 'timeout',
    titre: 'Timeout (Durée position)',
    categorieId: 'straddle',
    definition: 'Durée maximale pour tenir la position. Inversement proportionnel à ATR (volatilité haute = décline vite).',
    formule: 'ATR_norm = (ATR / 0.0008) capped at 1.0\nTimeout = 32 - (ATR_norm × 14)',
    inputs: ['ATR moyen du quarter', 'Référence: 0.0008'],
    output: {
      type: 'integer',
      range: '18 - 32',
      unite: 'minutes'
    },
    exemple: 'ATR=0.0004 (faible) → norm=0.5 → Timeout=32-(0.5×14)=25 min\nATR=0.0012 (élevé) → norm=1.0 → Timeout=32-(1.0×14)=18 min',
    notes: [
      'Volatilité basse → timeout long (volatilité décline lentement)',
      'Volatilité haute → timeout court (volatilité décline vite)',
      'Range: 18-32 minutes pour Forex M1'
    ]
  },

  // === WHIPSAW ===
  whipsaw_freq: {
    id: 'whipsaw_freq',
    titre: 'Whipsaw Frequency %',
    categorieId: 'whipsaw',
    definition: 'Pourcentage de trades où BOTH Buy Stop ET Sell Stop se déclenchent dans 15min (perte garantie).',
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
    titre: 'Whipsaw Risk Level',
    categorieId: 'whipsaw',
    definition: 'Catégorisation du risque basée sur la fréquence whipsaw.',
    formule: 'Risk = "Very Low" if % < 5\n      = "Low" if % < 10\n      = "Medium" if % < 20\n      = "High" if % < 35\n      = "Very High" if % ≥ 35',
    inputs: ['Whipsaw frequency %'],
    output: {
      type: 'string (enum)',
      range: '{VeryLow, Low, Medium, High, VeryHigh}',
      unite: 'catégorie'
    },
    exemple: '4.5% → Very Low risk',
    notes: [
      'Visuel: couleur verte/jaune/rouge correspondante',
      'Utilisé pour affichage BestSliceCard'
    ]
  },

  total_trades_simules: {
    id: 'total_trades_simules',
    titre: 'Total Trades Simulés',
    categorieId: 'whipsaw',
    definition: 'Nombre total d\'entrées potentielles analysées pour déterminer whipsaw frequency.',
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
    definition: 'Minutes jusqu\'au pic de volatilité après le début du quarter. Identifie le moment de la plus grande amplitude.',
    formule: 'Peak_min = argmax(ATR[i]) où i ∈ [0, quarter_duration]',
    inputs: ['ATR par minute', 'Time series'],
    output: {
      type: 'integer',
      range: '0 - quarter_duration',
      unite: 'minutes'
    },
    exemple: 'Peak ATR à minute 12 du quarter → Peak Duration = 12 min',
    notes: [
      'Typique Forex: 5-15 minutes après event',
      'Utile pour TP placement timing'
    ]
  },

  half_life: {
    id: 'half_life',
    titre: 'Volatility Half-life',
    categorieId: 'timing',
    definition: 'Temps jusqu\'à moitié du pic de volatilité. Mesure la décroissance exponentielle.',
    formule: 'Half_life = t où ATR(t) = peak_ATR / 2',
    inputs: ['ATR decay curve', 'Peak ATR value'],
    output: {
      type: 'integer',
      range: '0 - peak_duration',
      unite: 'minutes'
    },
    exemple: 'Peak=20 pips à 12min, ATR=10 pips à 18min → Half-life = 6 min (12→18)',
    notes: [
      'Exponentiel: volatilité décline vite puis lentement',
      'Détermine trade expiration logique'
    ]
  },

  trade_expiration: {
    id: 'trade_expiration',
    titre: 'Recommended Trade Expiration',
    categorieId: 'timing',
    definition: 'Durée totale recommandée du trade. Basée sur peak_duration avec buffer.',
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
    formule: 'Score = (w1×ATR_norm + w2×Range_norm + w3×Body% + w4×Noise + w5×Breakout) / sum(weights)',
    inputs: ['ATR%', 'Range%', 'Body%', 'Noise Ratio', 'Breakout%'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: 'points'
    },
    exemple: 'Score brut = 78/100 (avant whipsaw)',
    notes: [
      'Poids: emphasis sur volatilité et directionalité',
      'Base pour ajustement whipsaw'
    ]
  },

  score_ajuste: {
    id: 'score_ajuste',
    titre: 'Straddle Score (Ajusté)',
    categorieId: 'scores',
    definition: 'Score final après pénalité whipsaw. C\'est le vrai score de viabilité.',
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
  }
}
