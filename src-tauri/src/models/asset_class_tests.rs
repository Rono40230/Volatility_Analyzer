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
