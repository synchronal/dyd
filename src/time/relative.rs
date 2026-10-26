use crate::app::AppResult;

use chrono::Months;
use chrono::prelude::*;
use regex::Regex;
use std::sync::OnceLock;

static PATTERN: OnceLock<Regex> = OnceLock::new();

#[derive(Debug)]
struct TimeParseError(String);

impl std::fmt::Display for TimeParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl std::error::Error for TimeParseError {}

/// Transforms a description of relative time to a DateTime in UTC.
///
/// Relative times can be described in the following formats:
///
/// - `1 day ago`
/// - `3 days ago`
/// - `1 week ago`
///
pub fn parse_relative(string: &str, base: &DateTime<Utc>) -> AppResult<DateTime<Utc>> {
  let pattern = PATTERN.get_or_init(|| Regex::new(r"^(?P<amount>\d+) (?P<unit>\w+) ago$").unwrap());

  match pattern.captures(string) {
    Some(captures) => compute_relative(captures, base),
    None => Err(Box::new(TimeParseError("Pattern should match 'N <units> ago".into()))),
  }
}

fn compute_relative(captures: regex::Captures, base: &DateTime<Utc>) -> AppResult<DateTime<Utc>> {
  let duration = {
    let amount: i64 = captures.name("amount").unwrap().as_str().parse::<i64>()?;

    match captures.name("unit").unwrap().as_str() {
      "day" | "days" => chrono::Duration::try_days(amount),
      "month" | "months" => return relative_months(base, amount),
      "week" | "weeks" => chrono::Duration::try_weeks(amount),
      other => return Err(Box::new(TimeParseError(format!("Unknown unit {other}")))),
    }
  };

  match duration.and_then(|duration| base.checked_sub_signed(duration)) {
    Some(time) => Ok(time),
    None => Err(Box::new(TimeParseError("Unable to parse relative time".into()))),
  }
}

/// Clamps to the last day of the target month, so `1 month ago` from March 31 is February 28.
fn relative_months(base: &DateTime<Utc>, amount: i64) -> AppResult<DateTime<Utc>> {
  let months = u32::try_from(amount).map(Months::new)?;

  match base.checked_sub_months(months) {
    Some(time) => Ok(time),
    None => Err(Box::new(TimeParseError("Unable to parse relative time".into()))),
  }
}

#[cfg(test)]
mod tests {
  use chrono::prelude::*;
  use std::error::Error;

  #[test]
  fn parse_relative_weeks_ok() -> Result<(), Box<dyn Error>> {
    let naive = NaiveDate::from_ymd_opt(2022, 4, 1)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let now: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    let time = super::parse_relative("1 week ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 3, 25)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("2 weeks ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 3, 18)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_days_ok() -> Result<(), Box<dyn Error>> {
    let naive = NaiveDate::from_ymd_opt(2022, 4, 1)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let now: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    let time = super::parse_relative("4 days ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 3, 28)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_months_ok() -> Result<(), Box<dyn Error>> {
    let naive = NaiveDate::from_ymd_opt(2022, 4, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let now: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    let time = super::parse_relative("2 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 2, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("4 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 12, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("8 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 8, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("12 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 4, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("16 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2020, 12, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_months_last_day_ok() -> Result<(), Box<dyn Error>> {
    let naive = NaiveDate::from_ymd_opt(2022, 3, 31)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let now: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    let time = super::parse_relative("1 month ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 2, 28)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("4 months ago", &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 11, 30)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_months_from_leap_day_ok() -> Result<(), Box<dyn Error>> {
    let naive = NaiveDate::from_ymd_opt(2024, 2, 29)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let now: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    let time = super::parse_relative("3 months ago", &now)?;
    let naive = NaiveDate::from_ymd_opt(2023, 11, 29)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative("12 months ago", &now)?;
    let naive = NaiveDate::from_ymd_opt(2023, 2, 28)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_out_of_range_error() {
    for string in [
      "999999999999999 days ago",
      "999999999999999 weeks ago",
      "999999999999999 months ago",
    ] {
      assert!(super::parse_relative(string, &Utc::now()).is_err(), "{string}");
    }
  }

  #[test]
  fn parse_relative_error() {
    let error = super::parse_relative("hello", &Utc::now()).unwrap_err();
    assert!(error.is::<super::TimeParseError>());

    let error = super::parse_relative("2 moons ago", &Utc::now()).unwrap_err();
    assert!(error.is::<super::TimeParseError>());
  }
}
