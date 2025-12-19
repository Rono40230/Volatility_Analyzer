use crate::models::StraddleParameters;

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
        spread_margin: Option<f64>,
        half_life_minutes: Option<u16>,
    ) -> StraddleParameters {
        // Marge de sécurité spread (défaut 3.0 pips)
        let spread_safety = spread_margin.unwrap_or(3.0);

        // 1. Offset Adaptatif (DURCI V4)
        // Si bruit > 2.5, on s'écarte BEAUCOUP (3.0x) pour éviter les mèches
        // Sinon on reste prudent (2.0x) - Fini le 1.5x trop risqué
        let offset_multiplier = if noise_ratio > 2.5 { 3.0 } else { 2.0 };
        let offset_pips = (atr * offset_multiplier).ceil() + spread_safety;

        // 2. Stop Loss Adaptatif (ÉLARGI V4)
        // Plus il y a de bruit, plus le SL doit être large pour survivre au Whipsaw
        let sl_ratio = if noise_ratio > 3.5 {
            5.0 // Bruit extrême = SL très large
        } else if noise_ratio > 2.5 {
            4.0 // Bruit fort
        } else if noise_ratio > 2.0 {
            3.0 // Bruit moyen
        } else {
            2.5 // Calme (mais on garde 2.5 min pour sécurité)
        };
        let stop_loss_pips = (atr * sl_ratio).ceil();

        // 3. Trailing Stop (Suivi)
        // Environ 30-40% du SL, ou adaptatif
        let ts_ratio = if noise_ratio > 3.0 {
            2.0
        } else if noise_ratio > 2.0 {
            1.5
        } else if noise_ratio > 1.5 {
            1.2
        } else {
            1.0
        };
        let trailing_stop_pips = (atr * ts_ratio).ceil();

        // 4. SL Recovery (Simultané)
        // Doit couvrir l'offset inverse + marge.
        // Max(SL standard, 3x Offset)
        let sl_recovery_pips = stop_loss_pips.max(offset_pips * 3.0).ceil();

        // 5. Timeout Dynamique
        // Basé sur la demi-vie de la volatilité si disponible
        // Sinon par défaut 3 minutes pour le scalping/news
        let timeout_minutes = half_life_minutes
            .map(|hl| hl.clamp(1, 15) as i32) // Min 1 min, Max 15 min
            .unwrap_or(3);

        // 6. Risk/Reward (Théorique, basé sur volatilité attendue vs SL)
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
            risk_reward_ratio,
            spread_safety_margin_pips: spread_safety,
        }
    }
}
