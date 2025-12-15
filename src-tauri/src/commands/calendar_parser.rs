pub fn parse_record(record: &csv::StringRecord) -> Option<(String, String, String, String)> {
    if record[0].contains('-') {
        if record.len() < 5 {
            return None;
        }
        let date = record[0].trim();
        let time = record[1].trim();
        let currency = record[2].trim();
        let event = record[3].trim();
        let impact = record[4].trim();

        let date_parts: Vec<&str> = date.split('-').collect();
        let time_parts: Vec<&str> = time.split(':').collect();

        if date_parts.len() != 3 || time_parts.len() < 2 {
            return None;
        }

        let dt = format!(
            "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:00",
            date_parts[0].trim(),
            date_parts[1].trim(),
            date_parts[2].trim(),
            time_parts[0].trim(),
            time_parts[1].trim()
        );

        Some((
            dt,
            currency.to_string(),
            impact.to_string(),
            event.to_string(),
        ))
    } else {
        if record.len() < 8 {
            return None;
        }

        let year = record[0].trim();
        let month = record[1].trim();
        let day = record[2].trim();
        let hour = record[3].trim();
        let minute = record[4].trim();
        let symbol = record[5].trim();
        let impact = record[6].trim();
        let description = record[7].trim();

        let dt = format!(
            "{}-{:0>2}-{:0>2} {:0>2}:{:0>2}:00",
            year, month, day, hour, minute
        );
        Some((
            dt,
            symbol.to_string(),
            impact.to_string(),
            description.to_string(),
        ))
    }
}
