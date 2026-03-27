use crate::error::YukiError;

/// Parse a period string into (start_date, end_date) as "YYYY-MM-DD" strings.
///
/// Supported formats:
/// - `"YYYY"` — full calendar year
/// - `"YYYY-QN"` — calendar quarter (Q1–Q4)
/// - `"YYYY-MM"` — calendar month
pub fn parse_period(period: &str) -> Result<(String, String), YukiError> {
    let invalid = || YukiError::Config(format!("invalid period: {period}"));

    // YYYY
    if period.len() == 4 && period.chars().all(|c| c.is_ascii_digit()) {
        let year: u32 = period.parse().map_err(|_| invalid())?;
        return Ok((format!("{year:04}-01-01"), format!("{year:04}-12-31")));
    }

    // YYYY-QN
    if period.len() == 7 {
        let (year_str, rest) = period.split_at(4);
        if let Some(q) = rest.strip_prefix("-Q") {
            let year: u32 = year_str.parse().map_err(|_| invalid())?;
            let quarter: u32 = q.parse().map_err(|_| invalid())?;
            let (start_month, end_month, end_day) = match quarter {
                1 => (1u32, 3u32, 31u32),
                2 => (4, 6, 30),
                3 => (7, 9, 30),
                4 => (10, 12, 31),
                _ => return Err(invalid()),
            };
            return Ok((
                format!("{year:04}-{start_month:02}-01"),
                format!("{year:04}-{end_month:02}-{end_day:02}"),
            ));
        }
    }

    // YYYY-MM
    if period.len() == 7 {
        let (year_str, rest) = period.split_at(4);
        if let Some(month_str) = rest.strip_prefix('-') {
            let year: u32 = year_str.parse().map_err(|_| invalid())?;
            let month: u32 = month_str.parse().map_err(|_| invalid())?;
            if month == 0 || month > 12 {
                return Err(invalid());
            }
            let last_day = days_in_month(year, month);
            return Ok((
                format!("{year:04}-{month:02}-01"),
                format!("{year:04}-{month:02}-{last_day:02}"),
            ));
        }
    }

    Err(invalid())
}

/// Return the number of days in the given month of the given year.
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => unreachable!("month already validated"),
    }
}

/// Determine whether a year is a leap year.
fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
