use super::helpers::get_pip_value;
use super::types::PairImpactDetail;
use crate::commands::correlation::data_availability::has_candles_for_event;
use crate::commands::correlation::volatility_helpers::calculate_volatilities_optimized;
use crate::services::candle_index::CandleIndex;
use chrono::{DateTime, Utc};

pub fn calculate_pair_impacts(
    pairs: &[String],
    event_datetime: &DateTime<Utc>,
    candle_index: &CandleIndex,
) -> Result<Vec<PairImpactDetail>, String> {
    let mut pair_impacts = Vec::new();

    for pair in pairs {
        // Vérifier si des candles existent pour cette paire à cet instant
        let has_data = has_candles_for_event(candle_index, pair, event_datetime.naive_utc());

        let metrics = calculate_volatilities_optimized(
            candle_index,
            pair,
            event_datetime.naive_utc(),
            30,
            7,
            get_pip_value(pair),
        )?;

        let event_volatility = metrics.event_volatility;
        let baseline_volatility = metrics.baseline_volatility;
        let multiplier = if baseline_volatility > 0.0 {
            event_volatility / baseline_volatility
        } else {
            0.0
        };

        let direction = if multiplier > 10.0 {
            "HAUSSIER".to_string()
        } else if multiplier > 5.0 {
            "BAISSIER".to_string()
        } else {
            "NEUTRE".to_string()
        };

        let points = event_volatility / 10.0;
        let pip_value = get_pip_value(pair);
        let price = points * pip_value;

        pair_impacts.push(PairImpactDetail {
            symbol: pair.clone(),
            event_volatility,
            baseline_volatility,
            event_volatility_formatted: format!("{:.1}", event_volatility),
            baseline_volatility_formatted: format!("{:.1}", baseline_volatility),
            points,
            points_formatted: format!("{:.1}", points),
            price,
            price_formatted: format!("{:.2}", price),
            multiplier,
            direction,
            has_data: Some(has_data),
        });
    }

    pair_impacts.sort_by(|a, b| {
        b.multiplier
            .partial_cmp(&a.multiplier)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(pair_impacts)
}

pub fn generate_observations(pair_impacts: &[PairImpactDetail]) -> Vec<String> {
    let mut observations = Vec::new();

    if let Some(top_pair) = pair_impacts.first() {
        observations.push(format!(
            "{} a enregistré le plus fort impact avec {:.0} pips, soit {:.1}× sa volatilité normale",
            top_pair.symbol, top_pair.event_volatility, top_pair.multiplier
        ));
    }

    if let Some(biggest_vol) = pair_impacts.iter().max_by(|a, b| {
        a.event_volatility
            .partial_cmp(&b.event_volatility)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) {
        observations.push(format!(
            "Variation maximale observée: {} avec {:.1} pips de volatilité événement",
            biggest_vol.symbol, biggest_vol.event_volatility
        ));
    }

    let high_impact_count = pair_impacts.iter().filter(|p| p.multiplier > 5.0).count();
    if high_impact_count > 0 {
        let avg_multiplier = pair_impacts
            .iter()
            .filter(|p| p.multiplier > 5.0)
            .map(|p| p.multiplier)
            .sum::<f64>()
            / high_impact_count.max(1) as f64;

        observations.push(format!(
            "⚠️ Attention: {} paires ont montré une volatilité EXCESSIVE (multiplicateur >5×). Ces multiplicateurs élevés (moy. {:.1}×) indiquent une réaction disproportionnée. À éviter en trading régulier, risque trop élevé pour le gain potentiel",
            high_impact_count, avg_multiplier
        ));
    }

    observations
}
