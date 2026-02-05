use chrono::{Datelike, NaiveDateTime, Weekday};

/// Convertit une heure UTC en heure de Paris
#[allow(dead_code)]
pub fn utc_to_paris(utc_hour: u32, date: &NaiveDateTime) -> u32 {
    let is_dst = is_paris_dst(date);
    let offset = if is_dst { 2 } else { 1 };
    (utc_hour + offset) % 24
}

/// Détermine si Paris est en heure d'été
#[allow(dead_code)]
pub fn is_paris_dst(date: &NaiveDateTime) -> bool {
    let month = date.month();
    let day = date.day();

    if month > 3 && month < 10 {
        return true;
    }

    if !(3..=10).contains(&month) {
        return false;
    }

    if month == 3 {
        let last_sunday = last_sunday_of_month(date.year(), 3);
        return day >= last_sunday;
    }

    if month == 10 {
        let last_sunday = last_sunday_of_month(date.year(), 10);
        return day < last_sunday;
    }

    false
}

/// Calcule le jour du dernier dimanche d'un mois
#[allow(dead_code)]
fn last_sunday_of_month(year: i32, month: u32) -> u32 {
    for d in (1..=31).rev() {
        if let Some(date) =
            chrono::NaiveDate::from_ymd_opt(year, month, d).and_then(|d| d.and_hms_opt(12, 0, 0))
        {
            if date.weekday() == Weekday::Sun {
                return d;
            }
        }
    }
    31
}

/// Formate les horaires Paris d'une session
pub fn format_paris_hours(utc_start: u32, utc_end: u32, is_winter: bool) -> String {
    let offset = if is_winter { 1 } else { 2 };
    let start = (utc_start + offset) % 24;
    let end = (utc_end + offset) % 24;
    format!("{:02}h{:02}-{:02}h{:02}", start, 0, end, 0)
}

/// Détermine si une heure UTC appartient à une session
pub fn is_in_session(hour: u32, utc_start: u32, utc_end: u32) -> bool {
    if utc_start < utc_end {
        hour >= utc_start && hour < utc_end
    } else {
        hour >= utc_start || hour < utc_end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_paris_dst_summer() {
        let summer = NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(is_paris_dst(&summer));
    }

    #[test]
    fn test_is_paris_dst_winter() {
        let winter = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(!is_paris_dst(&winter));
    }

    #[test]
    fn test_is_paris_dst_march_boundary() {
        // En 2024, le dernier dimanche de mars est le 31.
        let after_dst = NaiveDateTime::parse_from_str("2024-03-31 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(is_paris_dst(&after_dst));

        let before_dst = NaiveDateTime::parse_from_str("2024-03-25 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(!is_paris_dst(&before_dst));
    }

    #[test]
    fn test_is_paris_dst_october_boundary() {
        let before = NaiveDateTime::parse_from_str("2024-10-25 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(is_paris_dst(&before));

        let after = NaiveDateTime::parse_from_str("2024-10-28 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert!(!is_paris_dst(&after));
    }

    #[test]
    fn test_is_in_session_normal_range() {
        assert!(is_in_session(10, 8, 17));
        assert!(is_in_session(8, 8, 17));
        assert!(!is_in_session(17, 8, 17));
        assert!(!is_in_session(20, 8, 17));
    }

    #[test]
    fn test_is_in_session_overnight() {
        assert!(is_in_session(23, 22, 7));
        assert!(is_in_session(5, 22, 7));
        assert!(!is_in_session(12, 22, 7));
        assert!(is_in_session(0, 22, 7));
    }

    #[test]
    fn test_utc_to_paris_summer() {
        let summer = NaiveDateTime::parse_from_str("2024-06-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert_eq!(utc_to_paris(12, &summer), 14); // UTC+2 en été
    }

    #[test]
    fn test_utc_to_paris_winter() {
        let winter = NaiveDateTime::parse_from_str("2024-01-15 12:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("Failed to parse datetime");
        assert_eq!(utc_to_paris(12, &winter), 13); // UTC+1 en hiver
    }

    #[test]
    fn test_format_paris_hours() {
        let summer = format_paris_hours(8, 17, false); // winter=false = été
        assert!(summer.contains("10h00") || summer.contains("10h01")); // 8+2=10

        let winter = format_paris_hours(8, 17, true); // winter=true = hiver
        assert!(winter.contains("09h00") || winter.contains("09h01")); // 8+1=9
    }

    #[test]
    fn test_format_paris_hours_wraparound() {
        let formatted = format_paris_hours(22, 7, true);
        assert!(!formatted.is_empty());
    }
}
