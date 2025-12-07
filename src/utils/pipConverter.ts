/**
 * Convertisseur Points MT5 ↔ Pips
 * 
 * MT5 utilise nativement les "points" (unité minimale).
 * Mais les traders parlent en "pips" (unité de cotation).
 * 
 * Règles (basées sur norme MT5 officielle):
 * - Forex 5 décimales (EURUSD, GBPUSD, USDCAD): pip_value = 0.0001 → 1 pip = 10 points
 * - JPY 3 décimales (USDJPY, EURJPY, CADJPY): pip_value = 0.01 → 1 pip = 10 points
 * - Or (XAUUSD, XAUJPY): pip_value = 0.1 → 1 pip = 10 points
 * - Argent (XAGUSD): pip_value = 0.001 → 1 pip = 1000 points
 * - Indices (USA500IDXUSD): pip_value = 1.0 → 1 pip = 1 point
 * - Crypto (BTCUSD, ETHUSD): pip_value = 1.0 → 1 pip = 1 point
 */

type SymbolType = 'forex-5dec' | 'jpy' | 'gold' | 'silver' | 'indices' | 'crypto'

/**
 * Déterminer le type de symbole pour obtenir le bon pip_value
 */
function getSymbolType(symbol: string): SymbolType {
  if (symbol.includes('JPY')) return 'jpy'
  if (symbol.includes('XAU')) return 'gold'
  if (symbol.includes('XAG')) return 'silver'
  if (symbol.includes('US30') || symbol.includes('DE30') || symbol.includes('NAS100') || symbol.includes('SPX500') || symbol.includes('USA500')) return 'indices'
  if (symbol.includes('BTC') || symbol.includes('ETH')) return 'crypto'
  return 'forex-5dec'
}

/**
 * Obtenir la pip_value pour un symbole
 * pip_value = unité minimale de variation (en prix du marché)
 */
export function getPipValue(symbol: string): number {
  const type = getSymbolType(symbol)
  
  switch (type) {
    case 'jpy':      return 0.01      // 1 pip = 10 points
    case 'gold':     return 0.1       // 1 pip = 10 points
    case 'silver':   return 0.001     // 1 pip = 1000 points
    case 'indices':  return 1.0       // 1 pip = 1 point
    case 'crypto':   return 1.0       // 1 pip = 1 point
    default:         return 0.0001    // Forex 5 décimales: 1 pip = 10 points
  }
}

/**
 * Convertir les points MT5 en pips
 * Formula: pips = points / pip_value
 * 
 * @param symbol Symbole de trading (ex: "EURUSD", "BTCUSD")
 * @param points Valeur en points MT5
 * @returns Valeur en pips
 */
export function pointsToPips(symbol: string, points: number): number {
  const pipValue = getPipValue(symbol)
  return points / pipValue
}

/**
 * Convertir les pips en points MT5
 * Formula: points = pips × pip_value
 * 
 * @param symbol Symbole de trading
 * @param pips Valeur en pips
 * @returns Valeur en points MT5
 */
export function pipsToPoints(symbol: string, pips: number): number {
  const pipValue = getPipValue(symbol)
  return pips * pipValue
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
  const pipsFormatted = pips.toFixed(decimals)
  const pointsFormatted = points.toFixed(1)
  
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
