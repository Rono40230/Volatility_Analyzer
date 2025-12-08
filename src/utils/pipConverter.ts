/**
 * Convertisseur Points MT5 ↔ Pips
 * 
 * MT5 utilise nativement les "points" (unité minimale).
 * Mais les traders parlent en "pips" (unité de cotation).
 * 
 * Tableau de conversion (basé sur norme MT5 officielle):
 * - Forex 5 décimales (EURUSD, GBPUSD, USDCAD, EURJPY, CADJPY, etc): 1 pip = 10 points
 * - Or (XAUUSD, XAUJPY): 1 pip = 10 points
 * - Argent (XAGUSD): 1 pip = 1000 points
 * - Indices (USA500IDXUSD): 1 pip = 1 point
 * - Crypto (BTCUSD, ETHUSD): 1 pip = 1 point
 */

type SymbolType = 'forex' | 'gold' | 'silver' | 'indices' | 'crypto'

/**
 * Déterminer le type de symbole pour obtenir le bon ratio points→pips
 */
function getSymbolType(symbol: string): SymbolType {
  if (symbol.includes('XAU')) return 'gold'
  if (symbol.includes('XAG')) return 'silver'
  if (symbol.includes('US30') || symbol.includes('DE30') || symbol.includes('NAS100') || symbol.includes('SPX500') || symbol.includes('USA500')) return 'indices'
  if (symbol.includes('BTC') || symbol.includes('ETH')) return 'crypto'
  return 'forex'  // Tous les autres: EURUSD, USDJPY, CADJPY, EURJPY, etc
}

/**
 * Obtenir le nombre de points par pip
 * points_per_pip = nombre de points qu'il faut pour faire 1 pip
 */
function getPointsPerPip(symbol: string): number {
  const type = getSymbolType(symbol)
  
  switch (type) {
    case 'gold':     return 10        // 1 pip = 10 points
    case 'silver':   return 1000      // 1 pip = 1000 points
    case 'indices':  return 1         // 1 pip = 1 point
    case 'crypto':   return 1         // 1 pip = 1 point
    default:         return 10        // Forex (tous): 1 pip = 10 points
  }
}

/**
 * Obtenir la pip_value pour un symbole (valeur minimale de variation)
 * Conservé pour compatibilité, mais utilise pointsPerPip en interne
 */
export function getPipValue(symbol: string): number {
  // Retourne une valeur arbitraire pour compatibilité
  // La vraie conversion se fait via getPointsPerPip
  return getPointsPerPip(symbol)
}

/**
 * Convertir les points MT5 en pips
 * Formula: pips = points / points_per_pip
 * 
 * @param symbol Symbole de trading (ex: "EURUSD", "BTCUSD", "CADJPY")
 * @param points Valeur en points MT5
 * @returns Valeur en pips
 * 
 * Exemples:
 * - EURUSD: 100 points ÷ 10 = 10 pips
 * - CADJPY: 10 points ÷ 10 = 1 pip
 * - XAGUSD: 1000 points ÷ 1000 = 1 pip
 * - BTCUSD: 100 points ÷ 1 = 100 pips
 */
export function pointsToPips(symbol: string, points: number): number {
  const pointsPerPip = getPointsPerPip(symbol)
  return points / pointsPerPip
}

/**
 * Convertir les pips en points MT5
 * Formula: points = pips × points_per_pip
 * 
 * @param symbol Symbole de trading
 * @param pips Valeur en pips
 * @returns Valeur en points MT5
 */
export function pipsToPoints(symbol: string, pips: number): number {
  const pointsPerPip = getPointsPerPip(symbol)
  return pips * pointsPerPip
}

/**
 * Formater une valeur en points avec conversion pips
 * Exemple: "150 points soit 15 pips"
 * 
 * @param symbol Symbole de trading
 * @param points Valeur en points
 * @param decimals Nombre de décimales (défaut: 2)
 * @returns Chaîne formatée
 */
export function formatPointsWithPips(symbol: string, points: number | undefined, decimals = 2): string {
  // Gérer les valeurs undefined ou NaN
  if (points === undefined || points === null || isNaN(points)) {
    return 'N/A'
  }
  
  const pips = pointsToPips(symbol, points)
  const pipsFormatted = Math.round(pips * 10) / 10  // Arrondir à 1 décimale
  const pointsFormatted = Math.round(points).toString()  // Arrondir sans décimales
  
  // Toujours afficher la conversion pips pour transparence
  return `${pointsFormatted} points (soit ${pipsFormatted} pips)`
}

/**
 * Obtenir uniquement la conversion pips pour affichage court
 * Exemple: "15 pips"
 * 
 * @param symbol Symbole de trading
 * @param points Valeur en points
 * @param decimals Nombre de décimales (défaut: 0)
 * @returns Chaîne formatée
 */
export function formatAsPips(symbol: string, points: number, decimals = 0): string {
  const pips = pointsToPips(symbol, points)
  return `${pips.toFixed(decimals)} pips`
}

/**
 * Déterminer si le symbole utilise une conversion (non-1:1)
 * Retourne true si pip_value !== 1.0
 */
export function hasConversion(symbol: string): boolean {
  return getPipValue(symbol) !== 1.0
}
