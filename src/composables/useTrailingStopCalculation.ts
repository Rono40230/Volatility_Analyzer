/**
 * Calcul centralisé du Trailing Stop pour Straddle
 * Formule: TS = ATR × 0.75 × (1 + whipsaw_frequency × 0.3)
 */

/**
 * Calcule le Trailing Stop
 * @param atr - Average True Range
 * @param whipsawFrequency - Fréquence whipsaw (0-1), optionnel
 * @returns Distance du Trailing Stop en pips
 */
export function calculateTrailingStop(atr: number, whipsawFrequency: number = 0): number {
  const baseTS = atr * 0.75
  const adjustedTS = baseTS * (1 + whipsawFrequency * 0.3)
  return Math.round(adjustedTS * 10) / 10
}
