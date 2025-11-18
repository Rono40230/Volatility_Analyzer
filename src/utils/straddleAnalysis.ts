// src/utils/straddleAnalysis.ts - Logique d'analyse STRADDLE par tranche 15min

import type { Stats15Min } from '../stores/volatility'

/**
 * Résultat d'une tranche 15min avec son score straddle
 */
export interface Slice15minWithScore {
  hour: number
  quarter: number
  startTime: string // "01:45-02:00"
  stats: Stats15Min
  straddleScore: number // 0-100
}

/**
 * Informations d'un golden combo détecté
 */
export interface GoldenCombo {
  name: string
  description: string
  confidence: 'JACKPOT' | 'EXCELLENT' | 'BON' | 'MOYEN' | 'FAIBLE'
  winRate: number
  avgGainR: number
}

/**
 * Piège détecté dans une tranche
 */
export interface DetectedTrap {
  name: string
  description: string
  severity: 'CRITIQUE' | 'HAUTE' | 'MOYENNE' | 'BASSE'
  metric: string
  value: number
  threshold: number
  recommendation: string
}

/**
 * Plan d'action complet pour une tranche 15min
 */
export interface TradingPlan {
  entryTime: string
  slPips: number
  slPoints: number
  slUsd: number
  tpPips: number
  tpPoints: number
  tpUsd: number
  positionSize: number // % du risque (50-150%)
  riskReward: string // "1:1.5"
  winProbability: number // %
  avgGainR: number
  maxDuration: number // minutes
  trailingStopActivation: string // "+0.5R"
  trailingStopCoefficient: number // ATR multiplier (2.0 par défaut)
  tradeDurationMinutes: number // Phase 1.1: Durée optimale du trade (120-300)
  tradeExpiration: number // Phase 1.1: Identique à tradeDurationMinutes
}

/**
 * Analyse complète d'une tranche 15min
 */
export interface SliceAnalysis {
  rank: number // 1, 2, 3
  slice: Slice15minWithScore
  goldenCombos: GoldenCombo[]
  traps: DetectedTrap[]
  tradingPlan: TradingPlan
}

/**
 * Calcule le score STRADDLE pour une tranche 15min
 * Basé sur la même logique que le backend Rust
 */
export function calculateStraddleScore(
  slice: Stats15Min
): number {
  if (slice.candle_count === 0) return 0

  let score = 0

  // 1. RANGE (60 pts max) - Dominante pour straddle
  if (slice.range_mean > 0.0025) {
    score += 60 // >25 pips = excellent
  } else if (slice.range_mean > 0.0020) {
    score += 50 // 20-25 pips = très bon
  } else if (slice.range_mean > 0.0015) {
    score += 40 // 15-20 pips = bon
  } else if (slice.range_mean > 0.0010) {
    score += 20 // 10-15 pips = acceptable
  }

  // 2. ATR (25 pts max) - Volatilité soutenue
  if (slice.atr_mean > 0.0020) {
    score += 25 // >20 pips = excellent
  } else if (slice.atr_mean > 0.0015) {
    score += 20 // 15-20 pips = très bon
  } else if (slice.atr_mean > 0.0010) {
    score += 15 // 10-15 pips = bon
  } else if (slice.atr_mean > 0.0005) {
    score += 8 // 5-10 pips = acceptable
  }

  // 3. BodyRange (15 pts max) - Directionnalité
  if (slice.body_range_mean > 45.0) {
    score += 15 // >45% = excellent
  } else if (slice.body_range_mean > 35.0) {
    score += 12 // 35-45% = bon
  } else if (slice.body_range_mean > 25.0) {
    score += 8 // 25-35% = acceptable
  } else if (slice.body_range_mean > 15.0) {
    score += 3 // 15-25% = limite
  }

  return Math.min(score, 100)
}

/**
 * Détecte les golden combos pour une tranche 15min
 */
export function detectGoldenCombos(
  slice: Stats15Min
): GoldenCombo[] {
  const combos: GoldenCombo[] = []

  // COMBO 1: JACKPOT STRADDLE
  if (
    slice.range_mean > 0.0025 &&
    slice.atr_mean > 0.0020 &&
    slice.body_range_mean > 45.0 &&
    slice.noise_ratio_mean < 2.0 &&
    slice.tick_quality_mean > 0.001 &&
    slice.breakout_percentage > 15.0
  ) {
    combos.push({
      name: 'JACKPOT STRADDLE',
      description:
        'Range énorme + ATR élevé + Signal pur (BR>45%, NR<2.0) + Liquidité excellente',
      confidence: 'JACKPOT',
      winRate: 0.8,
      avgGainR: 4.0
    })
  }

  // COMBO 2: EXCELLENT + SIGNAL PUR
  if (
    slice.range_mean > 0.002 &&
    slice.atr_mean > 0.0015 &&
    slice.body_range_mean > 35.0 &&
    slice.noise_ratio_mean < 2.0
  ) {
    combos.push({
      name: 'EXCELLENT + SIGNAL PUR',
      description: 'Très bon mouvement + Signal très propre (NR<2.0)',
      confidence: 'EXCELLENT',
      winRate: 0.75,
      avgGainR: 3.0
    })
  }

  // COMBO 3: DIRECTIONNEL FORT
  if (
    slice.body_range_mean > 45.0 &&
    slice.noise_ratio_mean < 2.0 &&
    (slice.volume_imbalance_mean > 2.0 || slice.volume_imbalance_mean < 0.5)
  ) {
    combos.push({
      name: 'DIRECTIONNEL FORT',
      description:
        'Signal très pur + Tendance marquée (Imbalance déséquilibré). Scalping directionnel optimal.',
      confidence: 'EXCELLENT',
      winRate: 0.78,
      avgGainR: 3.5
    })
  }

  // COMBO 4: LIQUIDITÉ OPTIMALE
  if (slice.tick_quality_mean > 0.001 && slice.noise_ratio_mean < 2.0) {
    combos.push({
      name: 'LIQUIDITÉ OPTIMALE',
      description:
        'Spreads serrés (<1 pip) + Signal pur = Profit maximal par pip risqué',
      confidence: 'EXCELLENT',
      winRate: 0.65,
      avgGainR: 2.0
    })
  }

  // COMBO 5: BON SCALP STANDARD
  if (
    slice.range_mean > 0.0015 &&
    slice.atr_mean > 0.001 &&
    slice.body_range_mean > 25.0 &&
    slice.noise_ratio_mean < 3.0
  ) {
    combos.push({
      name: 'BON SCALP STANDARD',
      description: 'Conditions acceptables pour scalpe normal avec risque modéré',
      confidence: 'BON',
      winRate: 0.6,
      avgGainR: 2.0
    })
  }

  return combos
}

/**
 * Détecte les pièges potentiels
 */
export function detectTraps(
  slice: Stats15Min
): DetectedTrap[] {
  const traps: DetectedTrap[] = []

  // PIÈGE 1: FAUX MOUVEMENT (ATR élevé mais BR faible)
  if (slice.atr_mean > 0.0015 && slice.body_range_mean < 20.0) {
    traps.push({
      name: 'FAUX MOUVEMENT',
      description:
        'ATR élevé mais très peu de mouvement net (BR<20%). Spikes isolés = whipsaws fréquents.',
      severity: 'HAUTE',
      metric: 'BodyRange',
      value: slice.body_range_mean,
      threshold: 20.0,
      recommendation: 'Réduire position size à 25%, ou augmenter SL à ATR×3.0'
    })
  }

  // PIÈGE 2: CHAOS TOTAL
  if (
    slice.noise_ratio_mean > 3.5 &&
    slice.body_range_mean < 15.0 &&
    slice.breakout_percentage < 5.0
  ) {
    traps.push({
      name: 'CHAOS TOTAL',
      description:
        'Beaucoup de bruit + peu de vraie direction. Whipsaws constants = SL touch fréquent.',
      severity: 'CRITIQUE',
      metric: 'NoiseRatio',
      value: slice.noise_ratio_mean,
      threshold: 3.5,
      recommendation: 'SKIP cette tranche. Chercher autre opportunité.'
    })
  }

  // PIÈGE 3: INDÉCISION
  if (
    slice.volume_imbalance_mean >= 0.8 &&
    slice.volume_imbalance_mean <= 1.2 &&
    slice.volatility_mean < 10.0 &&
    slice.range_mean < 0.001
  ) {
    traps.push({
      name: 'INDÉCISION',
      description:
        'Marché équilibré (Imbalance ≈1.0) + peu de volume. Pas d\'avantage directionnel pour straddle.',
      severity: 'HAUTE',
      metric: 'Imbalance',
      value: slice.volume_imbalance_mean,
      threshold: 0.9,
      recommendation: 'Pas de trading directionnel agressif. Straddle non-optimal.'
    })
  }

  // PIÈGE 4: SPREADS PROHIBITIFS
  if (slice.tick_quality_mean < 0.0001) {
    traps.push({
      name: 'SPREADS PROHIBITIFS',
      description:
        'Tick Quality très faible = spreads énormes. Tous les gains scalp mangés par les spreads.',
      severity: 'CRITIQUE',
      metric: 'TickQuality',
      value: slice.tick_quality_mean,
      threshold: 0.0001,
      recommendation:
        'SKIP ou augmenter TP à ATR×4.0 pour compenser. Position size 50% max.'
    })
  }

  // PIÈGE 5: RANGE INSUFFISANT POUR STRADDLE
  if (slice.range_mean < 0.001) {
    traps.push({
      name: 'RANGE INSUFFISANT',
      description: `Range ${(slice.range_mean * 10000).toFixed(0)} pips << ${(0.0025 * 10000).toFixed(0)} pips minimum. Pas assez de mouvement pour straddle.`,
      severity: 'HAUTE',
      metric: 'Range',
      value: slice.range_mean,
      threshold: 0.0025,
      recommendation: 'SKIP cette tranche. Range trop faible pour stratégie straddle.'
    })
  }

  return traps
}

/**
 * Calcule le coefficient de trailing stop dynamique basé sur la volatilité relative
 * Formule: 1.5 + (ATR_courant / ATR_moyen - 1) * 0.5
 * Plage: [1.5, 2.5]
 */
function calculateDynamicTrailingStopCoefficient(
  currentAtr: number,
  averageAtr: number
): number {
  if (averageAtr === 0) return 2.0 // Fallback

  const volatilityRatio = currentAtr / averageAtr
  const coefficient = 1.5 + (volatilityRatio - 1) * 0.5

  // Limiter entre 1.5 et 2.5
  return Math.max(1.5, Math.min(2.5, coefficient))
}

/**
 * Calcule le plan d'action complet
 */
export function calculateTradingPlan(
  slice: Stats15Min,
  goldenCombos: GoldenCombo[],
  traps: DetectedTrap[],
  averageAtr: number = slice.atr_mean // Default: utiliser l'ATR courant
): TradingPlan {
  const atr = slice.atr_mean
  const tickQuality = slice.tick_quality_mean

  // Déterminer les multiplicateurs en fonction des conditions
  let slMultiplier = 2.0 // Par défaut
  let tpMultiplier = 3.0 // Par défaut
  let positionSize = 100 // % du risque

  // Si JACKPOT combo
  if (goldenCombos.some((c) => c.confidence === 'JACKPOT')) {
    slMultiplier = 1.5 // SL serré, confiance élevée
    tpMultiplier = 3.5
    positionSize = 100
  }
  // Si EXCELLENT combo
  else if (goldenCombos.some((c) => c.confidence === 'EXCELLENT')) {
    slMultiplier = 2.0
    tpMultiplier = 3.0
    positionSize = 100
  }
  // Si BON combo
  else if (goldenCombos.some((c) => c.confidence === 'BON')) {
    slMultiplier = 2.5
    tpMultiplier = 3.0
    positionSize = 75
  } else {
    slMultiplier = 3.0
    tpMultiplier = 3.0
    positionSize = 50
  }

  // Ajustements pièges
  if (traps.some((t) => t.severity === 'CRITIQUE')) {
    positionSize = Math.max(25, positionSize - 25) // Réduire fortement
  } else if (traps.some((t) => t.severity === 'HAUTE')) {
    positionSize = Math.max(50, positionSize - 15)
  }

  // Ajustements liquidité
  if (tickQuality > 0.001) {
    positionSize = Math.min(150, positionSize + 20) // Augmenter si spreads serrés
  } else if (tickQuality < 0.0001) {
    positionSize = 50 // Réduire fortement si spreads énormes
  }

  // Calculs SL/TP en pips
  const slPips = atr * slMultiplier * 10000 // Convertir en pips
  const tpPips = atr * tpMultiplier * 10000

  // Conversion en POINTS (1 point = 0.0001 pour forex M1)
  const slPoints = slPips * 10
  const tpPoints = tpPips * 10

  // Estimation USD (exemple: risque 100$ par trade)
  // À adapter selon la vraie taille de compte
  const estimatedRiskUsd = 50 // $ (exemple)
  const slUsd = estimatedRiskUsd
  const tpUsd = estimatedRiskUsd * (tpPips / slPips)

  // Calcul RR
  const riskReward = `1:${(tpPips / slPips).toFixed(1)}`

  // Win rate et gain moyen
  let winProbability = 50
  let avgGainR = 1.0
  if (goldenCombos.length > 0) {
    const bestCombo = goldenCombos[0]
    winProbability = Math.round(bestCombo.winRate * 100)
    avgGainR = bestCombo.avgGainR
  }

  // Calculer le coefficient de trailing stop dynamique
  const trailingStopCoefficient = calculateDynamicTrailingStopCoefficient(
    atr,
    averageAtr
  )

  return {
    entryTime: '-2min (avant fermeture)',
    slPips: Math.round(slPips),
    slPoints: Math.round(slPoints),
    slUsd: Math.round(slUsd * 100) / 100,
    tpPips: Math.round(tpPips),
    tpPoints: Math.round(tpPoints),
    tpUsd: Math.round(tpUsd * 100) / 100,
    positionSize,
    riskReward,
    winProbability,
    avgGainR,
    maxDuration: 300, // 5h (ancien paramètre, remplacé par tradeDurationMinutes)
    trailingStopActivation: '+0.5R',
    trailingStopCoefficient, // Dynamique basé sur volatilité relative
    tradeDurationMinutes: 180, // Default: 3h (recalculé lors du binding avec BidiParameters)
    tradeExpiration: 180 // Phase 1.1: Identique à tradeDurationMinutes
  }
}

/**
 * Trouve les 3 meilleures tranches 15min de la journée
 */
export function findTop3Slices(
  stats15min: Stats15Min[]
): Slice15minWithScore[] {
  // Calculer le score pour chaque tranche
  const scoredSlices: Slice15minWithScore[] = stats15min.map((stat) => ({
    hour: stat.hour,
    quarter: stat.quarter,
    startTime: formatSliceTime(stat.hour, stat.quarter),
    stats: stat,
    straddleScore: calculateStraddleScore(stat)
  }))

  // Trier par score décroissant
  scoredSlices.sort((a, b) => b.straddleScore - a.straddleScore)

  // Retourner top 3
  return scoredSlices.slice(0, 3)
}

/**
 * Crée l'analyse complète pour les TOP 3
 */
export function analyzeTop3Slices(
  stats15min: Stats15Min[]
): SliceAnalysis[] {
  const topSlices = findTop3Slices(stats15min)

  // Calculer l'ATR moyen sur toutes les tranches pour la volatilité relative
  const averageAtr =
    stats15min.length > 0
      ? stats15min.reduce((sum, stat) => sum + stat.atr_mean, 0) /
        stats15min.length
      : 0

  return topSlices.map((slice, index) => {
    const goldenCombos = detectGoldenCombos(slice.stats)
    const traps = detectTraps(slice.stats)
    const tradingPlan = calculateTradingPlan(
      slice.stats,
      goldenCombos,
      traps,
      averageAtr
    )

    return {
      rank: index + 1,
      slice,
      goldenCombos,
      traps,
      tradingPlan
    }
  })
}

/**
 * Formate l'heure d'une tranche 15min
 */
function formatSliceTime(hour: number, quarter: number): string {
  const startMin = quarter * 15
  const endMin = Math.min(startMin + 15, 60)
  const nextHour = endMin === 60 ? (hour + 1) % 24 : hour

  return `${String(hour).padStart(2, '0')}:${String(startMin).padStart(2, '0')}-${String(nextHour).padStart(2, '0')}:${String(endMin).padStart(2, '0')}`
}

/**
 * Détermine si une tranche est dans le TOP 3
 */
export function isInTop3(
  hour: number,
  quarter: number,
  top3Slices: Slice15minWithScore[]
): boolean {
  return top3Slices.some((s) => s.hour === hour && s.quarter === quarter)
}

/**
 * Obtient le rang si dans TOP 3, sinon -1
 */
export function getTop3Rank(
  hour: number,
  quarter: number,
  top3Slices: Slice15minWithScore[]
): number {
  const index = top3Slices.findIndex((s) => s.hour === hour && s.quarter === quarter)
  return index !== -1 ? index + 1 : -1
}

/**
 * Paramètres optimisés pour le robot Bidi
 * 🔐 RiskPercent est TOUJOURS 1.0 (constante immuable)
 * 
 * FOCUS: 4 paramètres opérationnels
 * - EventTime: Heure de déclenchement (à la minute près)
 * - StopLossLevelPercent: Distance SL en points concrets
 * - ATRMultiplier: Agressivité du trailing stop
 * - TradeDurationMinutes: Durée optimale du trade basée sur la volatilité
 */
export interface BidiParameters {
  // 📍 HEURE EXACTE DE DÉCLENCHEMENT
  eventTime: string // HH:MM:SS (ex: "14:29:50")
  eventTimeExplanation: string // Explication simple

  // 🛑 POSITIONNEMENT STOP LOSS
  stopLossLevelPercent: number // % de l'ATR (25-40%)
  stopLossPoints: number // Nombre de points concrets (ex: 15)
  atrMeanForCalculation: number // ATR du créneau pour la calculatrice
  stopLossExplanation: string // Pourquoi cette distance

  // 📈 AGRESSIVITÉ DU TRAILING STOP
  atrMultiplier: number // Multiplicateur (1.5-3.0)
  trailingStepPoints: number // Nombre de points par tick (ex: 125)
  atrMultiplierProfile: string // "Agressif" | "Normal" | "Généreux" | "Très Généreux"
  atrMultiplierExplanation: string // Comportement du trailing

  // ⏱️ DURÉE DU TRADE (Phase 1.1 - VolatilityDuration)
  tradeDurationMinutes: number // Durée optimale en minutes (120-300)
  tradeDurationExplanation: string // Pourquoi cette durée

  // 🔐 CONSTANTES (informel)
  riskPercent: 1.0
  tradeExpiration: number // Dynamique = tradeDurationMinutes
}

/**
 * 🎯 Calcule les paramètres optimisés pour Bidi basés sur l'analyse d'une opportunité TOP 3
 *
 * FOCUS: 3 paramètres opérationnels uniquement
 * - EventTime: Heure exacte du trigger
 * - StopLossLevelPercent: Distance SL en points
 * - ATRMultiplier: Agressivité du trailing
 *
 * @param slice Tranche 15min analysée avec son score
 * @param allSlices Toutes les tranches pour contexte
 * @returns BidiParameters simplifiés et concrets
 */
export function calculateBidiParameters(
  slice: Slice15minWithScore,
  allSlices: Slice15minWithScore[]
): BidiParameters {
  const stats = slice.stats

  // ========================================
  // 1️⃣ EVENT TIME - Heure de déclenchement
  // ========================================
  const hours = slice.hour < 10 ? `0${slice.hour}` : `${slice.hour}`
  const minutes = slice.quarter === 1 ? '00' : slice.quarter === 2 ? '15' : slice.quarter === 3 ? '30' : '45'
  const eventTime = `${hours}:${minutes === '45' ? '59' : minutes}:50`
  const eventTimeExplanation = `Signal déclenché à la fin du créneau optimal (${slice.startTime})`

  // ========================================
  // 2️⃣ STOP LOSS - Distance en points concrets
  // ========================================
  // Récupère ATR (potentiellement en décimales, convertir en points si nécessaire)
  let atrMean = stats.atr_mean || 30
  
  // Si ATR est très petit (< 1), c'est probablement en décimales (0.0015) → convertir en points
  if (atrMean < 1) {
    atrMean = atrMean * 10000 // 0.0015 × 10000 = 15 points
  }
  
  const rangeAvg = stats.range_mean || 30
  
  let stopLossLevelPercent = 30.0
  let stopLossExplanation = ''

  if (rangeAvg > 60) {
    stopLossLevelPercent = 25.0
    stopLossExplanation = `Range large (${rangeAvg.toFixed(0)}pts) → SL serré pour capturer mouvements rapides`
  } else if (rangeAvg > 40) {
    stopLossLevelPercent = 30.0
    stopLossExplanation = `Range modéré (${rangeAvg.toFixed(0)}pts) → SL équilibré`
  } else if (rangeAvg > 20) {
    stopLossLevelPercent = 35.0
    stopLossExplanation = `Range moyen (${rangeAvg.toFixed(0)}pts) → SL un peu lâche`
  } else {
    stopLossLevelPercent = 40.0
    stopLossExplanation = `Range serré (${rangeAvg.toFixed(0)}pts) → SL très lâche pour éviter fausses sorties`
  }

  // Calcul en points concrets
  const stopLossPoints = Math.round((stopLossLevelPercent / 100) * atrMean)

  // ========================================
  // 3️⃣ ATR MULTIPLIER - Agressivité du trailing
  // ========================================
  let atrMultiplier = 2.0
  let atrMultiplierProfile = 'Normal'

  if (atrMean > 50) {
    atrMultiplier = 1.5
    atrMultiplierProfile = 'Agressif'
  } else if (atrMean > 40) {
    atrMultiplier = 2.0
    atrMultiplierProfile = 'Normal'
  } else if (atrMean > 25) {
    atrMultiplier = 2.5
    atrMultiplierProfile = 'Généreux'
  } else {
    atrMultiplier = 3.0
    atrMultiplierProfile = 'Très Généreux'
  }

  // Calcul du trailing step en points
  const trailingStepPoints = Math.round(atrMultiplier * atrMean)
  const atrMultiplierExplanation = `À chaque tick, le TSL monte de ${trailingStepPoints}pts (irréversible). Profil: ${atrMultiplierProfile} (volatilité ${atrMean.toFixed(0)}pts)`

  // ========================================
  // 4️⃣ TRADE DURATION - Durée optimale basée sur volatilité
  // ========================================
  // Heuristique basée sur ATR et volatilité observée
  // Volatilité haute = pic court (120-150min)
  // Volatilité moyenne = pic modéré (150-210min)
  // Volatilité faible = plateau long (210-270min)
  let tradeDurationMinutes = 180 // Default: 3h
  let tradeDurationExplanation = ''

  if (atrMean > 50) {
    // Volatilité très élevée = pic intense et court
    tradeDurationMinutes = Math.min(150, Math.round(120 + (atrMean - 50) * 0.5))
    tradeDurationExplanation = `Volatilité très élevée (${atrMean.toFixed(0)}pts) → Pic court ${tradeDurationMinutes}min. Fermer avant effondrement.`
  } else if (atrMean > 40) {
    // Volatilité bonne
    tradeDurationMinutes = Math.min(170, Math.max(150, Math.round(140 + (atrMean - 40) * 1.0)))
    tradeDurationExplanation = `Volatilité élevée (${atrMean.toFixed(0)}pts) → Pic clair ${tradeDurationMinutes}min.`
  } else if (atrMean > 25) {
    // Volatilité acceptable
    tradeDurationMinutes = Math.min(210, Math.max(180, Math.round(180 + (atrMean - 25) * 0.5)))
    tradeDurationExplanation = `Volatilité acceptable (${atrMean.toFixed(0)}pts) → Plateau modéré ${tradeDurationMinutes}min.`
  } else {
    // Volatilité faible = plateau prolongé
    tradeDurationMinutes = 240
    tradeDurationExplanation = `Volatilité faible (${atrMean.toFixed(0)}pts) → Plateau long ${tradeDurationMinutes}min.`
  }

  // Limiter entre 120 et 300 minutes
  tradeDurationMinutes = Math.max(120, Math.min(300, tradeDurationMinutes))

  return {
    eventTime,
    eventTimeExplanation,
    stopLossLevelPercent,
    stopLossPoints,
    atrMeanForCalculation: Math.round(atrMean),
    stopLossExplanation,
    atrMultiplier,
    trailingStepPoints,
    atrMultiplierProfile,
    atrMultiplierExplanation,
    tradeDurationMinutes,
    tradeDurationExplanation,
    riskPercent: 1.0,
    tradeExpiration: tradeDurationMinutes
  }
}
