// utils/assetUnit.ts — Résolution de l'unité d'affichage par symbole
// Miroir de la logique Rust asset_class.rs

import { useConversionStore } from '../stores/conversionStore'

const DOLLAR_ASSETS = ['XAU', 'GOLD', 'XAG', 'SILVER', 'BTC', 'ETH', 'SOL', 'BNB', 'XRP', 'ADA', 'DOT', 'LTC', 'BCH', 'DOGE', 'SHIB', 'LINK', 'MATIC', 'AVAX', 'UNI', 'XLM', 'TRX', 'ATOM', 'NEAR', 'PEPE', 'OIL', 'WTI', 'BRENT', 'CRUDE', 'XPT', 'XPD', 'NGAS']
const POINT_ASSETS = ['IDX', 'US30', 'DAX', 'NAS', 'GER', 'SPX', 'US100', 'US500', 'FRA40', 'UK100', 'EUSTX', 'JPN225', 'USATEC', 'USAIDX', 'DEUIDX', 'USTEC', 'HK50', 'FR40', 'GR30', 'DE40', 'WS30', 'NDX', 'VIX', 'DXY', 'STOXX', 'CAC', 'FTSE', 'NI225', 'ASX', 'HSI']

export function getUnitForSymbol(symbol: string): string {
  try {
    const store = useConversionStore()
    return store.getUnitForSymbol(symbol)
  } catch (e) {
    // Fallback si Pinia n'est pas encore prêt ou hors contexte Vue
    const s = symbol.toUpperCase()
    if (DOLLAR_ASSETS.some(k => s.includes(k))) return '$'
    if (POINT_ASSETS.some(k => s.includes(k))) return 'pts'
    return 'pips'
  }
}

/**
 * Retourne le pip_value pour dénormaliser les valeurs pips → unité réelle.
 * Miroir exact de AssetProperties::from_symbol() en Rust.
 * pips × pip_value = mouvement de prix réel
 */
export function getPipValueForSymbol(symbol: string): number {
  try {
    const store = useConversionStore()
    const s = symbol.toUpperCase()
    const conv = store.conversions.find(c => s.includes(c.symbol.toUpperCase()) || c.symbol.toUpperCase().includes(s))
    if (conv) return conv.pip_value
  } catch (e) {
    // Ignore et utilise le fallback statique
  }

  const s = symbol.toUpperCase()
  if (s.includes('JPY') || s.includes('HUF') || s.includes('CZK')) return 0.01
  if (s.includes('XAU') || s.includes('GOLD')) return 0.1
  if (s.includes('XAG') || s.includes('SILVER')) return 0.01
  if (s.includes('NGAS')) return 0.001
  if (DOLLAR_ASSETS.some(k => s.includes(k))) return 1.0
  if (POINT_ASSETS.some(k => s.includes(k))) return 1.0
  return 0.0001 // Forex majeur par défaut
}

/**
 * Convertit une valeur en pips vers la valeur d'affichage dans l'unité du symbole.
 * Pour les forex → pips (inchangé). Pour $/pts → mouvement de prix réel.
 */
export function pipsToDisplayValue(pipsValue: number, symbol: string, customUnit?: string): number {
  const unit = customUnit || getUnitForSymbol(symbol)
  if (unit === 'pips') return pipsValue
  // Pour $ et pts : dénormaliser (pips × pip_value = prix réel)
  return pipsValue * getPipValueForSymbol(symbol)
}
