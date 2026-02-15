use super::*;

#[test]
fn test_detection_eurusd() {
    let props = AssetProperties::from_symbol("EURUSD");
    assert_eq!(props.asset_type, AssetType::ForexMajor);
    assert_eq!(props.pip_value, 0.0001);
    assert_eq!(props.normalize(0.0005), 5.0); // 5 pips
}

#[test]
fn test_detection_usdjpy() {
    let props = AssetProperties::from_symbol("USDJPY");
    assert_eq!(props.asset_type, AssetType::ForexJpy);
    assert_eq!(props.pip_value, 0.01);
    assert_eq!(props.normalize(0.05), 5.0); // 5 pips
}

#[test]
fn test_detection_gold() {
    let props = AssetProperties::from_symbol("XAUUSD");
    assert_eq!(props.asset_type, AssetType::Gold);
    assert_eq!(props.pip_value, 0.1);
    assert_eq!(props.normalize(1.5), 15.0); // 15 pips (1.5$)
}

#[test]
fn test_detection_btc() {
    let props = AssetProperties::from_symbol("BTCUSD");
    assert_eq!(props.asset_type, AssetType::Crypto);
    assert_eq!(props.pip_value, 1.0);
    assert_eq!(props.normalize(500.0), 500.0); // 500$
}

#[test]
fn test_detection_indices() {
    let symbols = vec!["USATEC", "USAIDX", "DEUIDX", "NAS100", "US30", "GER40", "SPX500"];
    for symbol in symbols {
        let props = AssetProperties::from_symbol(symbol);
        assert_eq!(props.asset_type, AssetType::Index, "Failed for {}", symbol);
        assert_eq!(props.pip_value, 1.0, "Failed for {}", symbol);
    }
}

#[test]
fn test_detection_extended() {
    // Crypto Extended
    let cryptos = vec!["LTCUSD", "DOGEUSD", "SHIBUSD", "MATICUSD", "PEPEUSD"];
    for s in cryptos {
        let props = AssetProperties::from_symbol(s);
        assert_eq!(props.asset_type, AssetType::Crypto, "Failed for {}", s);
    }

    // Commodities
    let commodities = vec!["WTI", "BRENT", "UKOIL", "USOIL", "XPTUSD"];
    for s in commodities {
        let props = AssetProperties::from_symbol(s);
        assert_eq!(props.asset_type, AssetType::Commodity, "Failed for {}", s);
        assert_eq!(props.pip_value, 0.01, "Failed for {}", s);
    }

    // NGAS
    let ngas = AssetProperties::from_symbol("NGAS");
    assert_eq!(ngas.asset_type, AssetType::Commodity);
    assert_eq!(ngas.pip_value, 0.001);

    // Forex Exotique (HUF/CZK)
    let exotic = AssetProperties::from_symbol("USDHUF");
    assert_eq!(exotic.asset_type, AssetType::ForexJpy); // Mapped to JPY logic
    assert_eq!(exotic.pip_value, 0.01);
}

#[test]
fn test_normalize_zero_et_protection_division() {
    let props = AssetProperties::from_symbol("EURUSD");
    // Valeur brute zéro → retourne 0
    assert_eq!(props.normalize(0.0), 0.0);

    // pip_value invalide → retourne 0 (protection division par zéro)
    let props_invalide = AssetProperties {
        asset_type: AssetType::ForexMajor,
        pip_value: 0.0,
        unit: "pips".to_string(),
        display_digits: 4,
    };
    assert_eq!(props_invalide.normalize(1.5), 0.0);

    // pip_value négatif → retourne 0
    let props_negatif = AssetProperties {
        asset_type: AssetType::ForexMajor,
        pip_value: -0.001,
        unit: "pips".to_string(),
        display_digits: 4,
    };
    assert_eq!(props_negatif.normalize(1.5), 0.0);
}

#[test]
fn test_override_invalide_ignore() {
    use crate::models::symbol_conversion::SymbolConversion;

    // Override avec pip_value = 0 → ignoré, fallback hardcodé
    let override_zero = SymbolConversion {
        symbol: "EURUSD".to_string(),
        pip_value: 0.0,
        unit: "pips".to_string(),
        display_digits: 4,
    };
    let props = AssetProperties::from_symbol_with_override("EURUSD", Some(override_zero));
    assert_eq!(props.pip_value, 0.0001); // Fallback hardcodé

    // Override avec pip_value négatif → ignoré
    let override_neg = SymbolConversion {
        symbol: "EURUSD".to_string(),
        pip_value: -1.0,
        unit: "pips".to_string(),
        display_digits: 4,
    };
    let props = AssetProperties::from_symbol_with_override("EURUSD", Some(override_neg));
    assert_eq!(props.pip_value, 0.0001); // Fallback hardcodé

    // Override valide → utilisé normalement
    let override_ok = SymbolConversion {
        symbol: "EURUSD".to_string(),
        pip_value: 0.001,
        unit: "points".to_string(),
        display_digits: 3,
    };
    let props = AssetProperties::from_symbol_with_override("EURUSD", Some(override_ok));
    assert_eq!(props.pip_value, 0.001);
    assert_eq!(props.unit, "points");
}
