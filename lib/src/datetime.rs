use crate::{config::UnitEntry, duration::duration_from_parser};
use chrono::{prelude::*, Duration};
use chrono_tz::{Tz, UTC};
use pest::{
    error::{Error, ErrorVariant},
    Parser,
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "datetime.pest"]
pub(crate) struct DatetimeParser {}

fn map_weekday(rule: Rule) -> Weekday {
    match rule {
        Rule::monday => Weekday::Mon,
        Rule::tuesday => Weekday::Tue,
        Rule::wednesday => Weekday::Wed,
        Rule::thursday => Weekday::Thu,
        Rule::friday => Weekday::Fri,
        Rule::saturday => Weekday::Sat,
        Rule::sunday => Weekday::Sun,
        _ => unreachable!(),
    }
}

impl UnitEntry for chrono::DateTime<Utc> {
    type Error = Error<Rule>;
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
        let now = Utc::now();
        let now_date = now.date_naive();

        let mut parser = DatetimeParser::parse(Rule::timestamp, input.as_ref())?;
        let timestamp = parser.next().unwrap().into_inner().next().unwrap();
        let span = timestamp.as_span();
        match timestamp.as_rule() {
            Rule::full_len => {
                let mut full_len = timestamp.into_inner();
                let mut next = full_len.next();
                let weekday = if next.as_ref().unwrap().as_rule() == Rule::weekday {
                    let weekday_rule = next.unwrap().into_inner().next().unwrap().as_rule();
                    next = full_len.next();
                    Some(map_weekday(weekday_rule))
                } else {
                    None
                };
                // this one is either date or time
                let (year, month, day) = if next.as_ref().unwrap().as_rule() == Rule::date {
                    let mut date = next.unwrap().into_inner();
                    let year = date.next().unwrap();
                    let year: i32 = match year.as_rule() {
                        Rule::number_4b => year.as_str().parse().unwrap(),
                        Rule::number_2b => year.as_str().parse::<i32>().unwrap() + 2000,
                        _ => unreachable!(),
                    };
                    let month: u32 = date.next().unwrap().as_str().parse().unwrap();
                    let day: u32 = date.next().unwrap().as_str().parse().unwrap();

                    next = full_len.next();
                    (year, month, day)
                } else {
                    (now_date.year(), now_date.month(), now_date.day())
                };
                // this one could be time, timezone or None
                let (hour, minute, second, microsecond) =
                    if next.as_ref().is_some_and(|x| x.as_rule() == Rule::time) {
                        let mut time = next.unwrap().into_inner();
                        let hour: u32 = time.next().unwrap().as_str().parse().unwrap();
                        let minute: u32 = time.next().unwrap().as_str().parse().unwrap();
                        let second: u32 = time.next().map_or(0, |x| x.as_str().parse().unwrap());
                        let microsecond: i64 =
                            time.next().map_or(0, |x| x.as_str().parse().unwrap());

                        next = full_len.next();
                        (hour, minute, second, microsecond)
                    } else {
                        (0, 0, 0, 0)
                    };

                // this one could be timezone or None
                let timezone = if next.as_ref().is_some_and(|x| x.as_rule() == Rule::timezone) {
                    let span = next.as_ref().unwrap().as_span();
                    if let Ok(tz) = next.unwrap().as_str().parse::<Tz>() {
                        Some(tz)
                    } else {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Failed to parse timezone.".to_string(),
                            },
                            span,
                        ));
                    }
                } else {
                    None
                };

                let res = match timezone {
                    Some(tz) => tz
                        .with_ymd_and_hms(year, month, day, hour, minute, second)
                        .single()
                        .ok_or(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Failed to create timestamp.".to_string(),
                            },
                            span,
                        ))?
                        .with_timezone(&Utc),
                    None => Utc
                        .with_ymd_and_hms(year, month, day, hour, minute, second)
                        .single()
                        .ok_or(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Failed to create timestamp.".to_string(),
                            },
                            span,
                        ))?,
                };
                let res = res + Duration::microseconds(microsecond);

                // weekday validation
                if let Some(weekday) = weekday {
                    if weekday != res.weekday() {
                        return Err(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "The specified weekday does not match the specified date."
                                    .to_string(),
                            },
                            span,
                        ));
                    }
                }
                Ok(res)
            }
            Rule::relative => {
                let now = Utc::now();

                let mut relative = timestamp.into_inner();

                let variant = relative.next().unwrap();
                let rule = variant.as_rule();

                let timespan = variant.into_inner();
                let duration = duration_from_parser(timespan)?;
                match rule {
                    Rule::relative_forward => Ok(now + duration),
                    Rule::relative_backward => Ok(now - duration),
                    _ => unreachable!(),
                }
            }
            Rule::absolute => {
                let offset = input.as_ref()[1..].parse().unwrap();
                let naive =
                    NaiveDateTime::from_timestamp_opt(offset, 0).ok_or(Error::new_from_span(
                        ErrorVariant::CustomError {
                            message: "Invalid absolute timestamp.".to_string(),
                        },
                        span,
                    ))?;
                Ok(Utc.from_utc_datetime(&naive))
            }
            Rule::special => {
                let mut special = timestamp.into_inner();
                let keyword = special.next().unwrap();
                let timezone: Tz = if let Some(timezone) = special.next() {
                    if let Ok(tz) = timezone.as_str().parse() {
                        tz
                    } else {
                        UTC
                    }
                } else {
                    UTC
                };

                match keyword.as_rule() {
                    Rule::today => Ok(timezone
                        .with_ymd_and_hms(
                            now_date.year(),
                            now_date.month(),
                            now_date.day(),
                            0,
                            0,
                            0,
                        )
                        .single()
                        .ok_or(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Invalid timestamp.".to_string(),
                            },
                            span,
                        ))?
                        .with_timezone(&Utc)),
                    Rule::tomorrow => Ok(timezone
                        .with_ymd_and_hms(
                            now_date.year(),
                            now_date.month(),
                            now_date.day(),
                            0,
                            0,
                            0,
                        )
                        .single()
                        .ok_or(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Invalid timestamp.".to_string(),
                            },
                            span,
                        ))?
                        .with_timezone(&Utc)
                        + Duration::days(1)),
                    Rule::yesturday => Ok(timezone
                        .with_ymd_and_hms(
                            now_date.year(),
                            now_date.month(),
                            now_date.day(),
                            0,
                            0,
                            0,
                        )
                        .single()
                        .ok_or(Error::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Invalid timestamp.".to_string(),
                            },
                            span,
                        ))?
                        .with_timezone(&Utc)
                        - Duration::days(1)),
                    Rule::now => Ok(now),
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::UnitEntry;
    use chrono::{DateTime, TimeZone, Timelike, Utc};

    fn test_pairs(pair: &Vec<(&str, DateTime<Utc>)>) {
        for each in pair {
            let parse: DateTime<Utc> = UnitEntry::parse_from_str(each.0).unwrap();
            assert_eq!(parse, each.1);
            println!("{} passed.", each.0);
        }
    }

    #[test]
    fn test_datetime() {
        let pairs = vec![
            (
                "Fri 2012-11-23 11:12:13",
                Utc.with_ymd_and_hms(2012, 11, 23, 11, 12, 13).unwrap(),
            ),
            (
                "2012-11-23 11:12:13",
                Utc.with_ymd_and_hms(2012, 11, 23, 11, 12, 13).unwrap(),
            ),
            (
                "2012-11-23 11:12:13 UTC",
                Utc.with_ymd_and_hms(2012, 11, 23, 11, 12, 13).unwrap(),
            ),
            (
                "2012-11-23",
                Utc.with_ymd_and_hms(2012, 11, 23, 0, 0, 0).unwrap(),
            ),
            (
                "12-11-23",
                Utc.with_ymd_and_hms(2012, 11, 23, 0, 0, 0).unwrap(),
            ),
            (
                "11:12:13",
                Utc::now()
                    .with_hour(11)
                    .unwrap()
                    .with_minute(12)
                    .unwrap()
                    .with_second(13)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap(),
            ),
            (
                "11:12",
                Utc::now()
                    .with_hour(11)
                    .unwrap()
                    .with_minute(12)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap(),
            ),
            // cannot be tested due to processing time
            //             ("now", Utc::now()),
            (
                "today",
                Utc::now()
                    .with_hour(0)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap(),
            ),
            // cannot be tested due to processing time
            //            ("+3h30min", Utc::now() + Duration::minutes(3 * 60 + 30)),
            //            ("-5s", Utc::now() - Duration::seconds(5)),
            //            ("11min ago", Utc::now() - Duration::minutes(11)),
            (
                "@1395716396",
                // different from systemd examples
                Utc.with_ymd_and_hms(2014, 3, 25, 2, 59, 56).unwrap(),
            ),
        ];

        test_pairs(&pairs);
    }
}
