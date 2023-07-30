use crate::config::UnitEntry;
use chrono::Duration;

impl UnitEntry for Duration {
    type Error = ();
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
        enum Token {
            Integer,
            Character,
            SOI,
        }
        let mut result = Vec::new();

        let input = input.as_ref().chars();
        let mut integer = Vec::new();
        let mut suffix = Vec::new();
        let mut prev = Token::SOI;

        fn segment(
            integer: &mut Vec<char>,
            suffix: &mut Vec<char>,
        ) -> std::result::Result<Duration, ()> {
            let int: i64 = integer.iter().collect::<String>().parse().map_err(|_| ())?;

            const SECONDS_IN_MONTH: i64 = /* 30.44 * 24 * 60 * 60 = */ 2630016;
            const SECONDS_IN_YEAR: i64 = /* 365.25 * 24 * 60 * 60 = */ 31557600;

            let suf: String = suffix.iter().collect();

            integer.clear();
            suffix.clear();

            Ok(match suf.as_str() {
                "usec" | "us" | "μs" => Duration::microseconds(int),
                "msec" | "ms" => Duration::milliseconds(int),
                "seconds" | "second" | "sec" | "s" => Duration::seconds(int),
                "minutes" | "minute" | "min" | "m" => Duration::minutes(int),
                "hours" | "hour" | "hr" | "h" => Duration::hours(int),
                "days" | "day" | "d" => Duration::days(int),
                "weeks" | "week" | "w" => Duration::weeks(int),
                "months" | "month" | "M" => Duration::days(int * SECONDS_IN_MONTH),
                "years" | "year" | "y" => Duration::days(int * SECONDS_IN_YEAR),
                _ => {
                    dbg!("unmatched suffix: {suffix}");
                    return Err(());
                }
            })
        }

        for cursor in input {
            match cursor {
                '0'..='9' => {
                    if let Token::Character = prev {
                        // end of a set
                        let partial = segment(&mut integer, &mut suffix)?;
                        result.push(partial);
                    }
                    integer.push(cursor);
                    prev = Token::Integer;
                }
                'a'..='z' | 'A'..='Z' => {
                    suffix.push(cursor);
                    prev = Token::Character;
                }
                ' ' => {
                    continue;
                }
                _ => {
                    dbg!("unmatched token: {cursor}");
                    return Err(());
                }
            }
        }
        if !integer.is_empty() & !suffix.is_empty() {
            let partial = segment(&mut integer, &mut suffix)?;
            result.push(partial);
        } else {
            dbg!("empty");
            return Err(());
        }

        Ok(result
            .into_iter()
            .reduce(|x, y| x + y)
            .unwrap_or(Duration::zero()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::config::UnitEntry;

    #[test]
    fn single() {
        let pairs = vec![
            ("1us", Duration::microseconds(1)),
            ("2ms", Duration::milliseconds(2)),
            ("3s", Duration::seconds(3)),
            ("4m", Duration::minutes(4)),
            ("5h", Duration::hours(5)),
            ("6d", Duration::days(6)),
        ];
        test_pairs(&pairs);
    }

    #[test]
    fn complex() {
        let pairs = vec![
            ("5s400ms", Duration::milliseconds(5400)),
            ("3w 5d", Duration::days(26)),
        ];
        test_pairs(&pairs);
    }

    fn test_pairs(pair: &Vec<(&str, Duration)>) {
        for each in pair {
            let parse = Duration::parse_from_str(each.0).unwrap();
            assert_eq!(parse, each.1);
        }
    }
}
