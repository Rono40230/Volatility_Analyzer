/// Seuils de scoring pour HourlyStats
///
/// ATR adapté Forex (30 pts)
pub const ATR_EXCELLENT: f64 = 0.00025;
pub const ATR_GOOD: f64 = 0.00015;
pub const ATR_FAIR: f64 = 0.00010;
pub const ATR_POOR: f64 = 0.00005;

/// Body Range réaliste (25 pts)
pub const BODY_RANGE_EXCELLENT: f64 = 45.0;
pub const BODY_RANGE_GOOD: f64 = 35.0;
pub const BODY_RANGE_FAIR: f64 = 25.0;
pub const BODY_RANGE_POOR: f64 = 15.0;

/// Volatilité (20 pts)
pub const VOL_EXCELLENT: f64 = 0.30;
pub const VOL_GOOD: f64 = 0.20;
pub const VOL_FAIR: f64 = 0.10;
pub const VOL_POOR: f64 = 0.05;

/// Noise Ratio (15 pts)
pub const NOISE_EXCELLENT: f64 = 2.0;
pub const NOISE_GOOD: f64 = 3.0;
pub const NOISE_FAIR: f64 = 4.0;

/// Breakout % (10 pts)
pub const BREAKOUT_EXCELLENT: f64 = 15.0;
pub const BREAKOUT_GOOD: f64 = 10.0;
pub const BREAKOUT_FAIR: f64 = 5.0;
