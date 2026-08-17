use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDate, Utc};

/// Parse a user-supplied date into a UTC timestamp.
///
/// Accepts either `YYYY-MM-DD` (interpreted as UTC midnight) or a full
/// RFC 3339 timestamp such as `2024-01-15T00:00:00Z`.
pub fn parse_date(s: &str) -> Result<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let midnight = date
            .and_hms_opt(0, 0, 0)
            .expect("midnight is always a valid time");
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(midnight, Utc));
    }
    bail!("could not parse date '{s}': expected YYYY-MM-DD or an RFC 3339 timestamp")
}

#[cfg(test)]
mod tests {
    use super::parse_date;
    use chrono::{Datelike, Timelike, Utc};

    #[test]
    fn parses_iso_date_as_utc_midnight() {
        let dt = parse_date("2024-01-15").unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
        assert_eq!(dt.minute(), 0);
        assert_eq!(dt.offset(), &Utc);
    }

    #[test]
    fn parses_rfc3339() {
        let dt = parse_date("2024-01-15T12:30:00Z").unwrap();
        assert_eq!(dt.hour(), 12);
        assert_eq!(dt.minute(), 30);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_date("not-a-date").is_err());
    }
}
