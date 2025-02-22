use chrono::NaiveDate;

pub enum Format {
    DATE
}

impl Format {
    fn get_value(&self) -> &str {
        match self {
            Format::DATE => "%Y-%m-%d",
        }
    }
}

/**
 * Parse a date string into a NaiveDate object
 */
pub fn parse_date(date: &str, format: Format) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, format.get_value()).ok()
}

/**
 * Format a NaiveDate object into a string
 */
pub fn format_date(date: &NaiveDate, format: Format) -> String {
    date.format(format.get_value()).to_string()
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_date_valid() {
        let date_str = "2025-02-22";
        let format = Format::DATE;

        // Test the parse_date function with a valid date
        let parsed_date = parse_date(date_str, format);
        assert!(parsed_date.is_some(), "The date should be parsed correctly.");

        // Check that the parsed date matches the expected date
        let expected_date = NaiveDate::from_ymd_opt(2025, 2, 22);
        assert_eq!(parsed_date.unwrap(), expected_date.unwrap(), "The parsed date is incorrect.");
    }

    #[test]
    fn test_parse_date_invalid() {
        let date_str = "2025-13-01"; // Invalid month
        let format = Format::DATE;

        // Test the parse_date function with an invalid date
        let parsed_date = parse_date(date_str, format);
        assert!(parsed_date.is_none(), "An invalid date should return None.");
    }

    #[test]
    fn test_format_date_valid() {
        let date = NaiveDate::from_ymd_opt(2025, 2, 22);
        let format = Format::DATE;

        // Test the format_date function with a valid date
        let formatted_date = format_date(&date.unwrap(), format);
        assert_eq!(formatted_date, "2025-02-22", "The formatted date is incorrect.");
    }

    #[test]
    fn test_format_date_empty() {
        // Test the format_date function with an empty date
        let date = NaiveDate::from_ymd_opt(0, 1, 1); // A "blank" (theoretical) date
        let format = Format::DATE;

        // Check the formatted result for this date
        let formatted_date = format_date(&date.unwrap(), format);
        assert_eq!(formatted_date, "0000-01-01", "The formatted empty date is incorrect.");
    }
}
