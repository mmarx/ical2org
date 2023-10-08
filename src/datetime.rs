use std::fmt::Display;

use chrono::{DateTime, NaiveTime, TimeZone, Timelike};

/// Formats a datetime for org agendas, using the given timezone.
pub fn org_datetime<Tz, Ty>(datetime: DateTime<Tz>, timezone: &Ty) -> String
where
    Ty: TimeZone,
    Tz: TimeZone,
    Ty::Offset: Display,
{
    datetime
        .with_timezone(timezone)
        .format("<%Y-%m-%d %a %H:%M>")
        .to_string()
}

/// Formats a date for org agendas, using the given timezone.
pub fn org_date<Tz, Ty>(date: DateTime<Tz>, timezone: &Ty) -> String
where
    Ty: TimeZone,
    Tz: TimeZone,
    Ty::Offset: Display,
{
    date.with_timezone(timezone)
        .format("<%Y-%m-%d %a>")
        .to_string()
}

/// Checks whether the given time denotes midnight.
pub fn is_midnight(time: NaiveTime) -> bool {
    time.hour() == 0 && time.minute() == 0 && time.second() == 0 && time.nanosecond() == 0
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    use chrono::Utc;
    use chrono_tz::Europe::Prague;

    #[test]
    fn datetime() {
        assert_eq!(
            org_datetime(
                Utc.with_ymd_and_hms(2017, 12, 15, 17, 35, 0).unwrap(),
                &Prague
            ),
            "<2017-12-15 Fri 18:35>"
        );
        assert_eq!(
            org_datetime(
                Utc.with_ymd_and_hms(2017, 12, 15, 18, 35, 0).unwrap(),
                &Prague
            ),
            "<2017-12-15 Fri 19:35>"
        );
    }

    #[test]
    fn date() {
        assert_eq!(
            org_date(
                Utc.with_ymd_and_hms(2017, 12, 15, 17, 35, 0).unwrap(),
                &Prague
            ),
            "<2017-12-15 Fri>"
        );
        assert_eq!(
            org_date(
                Utc.with_ymd_and_hms(2017, 12, 15, 18, 35, 0).unwrap(),
                &Prague
            ),
            "<2017-12-15 Fri>"
        );
    }
}
