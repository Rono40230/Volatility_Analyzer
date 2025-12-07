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
    id: 'straddle',
    titre: 'Paramètres Straddle',
    emoji: '🎯',
    description: 'Configuration optimale du Straddle',
    formules: ['offset', 'take_profit', 'offset_ajuste', 'risk_level', 'meilleur_moment', 'win_rate_ajuste', 'trailing_stop', 'timeout']
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
    definition: 'Ratio Range / Body. Mesure le "bruit" (mèches) vs signal (direction). >3 = trop de bruit.',
    explication_litterale: 'Cette formule regarde si une chandelle a beaucoup de "queues" (wicks) par rapport à son corps. Si beaucoup de queues = marché bruyant (faux mouvements). Si peu de queues et corps gros = marché directionnel (vrai mouvement). Ratio > 3 = très bruyant (mauvais pour trader). Ratio < 1.5 = très directionnel (bon pour trader).',
    formule: 'Noise = (High - Low) / |Close - Open|',
    inputs: ['High', 'Low', 'Close', 'Open'],
    output: {
      type: 'float',
      range: '1.0 - ∞',
      unite: 'ratio'
    },
    exemple: 'Range=30 points, Body=10 points → Noise = 3.0 (bruit modéré)',
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
    titre: 'Volume Imbalance',
    categorieId: 'mouvement',
    definition: 'Déséquilibre entre acheteurs et vendeurs (Bid/Ask). Prédit la direction du prochain mouvement.',
    explication_litterale: 'Cette formule regarde s\'il y a plus d\'acheteurs ou plus de vendeurs. Quand beaucoup plus de gens veulent acheter que vendre, les prix montent généralement. Si beaucoup plus vendent que n\'achètent, les prix baissent. Ce déséquilibre nous dit où le marché veut aller.',
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

  // === STRADDLE PARAMETERS ===
  offset: {
    id: 'offset',
    titre: 'Offset (Distance ordres)',
    categorieId: 'straddle',
    definition: 'Distance des ordres Buy Stop et Sell Stop par rapport au prix d\'entrée. Basé sur ATR pour adapter à la volatilité. Fondation de tous les autres calculs (TP, SL, entrée).',
    explication_litterale: 'Cette formule calcule à quelle distance on place nos ordres d\'achat et de vente par rapport au prix actuel. On utilise la volatilité (ATR) pour adapter la distance: si le marché est très volatil, on met les ordres plus loin (pour éviter les faux déclenchements), si le marché est calme, on les met plus près (pour déclencher plus souvent).',
    formule: 'Offset = ATR_mean × 1.75\n\nArrondissement: .ceil() (pas de décimales)',
    inputs: ['ATR mean (moyenne volatilité 1h)', 'Arrondir vers le haut'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'ATR=24.5 points → Offset = 24.5 × 1.75 = 42.875 → arrondi = 43 points',
    notes: [
      'Multiplicateur 1.75 = balance optimal entre:',
      '  - Activations fréquentes (offset petit → mieux)',
      '  - SL/TP non trop serrés (offset grand → mieux)',
      'ATR faible → Offset petit (marché calme)',
      'ATR élevé → Offset grand (marché volatil)',
      'Fondation pour: TP (offset×2), SL (offset×ratio), Risk Level'
    ]
  },

  take_profit: {
    id: 'take_profit',
    titre: 'Take Profit (TP)',
    categorieId: 'straddle',
    definition: 'Distance du Take Profit depuis l\'entrée. Fixé à 2× l\'offset pour Straddle (rapport Risk:Reward 1:2).',
    explication_litterale: 'Cette formule décide à quel niveau on ferme notre position en profit. On double la distance de l\'offset: si nos ordres sont à 43 points, on ferme le profit à 86 points. C\'est simple: on risque 43 points (avec le SL) pour gagner 86 points. C\'est un rapport 1 contre 2, ce qui est équitable.',
    formule: 'TP = Offset × 2.0\n\nArrondissement: .ceil() (pas de décimales)',
    inputs: ['Offset calculé'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'Offset=43 points → TP = 43 × 2.0 = 86 points (arrondi)',
    notes: [
      'Ratio 1:2 = Risk:Reward classique pour Straddle',
      'Risk (SL) doit être ≥ Offset (pour absorber whipsaws)',
      'Reward (TP) = 2× Offset (pour équilibre)',
      'Exemple complet: Offset=43 points, SL=77 points, TP=86 points'
    ]
  },

  risk_level: {
    id: 'risk_level',
    titre: 'Risk Level (Niveau de risque)',
    categorieId: 'straddle',
    definition: 'Niveau de risque basé sur le ratio SL/Offset. Mesure l\'adéquation du Stop Loss par rapport à la distance d\'activation.',
    explication_litterale: 'Cette formule regarde si notre stop-loss (ligne de perte) est assez loin de l\'offset (distance des ordres). On divise le stop-loss par l\'offset pour voir le ratio. Si le ratio est grand (2.0+), le stop est très loin = peu de risque = vert 🟢. Si le ratio est moyen (1.5-2.0), c\'est acceptable = orange 🟡. Si le ratio est petit (<1.5), le stop est trop proche = beaucoup de risque = rouge 🔴.',
    formule: 'Ratio = SL_ajusté / Offset\n\nIF Ratio > 2.0 → 🟢 LOW\nELSE IF Ratio > 1.5 → 🟡 MEDIUM\nELSE → 🔴 HIGH',
    inputs: ['SL ajusté', 'Offset'],
    output: {
      type: 'enum',
      range: '{LOW, MEDIUM, HIGH}',
      unite: 'risk_level'
    },
    exemple: 'Offset=43, SL=77 → Ratio=77/43=1.79 → 🟡 MEDIUM',
    notes: [
      'Ratio > 2.0 = SL très large, peu de risque (vert)',
      'Ratio 1.5-2.0 = SL adéquat, risque modéré (orange)',
      'Ratio < 1.5 = SL trop serré, risque élevé (rouge)',
      'Exemples: Whipsaw 33% → 1.8 = MEDIUM | Whipsaw 8% → 2.5 = LOW'
    ]
  },

  offset_ajuste: {
    id: 'offset_ajuste',
    titre: 'SL Ajusté (Stop Loss)',
    categorieId: 'straddle',
    definition: 'Stop Loss pondéré par la fréquence whipsaw. Plus whipsaw est élevé, plus le SL est réduit (peu d\'espace). Plus whipsaw est bas, plus le SL est large (plus d\'espace).',
    explication_litterale: 'Cette formule calcule où on met notre \"cut-loss\" (niveau auquel on accepte la perte). On part de l\'offset, puis on le multiplie par un nombre qui dépend des faux déclenchements (whipsaw). Si beaucoup de faux déclenchements (33%), on multiplie par 1.8 seulement (stop plus proche). Si peu de faux déclenchements (3%), on multiplie par 2.8 (stop très loin). Logique: avec beaucoup de faux déclenchements, on n\'a pas besoin d\'un stop loin. Avec peu de faux déclenchements, on peut mettre un stop loin sans peur.',
    formule: 'SL_ajusté = Offset × ratio(whipsaw_freq)\n\nRatio par whipsaw:\n- Whipsaw >30% → ratio 1.5× (trop de faux déclenchements)\n- Whipsaw 20-30% → ratio 1.8× (équilibre)\n- Whipsaw 10-20% → ratio 2.2× (augmente SL)\n- Whipsaw 5-10% → ratio 2.5× (SL large)\n- Whipsaw <5% → ratio 2.8× (SL très large, peu de whipsaws)',
    inputs: ['SL brut (= Offset)', 'Whipsaw frequency %'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'Offset=43 points, Whipsaw=33.4% → ratio=1.8 → SL_ajusté = 43 × 1.8 = 77 points (arrondi)',
    notes: [
      'LOGIQUE: Whipsaw HAUT (30%+) → SL RÉDUIT (1.5×) car trop de faux déclenchements',
      'LOGIQUE: Whipsaw BAS (<5%) → SL AUGMENTÉ (2.8×) car peu de faux déclenchements',
      'Arrondi toujours vers le haut (.ceil()) = pas de décimales',
      'Exemple ancien: 20 × (1 + 0.25×0.3) = 21.5 ❌ OBSOLÈTE',
      'Maintenant: 20 × 2.2 = 44 points ✅ PLUS RÉALISTE'
    ]
  },

  meilleur_moment: {
    id: 'meilleur_moment',
    titre: 'Meilleur Moment (Entry Timing)',
    categorieId: 'straddle',
    definition: 'Minutes avant le début du quarter pour entrer. Basé sur analyse des moments de déclenchement whipsaw (trigger_minutes).',
    explication_litterale: 'Cette formule dit QUAND entrer exactement (à quel nombre de minutes). On regarde quand les faux déclenchements se produisent habituellement (par exemple à 8 minutes), puis on entre 60% plus tôt (à 5 minutes). C\'est notre assurance: on entre en avance pour éviter les pièges.',
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
    explication_litterale: 'Cette formule calcule combien de fois on gagne réellement. On commence avec un pourcentage de victoires théoriques, puis on le réduit en fonction des faux déclenchements. Si on gagne 55% en théorie mais qu\'il y a 20% de faux déclenchements, on réduit: 55 × (1 - 0.20) = 44%. C\'est plus réaliste et honnête.',
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
    explication_litterale: 'Cette formule calcule un "stop qui suit le profit". Au lieu d\'un stop fixe, le stop se rapproche du prix au fur et à mesure que le profit augmente. On part d\'une valeur de base (1.59), puis on la réduit si beaucoup de faux déclenchements (pour être plus prudent). Si peu de faux déclenchements, on garde le stop plus agressif.',
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
    explication_litterale: 'Cette formule dit combien de minutes on peut tenir notre position. Si le marché est très volatil (beaucoup de mouvement), la volatilité va baisser vite, donc on ferme rapidement (18 minutes). Si le marché est calme (peu de mouvement), la volatilité va baisser lentement, donc on peut rester plus longtemps (32 minutes). C\'est logique: quand ça bouge beaucoup, ça se calme vite. Quand ça bouge peu, ça prend du temps.',
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
    titre: 'Whipsaw Risk Level',
    categorieId: 'whipsaw',
    definition: 'Catégorisation du risque basée sur la fréquence whipsaw.',
    explication_litterale: 'C\'est mon jugement sur le RISQUE de whipsaw à cette heure. "Very Low" = presque aucun risque. "High" = beaucoup de risque. Elle te dit: comment prudent dois-tu être cette heure? Si Risk="Very High", ça veut dire quasiment 1 fois sur 3, ton Straddle va se déclencher dans les deux sens = perdu.',
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
    definition: 'Minutes jusqu\'au pic de volatilité après le début du quarter. Identifie le moment de la plus grande amplitude.',
    explication_litterale: 'Cette formule regarde QUAND le marché bouge le plus après un événement. Si c\'est à 3 minutes = le gros mouvement arrive vite. Si c\'est à 30 minutes = le marché prend du temps à réagir. C\'est utile pour savoir quand placer ton Straddle pour attraper le bon moment.',
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
    explication_litterale: 'Cette formule mesure combien de temps il faut pour que la volatilité descende à la moitié du maximum. Si peak=100 points, half-life=5 minutes = à 5 minutes le marché bouge encore 50 points en moyenne. À 10 minutes = 25 points. Elle te dit quand ta position perd de la valeur.',
    formule: 'Half_life = t où ATR(t) = peak_ATR / 2',
    inputs: ['ATR decay curve', 'Peak ATR value'],
    output: {
      type: 'integer',
      range: '0 - peak_duration',
      unite: 'minutes'
    },
    exemple: 'Peak=20 points à 12min, ATR=10 points à 18min → Half-life = 6 min (12→18)',
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
      'Poids: emphasis sur volatilité et directionalité',
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
      'Si biaisé, utiliser pour stratégies directionnelles'
    ]
  }
}
