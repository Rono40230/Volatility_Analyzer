use crate::models::StraddleParameters;
use crate::models::trading_costs::TradingCostProfile;
use crate::services::straddle_multipliers::apply_time_adjustment;

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
        hour_utc: Option<u32>,
    ) -> StraddleParameters {
        // Récupération du profil de coûts
        let costs = TradingCostProfile::get_profile(symbol);
        let spread = costs.spread_avg;
        let slippage = costs.slippage;

        // 1. Offset Adaptatif
        // Priorité au P95 des mèches (Straddle simultané) + coûts
        let offset_pips = match p95_wick_pips {
            // Marge de sécurité +10% sur P95 des mèches : compense la variance
            // résiduelle au-delà du 95ème percentile (évite les faux déclenchements)
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
        let raw_stop_loss = (atr * sl_ratio).ceil() + slippage;
        // Ajustement temporel : élargit le SL en heures critiques, réduit en heures calmes
        let stop_loss_pips = match hour_utc {
            Some(h) => apply_time_adjustment(raw_stop_loss, h),
            None => raw_stop_loss,
        };

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

        // 6. Timeout Dynamique (harmonisé avec straddle_adjustments)
        // Cascade de priorité :
        //   1. Demi-vie si disponible → clamp [10, 30]
        //   2. Sinon heuristique ATR/Noise → clamp [10, 30]
        // Minimum 10 min : en-dessous, un straddle event-driven n'a pas
        // le temps de se déployer.
        let timeout_minutes = match half_life_minutes {
            Some(hl) => (hl as i32 * 2).clamp(10, 30),
            None => Self::deriver_timeout(atr, noise_ratio),
        };

        // 7. Risk/Reward (basé sur Hard TP vs SL)
        // R/R = potentiel de gain / risque = hard_tp / stop_loss
        // Par construction hard_tp = 2×SL, donc R/R ≈ 2.0
        let risk_reward_ratio = if stop_loss_pips > 0.0 {
            hard_tp_pips / stop_loss_pips
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
    /// Range harmonisé [10, 30] min (cohérent avec straddle_adjustments)
    fn deriver_timeout(atr: f64, noise_ratio: f64) -> i32 {
        if atr <= 0.0 {
            return 15; // Fallback raisonnable
        }

        // Volatilité élevée => timeout plus court (la volatilité décline vite)
        // Bruit élevé => timeout plus long (besoin de temps pour que le signal se dégage)
        let atr_factor = (30.0 / (atr * 3.0)).clamp(10.0, 25.0);
        let noise_factor = (noise_ratio * 2.0).clamp(0.0, 5.0);
        let timeout = (atr_factor + noise_factor).round() as i32;

        timeout.clamp(10, 30)
    }
}
