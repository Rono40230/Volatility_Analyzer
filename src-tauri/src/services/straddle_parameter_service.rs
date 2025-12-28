use crate::models::StraddleParameters;
use crate::models::trading_costs::TradingCostProfile;

pub struct StraddleParameterService;

impl StraddleParameterService {
    /// Calcule les paramètres Straddle unifiés (utilisé par Volatilité Brute et Corrélation)
    ///
    /// Logique harmonisée (Bidi V4 - High Volatility Hardened):
    /// - Offset : DURCI (2.0x à 3.0x ATR) pour éviter les mèches des gros mouvements
    /// - SL : ÉLARGI (2.5x à 5.0x ATR) pour encaisser le bruit initial
    /// - TP : Non défini explicitement (Trailing Stop utilisé)
    /// - Timeout : Basé sur la volatilité (fixe à 3 min pour l'instant ou calculé)
    ///
    /// NOTE: `atr` doit être déjà normalisé (en Pips/Points).
    pub fn calculate_parameters(
        atr: f64,
        noise_ratio: f64,
        _pip_value: f64, // Gardé pour compatibilité signature mais non utilisé si ATR normalisé
        symbol: &str,
        half_life_minutes: Option<u16>,
    ) -> StraddleParameters {
        // Récupération du profil de coûts
        let costs = TradingCostProfile::get_profile(symbol);
        let spread = costs.spread_avg;
        let slippage = costs.slippage;

        // 1. Offset Adaptatif (Formule Linéaire V5)
        // Formule: Offset = (ATR * Multiplier) + Spread + Slippage
        // Min: 1.5x ATR (si Noise=0)
        // Max: Capé à 4.0x ATR pour éviter des offsets impossibles à atteindre
        let offset_multiplier = (1.5 + (noise_ratio * 0.5)).min(4.0);
        let offset_pips = (atr * offset_multiplier).ceil() + spread + slippage;

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
        // Ratio demandé : 1.2x le SL Directionnel
        let sl_recovery_pips = (stop_loss_pips * 1.2).ceil();

        // 5. Hard TP (Take Profit Fixe)
        // Sécurité pour encaisser les spikes violents.
        // Target: 2.0x le risque (Stop Loss)
        let hard_tp_pips = (stop_loss_pips * 2.0).ceil();

        // 6. Timeout Dynamique
        // Basé sur la demi-vie de la volatilité si disponible
        // Sinon par défaut 3 minutes pour le scalping/news
        let timeout_minutes = half_life_minutes
            .map(|hl| hl.clamp(1, 15) as i32) // Min 1 min, Max 15 min
            .unwrap_or(3);

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
}
