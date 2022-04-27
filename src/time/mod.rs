use crate::app::AppResult;

use chrono::prelude::*;

pub fn parse_unix(time: String) -> AppResult<DateTime<Utc>> {
    let timestamp = time.parse::<i64>()?;
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    Ok(datetime)
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use std::error::Error;
    use std::num::ParseIntError;

    #[test]
    fn parse_unix() -> Result<(), Box<dyn Error>> {
        let time = super::parse_unix("1650989481".to_owned())?;

        let naive = NaiveDate::from_ymd(2022, 4, 26).and_hms(16, 11, 21);
        let expected: DateTime<Utc> = DateTime::from_utc(naive, Utc);

        assert_eq!(time, expected);
        Ok(())
    }

    #[test]
    fn parse_error() {
        let error = super::parse_unix("hello".to_owned()).unwrap_err();
        assert!(error.is::<ParseIntError>());
    }
}
