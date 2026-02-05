use crate::models::StraddleParameters;
use crate::models::trading_costs::TradingCostProfile;

pub struct StraddleParameterService;

impl StraddleParameterService {
    /// Calcule les paramètres Straddle unifiés (utilisé par Volatilité Brute et Corrélation)
    ///
    /// Logique harmonisée (Straddle simultané V4 - High Volatility Hardened):
    /// - Offset : basé sur le P95 des mèches récentes + coûts (fallback ATR si indisponible)
    /// - SL : ÉLARGI (2.5x à 5.0x ATR) pour encaisser le bruit initial
    /// - TP : Non défini explicitement (Trailing Stop utilisé)
    /// - Timeout : Basé sur la demi‑vie si disponible, sinon dérivé de la volatilité (ATR/Noise)
    ///
    /// NOTE: `atr` doit être déjà normalisé (en Pips/Points).
    pub fn calculate_parameters(
        atr: f64,
        noise_ratio: f64,
        _pip_value: f64, // Gardé pour compatibilité signature mais non utilisé si ATR normalisé
        symbol: &str,
        half_life_minutes: Option<u16>,
        p95_wick_pips: Option<f64>,
    ) -> StraddleParameters {
        // Récupération du profil de coûts
        let costs = TradingCostProfile::get_profile(symbol);
        let spread = costs.spread_avg;
        let slippage = costs.slippage;

        // 1. Offset Adaptatif
        // Priorité au P95 des mèches (Straddle simultané) + coûts
        let offset_pips = match p95_wick_pips {
            Some(p95) if p95 > 0.0 => (p95 * 1.1).ceil() + spread + slippage,
            _ => {
                // Formule fallback : Offset = (ATR * Multiplier) + Spread + Slippage
                // Min: 1.5x ATR (si Noise=0)
                // Max: Capé à 4.0x ATR pour éviter des offsets impossibles à atteindre
                let offset_multiplier = (1.5 + (noise_ratio * 0.5)).min(4.0);
                (atr * offset_multiplier).ceil() + spread + slippage
            }
        };

        // 2. Stop Loss Adaptatif (Formule Linéaire V5)
        // Formule: SL = (ATR * Multiplier) + Slippage
        // Min: 2.0x ATR
        // Max: Capé à 6.0x ATR
        let sl_ratio = (2.0 + (noise_ratio * 0.8)).min(6.0);
        let stop_loss_pips = (atr * sl_ratio).ceil() + slippage;

        // 3. Trailing Stop (Suivi)
        // Formule: TS = ATR * (0.8 + (NoiseRatio * 0.3))
        // Min: 0.8x ATR
        // Max: Capé à 3.0x ATR
        let ts_ratio = (0.8 + (noise_ratio * 0.3)).min(3.0);
        let trailing_stop_pips = (atr * ts_ratio).ceil();

        // 4. SL Recovery (Simultané)
        // Ratio demandé : 1.2x le Stop Loss
        let sl_recovery_pips = (stop_loss_pips * 1.2).ceil();

        // 5. Hard TP (Take Profit Fixe)
        // Sécurité pour encaisser les spikes violents.
        // Target: 2.0x le risque (Stop Loss)
        let hard_tp_pips = (stop_loss_pips * 2.0).ceil();

        // 6. Timeout Dynamique
        // Priorité à la demi‑vie si disponible, sinon dérivation ATR/Noise
        let timeout_minutes = match half_life_minutes {
            Some(hl) => hl.clamp(1, 15) as i32,
            None => Self::deriver_timeout(atr, noise_ratio),
        };

        // 7. Risk/Reward (Théorique, basé sur volatilité attendue vs SL)
        // Ici on met juste un indicateur
        let risk_reward_ratio = if stop_loss_pips > 0.0 {
            // On vise au moins 1x la volatilité (ATR)
            let target = atr;
            target / stop_loss_pips
        } else {
            0.0
        };

        StraddleParameters {
            offset_pips,
            stop_loss_pips,
            trailing_stop_pips,
            timeout_minutes,
            sl_recovery_pips,
            hard_tp_pips,
            risk_reward_ratio,
            spread_safety_margin_pips: spread,
        }
    }

    /// Dérive un timeout à partir de l’ATR (volatilité) et du Noise Ratio
    fn deriver_timeout(atr: f64, noise_ratio: f64) -> i32 {
        if atr <= 0.0 {
            return 5;
        }

        // Volatilité élevée => timeout plus court
        let atr_factor = (10.0 / (atr * 10.0)).clamp(2.0, 12.0);
        let noise_factor = (noise_ratio * 2.0).clamp(0.0, 6.0);
        let timeout = (atr_factor + noise_factor).round() as i32;

        timeout.clamp(2, 12)
    }
}
