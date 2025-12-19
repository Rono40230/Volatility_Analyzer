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
    formules: ['offset', 'take_profit', 'offset_ajuste', 'risk_level', 'meilleur_moment', 'win_rate_ajuste', 'trailing_stop', 'sl_recovery', 'timeout']
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
    description: 'Métriques de performance issues des simulations',
    formules: ['win_rate', 'profit_factor', 'max_drawdown', 'average_pips']
  },
  {
    id: 'conversion',
    titre: 'Tableau de conversion',
    emoji: '🔄',
    description: 'Conversion Points MT5 vers Pips',
    formules: []
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
    definition: 'Distance des ordres Buy Stop et Sell Stop. Logique "Bidi V4 Hardened" pour éviter les mèches.',
    explication_litterale: 'On place les ordres assez loin pour ne pas être déclenché par le bruit. Si le marché est très bruyant (Noise > 2.5), on s\'écarte beaucoup (ATR x3). Sinon, on reste prudent (ATR x2). On ajoute toujours une marge de sécurité (Spread).',
    formule: 'IF Noise > 2.5 → Offset = (ATR × 3.0) + Spread\nELSE → Offset = (ATR × 2.0) + Spread',
    inputs: ['ATR', 'Noise Ratio', 'Spread (def: 3.0)'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'ATR=10, Noise=2.8 → Offset = (10 × 3.0) + 3 = 33 points',
    notes: [
      'Logique V4 Durcie (fini le 1.5x trop risqué)',
      'Filtre agressivement les faux départs',
      'Priorité: Sécurité > Opportunité'
    ]
  },

  take_profit: {
    id: 'take_profit',
    titre: 'Take Profit (Target)',
    categorieId: 'straddle',
    definition: 'Objectif de profit théorique. Dans Bidi V2, la sortie est principalement gérée par le Trailing Stop, mais le TP sert de sécurité ou d\'objectif Risk:Reward.',
    explication_litterale: 'Bien que le robot utilise un Trailing Stop pour laisser courir les gains, on définit un Take Profit de sécurité. Il est généralement placé à 2 fois la distance du Stop Loss, assurant un ratio Risk:Reward de 1:2.',
    formule: 'TP = Stop Loss × 2.0',
    inputs: ['Stop Loss'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'SL=40 points → TP = 80 points',
    notes: [
      'Ratio 1:2 = Standard Risk:Reward',
      'Souvent non atteint car Trailing Stop sort avant',
      'Sert de "Home Run" target'
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
    titre: 'Stop Loss (SL) Adaptatif',
    categorieId: 'straddle',
    definition: 'Niveau de protection élargi (Bidi V4). Plus le bruit est fort, plus le SL est large pour survivre au Whipsaw.',
    explication_litterale: 'Le SL s\'adapte à la violence du marché. Si c\'est calme, on utilise ATR x2.5. Si c\'est le chaos (Noise > 3.5), on élargit énormément (ATR x5.0) pour ne pas se faire sortir sur un aller-retour rapide.',
    formule: 'Noise > 3.5 → SL = ATR × 5.0\nNoise > 2.5 → SL = ATR × 4.0\nNoise > 2.0 → SL = ATR × 3.0\nElse → SL = ATR × 2.5',
    inputs: ['ATR', 'Noise Ratio'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'ATR=10, Noise=3.6 → SL = 10 × 5.0 = 50 points',
    notes: [
      'Logique V4 Élargie',
      'But: Survivre à la volatilité initiale',
      'Minimum 2.5x ATR (vs 1.5x avant)'
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
    titre: 'Trailing Stop (Suivi)',
    categorieId: 'straddle',
    definition: 'Stop suiveur adaptatif. Sécurise les gains en remontant le SL à mesure que le prix avance.',
    explication_litterale: 'Le Trailing Stop suit le prix comme une ombre. Si le marché est calme (Noise < 1.5), il suit de près (1.0x ATR) pour verrouiller vite les gains. Si le marché est nerveux (Noise > 3.0), il laisse plus de marge (2.0x ATR) pour ne pas sortir trop tôt sur une correction mineure.',
    formule: 'Noise > 3.0 → TS = ATR × 2.0\nNoise > 2.0 → TS = ATR × 1.5\nNoise > 1.5 → TS = ATR × 1.2\nElse → TS = ATR × 1.0',
    inputs: ['ATR', 'Noise Ratio'],
    output: {
      type: 'float',
      range: '1.0 - 2.0',
      unite: 'x ATR'
    },
    exemple: 'ATR=20, Noise=1.2 → TS = 20 × 1.0 = 20 points\nATR=20, Noise=2.5 → TS = 20 × 1.5 = 30 points',
    notes: [
      'Adaptatif selon le bruit',
      'Plus le bruit est fort, plus le TS est large',
      'Permet de laisser courir les gains sur les gros mouvements'
    ]
  },

  sl_recovery: {
    id: 'sl_recovery',
    titre: 'SL Recovery (Mode Panique)',
    categorieId: 'straddle',
    definition: 'Stop Loss de secours en cas de mouvement violent inverse. Assure que le SL couvre au moins 3 fois l\'offset.',
    explication_litterale: 'C\'est une sécurité supplémentaire. Parfois, le SL calculé normalement est trop proche si le marché fait un "gap" violent. Cette formule force le SL à être au moins 3 fois plus loin que l\'entrée (Offset). C\'est le "filet de sécurité" ultime.',
    formule: 'SL_Recovery = max(SL, Offset × 3.0)',
    inputs: ['Stop Loss', 'Offset'],
    output: {
      type: 'float',
      range: '0.0 - ∞',
      unite: 'points'
    },
    exemple: 'Offset=10, SL=20 → SL_Recovery = max(20, 30) = 30 points',
    notes: [
      'Sécurité anti-gap',
      'Garantit un espace de respiration minimal',
      'Activé surtout quand l\'Offset est très petit'
    ]
  },

  timeout: {
    id: 'timeout',
    titre: 'Timeout (Durée position)',
    categorieId: 'straddle',
    definition: 'Durée maximale pour tenir la position. Dynamique basé sur la décroissance de volatilité (Half-Life).',
    explication_litterale: 'Pour le trading d\'annonces, l\'impulsion est rapide. On utilise la "demi-vie" de la volatilité pour savoir quand sortir. Si la volatilité retombe vite, on sort vite. Sinon, on garde jusqu\'à 15 minutes max. Par défaut 3 minutes si pas de données.',
    formule: 'Timeout = Half-Life (clamped 1-15 min) OR 3 min (default)',
    inputs: ['Half-Life', 'Default (3 min)'],
    output: {
      type: 'integer',
      range: '1 - 15',
      unite: 'minutes'
    },
    exemple: 'Half-Life=5 min → Timeout=5 min | Half-Life=20 min → Timeout=15 min',
    notes: [
      'Optimisé pour capturer l\'impulsion principale',
      'Évite le "time decay" et le retournement',
      'Min 1 min, Max 15 min'
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
      'Si biaisé, utiliser pour stratégies directionnelles'
    ]
  },

  // === BACKTEST & PERFORMANCE ===
  win_rate: {
    id: 'win_rate',
    titre: 'Win Rate (Taux de réussite)',
    categorieId: 'backtest',
    definition: 'Pourcentage de trades gagnants par rapport au nombre total de trades exécutés.',
    explication_litterale: 'Cette formule calcule simplement combien de fois la stratégie a gagné de l\'argent. Si on a fait 100 trades et gagné 60 fois, le Win Rate est de 60%. C\'est l\'indicateur de base de la fiabilité.',
    formule: 'Win Rate = (Winning Trades / Total Trades) × 100',
    inputs: ['Winning Trades', 'Total Trades'],
    output: {
      type: 'float',
      range: '0 - 100',
      unite: '%'
    },
    exemple: '60 gagnants / 100 total = 60%',
    notes: [
      '> 50% est généralement requis pour être profitable (sauf si Risk:Reward très élevé)',
      'Inclut les trades directionnels et simultanés'
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
  }
}
