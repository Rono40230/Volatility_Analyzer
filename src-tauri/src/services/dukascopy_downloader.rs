// services/dukascopy_downloader.rs
// Télécharge et décode les fichiers tick .bi5 depuis le CDN Dukascopy.
//
// CDN public : https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM-1}/{DD}/{HH}h_ticks.bi5
// Format bi5 : LZMA compressé, chaque tick = 20 octets big-endian.
// ⚠️ Mois 0-indexé (janvier = 00, décembre = 11).

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};

/// Tick décodé depuis un fichier .bi5 Dukascopy
#[derive(Debug, Clone)]
pub struct DukascopyTick {
    pub datetime_utc: DateTime<Utc>,
    pub ask: f64,
    pub bid: f64,
    pub ask_volume: f32,
    pub bid_volume: f32,
}

/// Progression du téléchargement (émis via événements Tauri)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub hours_total: u32,
    pub hours_done: u32,
    pub hours_with_data: u32,
    pub symbol: String,
    pub current_date: String,
    pub percent: f64,
}

const BASE_URL: &str = "https://datafeed.dukascopy.com/datafeed";
const TICK_SIZE: usize = 20;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 500;
const CONCURRENT_DOWNLOADS: usize = 8;

/// Tâche de téléchargement d'une heure (entrée du batch parallèle)
struct HourTask {
    url: String,
    hour_start_utc: DateTime<Utc>,
    date_label: String,
}

/// Résultat d'une tâche horaire
struct HourResult {
    ticks: Vec<DukascopyTick>,
    had_data: bool,
}

/// Construit l'URL CDN pour une heure spécifique. Le mois est 1-indexé en entrée.
pub fn build_url(symbol: &str, year: i32, month: u32, day: u32, hour: u32) -> String {
    format!(
        "{}/{}/{:04}/{:02}/{:02}/{:02}h_ticks.bi5",
        BASE_URL,
        symbol.to_uppercase(),
        year,
        month.saturating_sub(1), // CDN : mois 0-indexé
        day,
        hour
    )
}

/// Décompresse un fichier .bi5 (LZMA).
pub fn decompress_bi5(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    let mut reader = std::io::BufReader::new(data);
    let mut output = Vec::new();
    lzma_rs::lzma_decompress(&mut reader, &mut output)
        .map_err(|e| format!("Erreur décompression LZMA : {}", e))?;
    Ok(output)
}

/// Parse des ticks binaires décompressés.
///
/// Chaque tick = 20 octets BE :
///   [0..4]  u32 : millisecondes depuis le début de l'heure
///   [4..8]  u32 : ask × point_value
///   [8..12] u32 : bid × point_value
///   [12..16] f32 : ask_volume
///   [16..20] f32 : bid_volume
pub fn parse_bi5_ticks(
    raw: &[u8],
    hour_start_utc: DateTime<Utc>,
    point_value: f64,
) -> Result<Vec<DukascopyTick>, String> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    if raw.len() % TICK_SIZE != 0 {
        return Err(format!(
            "Taille invalide : {} octets (pas un multiple de {})",
            raw.len(),
            TICK_SIZE
        ));
    }

    let tick_count = raw.len() / TICK_SIZE;
    let mut ticks = Vec::with_capacity(tick_count);

    for i in 0..tick_count {
        let off = i * TICK_SIZE;
        let ms = read_u32_be(raw, off);
        let ask_raw = read_u32_be(raw, off + 4);
        let bid_raw = read_u32_be(raw, off + 8);
        let ask_vol = read_f32_be(raw, off + 12);
        let bid_vol = read_f32_be(raw, off + 16);

        let ask = ask_raw as f64 / point_value;
        let bid = bid_raw as f64 / point_value;

        if ask <= 0.0 || bid <= 0.0 || ask < bid {
            continue;
        }

        let datetime_utc = hour_start_utc + chrono::Duration::milliseconds(ms as i64);
        ticks.push(DukascopyTick { datetime_utc, ask, bid, ask_volume: ask_vol, bid_volume: bid_vol });
    }

    Ok(ticks)
}

/// Télécharge les ticks pour une plage de dates (parallèle par batch de 8).
pub async fn download_range<F>(
    symbol: &str,
    from: NaiveDate,
    to: NaiveDate,
    point_value: f64,
    mut on_progress: F,
) -> Result<Vec<DukascopyTick>, String>
where
    F: FnMut(DownloadProgress),
{
    let client = Arc::new(
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Erreur client HTTP : {}", e))?,
    );

    // Construire toutes les tâches horaires
    let mut tasks: Vec<HourTask> = Vec::new();
    let mut current = from;
    while current <= to {
        for hour in 0..24u32 {
            let hour_start = current.and_hms_opt(hour, 0, 0).ok_or("Erreur datetime")?;
            let hour_start_utc = DateTime::<Utc>::from_naive_utc_and_offset(hour_start, Utc);
            let url = build_url(symbol, current.year(), current.month(), current.day(), hour);
            tasks.push(HourTask { url, hour_start_utc, date_label: current.to_string() });
        }
        current = current.succ_opt().ok_or("Erreur date suivante")?;
    }

    let total_hours = tasks.len() as u32;
    let mut all_ticks: Vec<DukascopyTick> = Vec::with_capacity(total_hours as usize * 500);
    let mut hours_done: u32 = 0;
    let mut hours_with_data: u32 = 0;

    // Traiter par batch de CONCURRENT_DOWNLOADS
    for batch in tasks.chunks(CONCURRENT_DOWNLOADS) {
        let mut handles = Vec::with_capacity(batch.len());
        for task in batch {
            let c = Arc::clone(&client);
            let url = task.url.clone();
            let hsu = task.hour_start_utc;
            let pv = point_value;
            handles.push(tokio::spawn(async move {
                download_and_parse_hour(&c, &url, hsu, pv).await
            }));
        }
        let last_date = batch.last().map(|t| t.date_label.clone()).unwrap_or_default();
        for handle in handles {
            let result = handle.await.map_err(|e| format!("Task join error: {}", e))?;
            hours_done += 1;
            if let Ok(hr) = result {
                if hr.had_data {
                    hours_with_data += 1;
                    all_ticks.extend(hr.ticks);
                }
            }
        }
        on_progress(DownloadProgress {
            hours_total: total_hours, hours_done, hours_with_data,
            symbol: symbol.to_string(), current_date: last_date,
            percent: (hours_done as f64 / total_hours as f64) * 100.0,
        });
    }

    // Trier par timestamp (les batch parallèles peuvent mélanger l'ordre)
    all_ticks.sort_by_key(|t| t.datetime_utc);
    info!("✅ Téléchargement terminé : {} ticks, {}/{} h avec données", all_ticks.len(), hours_with_data, total_hours);
    Ok(all_ticks)
}

/// Télécharge, décompresse et parse une heure.
async fn download_and_parse_hour(
    client: &reqwest::Client, url: &str, hour_start_utc: DateTime<Utc>, point_value: f64,
) -> Result<HourResult, String> {
    match download_hour_retry(client, url).await {
        Ok(data) if !data.is_empty() => {
            let raw = decompress_bi5(&data)?;
            let ticks = parse_bi5_ticks(&raw, hour_start_utc, point_value)?;
            Ok(HourResult { had_data: !ticks.is_empty(), ticks })
        }
        Ok(_) => Ok(HourResult { had_data: false, ticks: Vec::new() }),
        Err(e) => { warn!("⚠ {} : {}", url, e); Ok(HourResult { had_data: false, ticks: Vec::new() }) }
    }
}

#[cfg(test)]
fn count_hours(from: NaiveDate, to: NaiveDate) -> u32 {
    let days = (to - from).num_days().max(0) as u32 + 1;
    days * 24
}

async fn download_hour_retry(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    for attempt in 0..MAX_RETRIES {
        match client.get(url).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.bytes().await.map(|b| b.to_vec())
                        .map_err(|e| format!("Erreur lecture réponse : {}", e));
                } else if status.as_u16() == 404 {
                    return Ok(Vec::new());
                }
                warn!("HTTP {} pour {} (tentative {}/{})", status, url, attempt + 1, MAX_RETRIES);
            }
            Err(e) => warn!("Réseau {} ({}/{}): {}", url, attempt + 1, MAX_RETRIES, e),
        }
        if attempt + 1 < MAX_RETRIES {
            tokio::time::sleep(std::time::Duration::from_millis(RETRY_BASE_MS * 2u64.pow(attempt))).await;
        }
    }
    Err(format!("Échec après {} tentatives : {}", MAX_RETRIES, url))
}

fn read_u32_be(data: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn read_f32_be(data: &[u8], offset: usize) -> f32 {
    f32::from_be_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_build_url_january() {
        let url = build_url("EURUSD", 2024, 1, 15, 14);
        assert_eq!(url, "https://datafeed.dukascopy.com/datafeed/EURUSD/2024/00/15/14h_ticks.bi5");
    }

    #[test]
    fn test_build_url_december() {
        let url = build_url("USDJPY", 2024, 12, 31, 23);
        assert_eq!(url, "https://datafeed.dukascopy.com/datafeed/USDJPY/2024/11/31/23h_ticks.bi5");
    }

    #[test]
    fn test_decompress_empty() {
        assert!(decompress_bi5(&[]).unwrap().is_empty());
    }

    #[test]
    fn test_parse_bi5_empty() {
        let hour = Utc::now();
        assert!(parse_bi5_ticks(&[], hour, 100_000.0).unwrap().is_empty());
    }

    #[test]
    fn test_parse_bi5_invalid_size() {
        assert!(parse_bi5_ticks(&[0u8; 15], Utc::now(), 100_000.0).is_err());
    }

    #[test]
    fn test_parse_bi5_single_tick() {
        let hour_start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let mut data = Vec::with_capacity(20);
        data.extend_from_slice(&5000u32.to_be_bytes());      // 5s dans l'heure
        data.extend_from_slice(&114172u32.to_be_bytes());     // ask
        data.extend_from_slice(&114126u32.to_be_bytes());     // bid
        data.extend_from_slice(&0.9f32.to_be_bytes());        // ask_vol
        data.extend_from_slice(&0.9f32.to_be_bytes());        // bid_vol

        let ticks = parse_bi5_ticks(&data, hour_start, 100_000.0).unwrap();
        assert_eq!(ticks.len(), 1);
        assert!((ticks[0].ask - 1.14172).abs() < 1e-10);
        assert!((ticks[0].bid - 1.14126).abs() < 1e-10);
        assert_eq!(ticks[0].datetime_utc, hour_start + chrono::Duration::milliseconds(5000));
    }

    #[test]
    fn test_parse_bi5_multiple_ticks() {
        let hour_start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let mut data = Vec::with_capacity(40);
        // Tick 1
        data.extend_from_slice(&1000u32.to_be_bytes());
        data.extend_from_slice(&114172u32.to_be_bytes());
        data.extend_from_slice(&114126u32.to_be_bytes());
        data.extend_from_slice(&0.5f32.to_be_bytes());
        data.extend_from_slice(&0.5f32.to_be_bytes());
        // Tick 2
        data.extend_from_slice(&2000u32.to_be_bytes());
        data.extend_from_slice(&114200u32.to_be_bytes());
        data.extend_from_slice(&114150u32.to_be_bytes());
        data.extend_from_slice(&1.0f32.to_be_bytes());
        data.extend_from_slice(&1.0f32.to_be_bytes());

        let ticks = parse_bi5_ticks(&data, hour_start, 100_000.0).unwrap();
        assert_eq!(ticks.len(), 2);
        assert!(ticks[0].datetime_utc < ticks[1].datetime_utc);
    }

    #[test]
    fn test_point_value_usdjpy() {
        let hour_start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
        let mut data = Vec::with_capacity(20);
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&149123u32.to_be_bytes());  // ask=149.123
        data.extend_from_slice(&149100u32.to_be_bytes());  // bid=149.100
        data.extend_from_slice(&1.0f32.to_be_bytes());
        data.extend_from_slice(&1.0f32.to_be_bytes());

        let ticks = parse_bi5_ticks(&data, hour_start, 1_000.0).unwrap();
        assert_eq!(ticks.len(), 1);
        assert!((ticks[0].ask - 149.123).abs() < 1e-6);
        assert!((ticks[0].bid - 149.100).abs() < 1e-6);
    }

    #[test]
    fn test_read_u32_be() {
        assert_eq!(read_u32_be(&[0x00, 0x00, 0x13, 0x88], 0), 5000);
    }

    #[test]
    fn test_read_f32_be() {
        let bytes = 0.9f32.to_be_bytes();
        assert!((read_f32_be(&bytes, 0) - 0.9).abs() < 1e-6);
    }

    #[test]
    fn test_count_hours() {
        use chrono::NaiveDate;
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(count_hours(from, to), 24); // 1 jour = 24h
        let to2 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        assert_eq!(count_hours(from, to2), 48); // 2 jours = 48h
    }
}
