use crate::app::AppResult;
use chrono::prelude::*;

/// Transforms a UNIX timestamp to a DateTime in UTC.
pub fn parse_unix(time: &str) -> AppResult<DateTime<Utc>> {
  let timestamp = time.parse::<i64>()?;
  let datetime = DateTime::from_timestamp(timestamp, 0).unwrap();

  Ok(datetime)
}

#[cfg(test)]
mod tests {
  use chrono::prelude::*;
  use std::error::Error;
  use std::num::ParseIntError;

  #[test]
  fn parse_unix_ok() -> Result<(), Box<dyn Error>> {
    let time = super::parse_unix(&"1650989481".to_owned())?;

    let naive = NaiveDate::from_ymd_opt(2022, 4, 26)
      .unwrap()
      .and_hms_opt(16, 11, 21)
      .unwrap();
    let expected: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);

    assert_eq!(time, expected);

    Ok(())
  }

  #[test]
  fn parse_unix_error() {
    let error = super::parse_unix(&"hello".to_owned()).unwrap_err();
    assert!(error.is::<ParseIntError>());
  }
}
