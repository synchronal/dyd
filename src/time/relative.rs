use crate::app::AppResult;

use chrono::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug)]
struct TimeParseError(String);

impl std::fmt::Display for TimeParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "There is an error: {}", self.0)
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
  lazy_static! {
    static ref PATTERN: Regex = Regex::new(r"^(?P<amount>\d+) (?P<unit>\w+) ago$").unwrap();
  }

  match PATTERN.captures(string) {
    Some(captures) => compute_relative(captures, base),
    None => Err(Box::new(TimeParseError("Pattern should match 'N <units> ago".into()))),
  }
}

fn compute_relative(captures: regex::Captures, base: &DateTime<Utc>) -> AppResult<DateTime<Utc>> {
  let duration = {
    let amount: i64 = captures.name("amount").unwrap().as_str().parse::<i64>()?;

    match captures.name("unit").unwrap().as_str() {
      "day" | "days" => chrono::Duration::days(amount),
      "month" | "months" => return relative_months(base, amount),
      "week" | "weeks" => chrono::Duration::weeks(amount),
      other => return Err(Box::new(TimeParseError(format!("Unknown unit {other}")))),
    }
  };

  match base.checked_sub_signed(duration) {
    Some(time) => Ok(time),
    None => Err(Box::new(TimeParseError("Unable to parse relative time".into()))),
  }
}

fn relative_months(base: &DateTime<Utc>, amount: i64) -> AppResult<DateTime<Utc>> {
  let current_month = base.month0();
  let new_month = (current_month as i64) - amount;

  if new_month < 0 {
    let current_year = base.year();
    let amount = (new_month + 1).abs();
    relative_months(
      &base
        .with_year(current_year - 1)
        .unwrap()
        .with_month(12)
        .unwrap(),
      amount,
    )
  } else {
    match base.with_month0(new_month as u32) {
      Some(datetime) => Ok(datetime),
      None => {
        let year = base.year();
        let last_day = get_days_from_month(year, (new_month + 1) as u32);

        Ok(
          base
            .with_day(1)
            .unwrap()
            .with_month0(new_month as u32)
            .unwrap()
            .with_day(last_day as u32)
            .unwrap(),
        )
      }
    }
  }
}

fn get_days_from_month(year: i32, month: u32) -> i64 {
  NaiveDate::from_ymd_opt(
    match month {
      12 => year + 1,
      _ => year,
    },
    match month {
      12 => 1,
      _ => month + 1,
    },
    1,
  )
  .unwrap()
  .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
  .num_days()
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

    let time = super::parse_relative(&"1 week ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 3, 25)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"2 weeks ago".to_owned(), &now).unwrap();
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

    let time = super::parse_relative(&"4 days ago".to_owned(), &now).unwrap();
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

    let time = super::parse_relative(&"2 months ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 2, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"4 months ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 12, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"8 months ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 8, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"12 months ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 4, 2)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"16 months ago".to_owned(), &now).unwrap();
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

    let time = super::parse_relative(&"1 month ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2022, 2, 28)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    let time = super::parse_relative(&"4 months ago".to_owned(), &now).unwrap();
    let naive = NaiveDate::from_ymd_opt(2021, 11, 30)
      .unwrap()
      .and_hms_opt(6, 1, 2)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_relative_error() {
    let error = super::parse_relative(&"hello".to_owned(), &Utc::now()).unwrap_err();
    assert!(error.is::<super::TimeParseError>());

    let error = super::parse_relative(&"2 moons ago".to_owned(), &Utc::now()).unwrap_err();
    assert!(error.is::<super::TimeParseError>());
  }
}
