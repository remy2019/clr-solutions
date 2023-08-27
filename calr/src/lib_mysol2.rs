use chrono::{naive::NaiveDate, Datelike, Local, Weekday};
use clap::{App, Arg};
use itertools::Itertools;
use std::{error::Error, iter::zip, str::FromStr};

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

const MONTH_NAMES: [&str; 12] = [
    "January",
    "Fabruary",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

const DAY_NAMES: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

type MyResult<T> = Result<T, Box<dyn Error>>;

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("calr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust cal")
        .arg(
            Arg::with_name("year")
                .value_name("YEAR")
                .help("Year (1-9999)"),
        )
        .arg(
            Arg::with_name("month")
                .value_name("MONTH")
                .short("m")
                .takes_value(true)
                .help("Month name or number (1-12)"),
        )
        .arg(
            Arg::with_name("show_year")
                .short("y")
                .long("year")
                .conflicts_with("year")
                .conflicts_with("month")
                .help("Show whole current year"),
        )
        .get_matches();

    let mut month = matches.value_of("month").map(parse_month).transpose()?;
    let mut year = matches.value_of("year").map(parse_year).transpose()?;
    let today = Local::today();
    if matches.is_present("show_year") {
        month = None;
        year = Some(today.year());
    } else if month.is_none() && year.is_none() {
        month = Some(today.month());
        year = Some(today.year());
    }

    Ok(Config {
        month,
        year: year.unwrap_or_else(|| today.year()),
        today: today.naive_local(),
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    parse_int(year).and_then(|num| {
        if (1..=9999).contains(&num) {
            Ok(num)
        } else {
            Err(format!("year \"{}\" not in the range 1 through 9999", year).into())
        }
    })
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int(month) {
        Ok(m) => match m {
            1..=12 => Ok(m),
            _ => Err(format!("month \"{}\" not in the range 1 through 12", month).into()),
        },
        Err(_) => {
            let lower = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();

            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month).into())
            }
        }
    }
}

fn format_month(year: i32, month: u32, print_year: bool, today: NaiveDate) -> Vec<String> {
    let last_day = last_day_in_month(year, month);
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut month_buffer = vec![];

    let header = format!(
        "{:^20}",
        format!(
            "{}{}",
            first_day.format("%B").to_string(),
            if print_year {
                first_day.format(" %Y").to_string()
            } else {
                "".to_string()
            }
        )
    );
    month_buffer.push(header);

    let subheader = DAY_NAMES.iter().join(" ");
    month_buffer.push(subheader);

    let mut whole_week_buffer = vec![];
    let mut week_buffer = vec![];
    for d in first_day.iter_days() {
        week_buffer.push(d.day());
        if d.weekday() == Weekday::Sat {
            whole_week_buffer.push(week_buffer.clone());
            week_buffer.clear();
        }

        if d == last_day {
            whole_week_buffer.push(week_buffer.clone());
            break;
        }
    }

    for (idx, w) in whole_week_buffer.into_iter().enumerate() {
        let raw_week = w.iter().map(|d| format!("{:>2}", d)).join(" ");
        let cooked_week = if idx == 0 {
            format!("{:>20}", raw_week)
        } else {
            format!("{:<20}", raw_week)
        };
        month_buffer.push(cooked_week);
    }

    let empty_line = " ".repeat(20);
    for idx in 0..8 {
        if month_buffer.get(idx).is_none() {
            month_buffer.push(empty_line.clone());
        }
    }

    month_buffer.iter_mut().for_each(|line| {
        line.push_str("  ");
    });

    let contains_today = (today.year() == year) && (today.month() == month);
    if contains_today {
        let today_day = today.day().to_string();
        for line in month_buffer.iter_mut().skip(2) {
            if line.contains(&today_day) {
                let matched = format!("{:>2}", today_day);
                let style = ansi_term::Style::new().reverse();
                *line = line.replace(
                    &matched,
                    &format!("{}{}{}", style.prefix(), matched, style.suffix()),
                );
                break;
            }
        }
    }

    month_buffer
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    let mut candidate = NaiveDate::from_ymd_opt(year, month, 24).unwrap();
    for d in 25..=32 {
        match NaiveDate::from_ymd_opt(year, month, d as u32) {
            Some(date) => candidate = date,
            None => break,
        }
    }
    candidate
}

pub fn run(config: Config) -> MyResult<()> {
    if let Some(month) = config.month {
        let buf = format_month(config.year, month, true, config.today);
        for line in buf {
            println!("{}", line);
        }
    } else {
        println!("{:>32}", config.year);
        let mut whole_buf = vec![];
        for i in [1, 4, 7, 10].into_iter() {
            let mut three_buf = vec![];
            let three_months: Vec<_> = (i..=i + 2)
                .map(|month| format_month(config.year, month, false, config.today))
                .collect();
            for ((x, y), z) in zip(
                zip(three_months[0].iter(), three_months[1].iter()),
                three_months[2].iter(),
            ) {
                three_buf.push(format!("{}{}{}", x, y, z));
            }
            whole_buf.push(three_buf.join("\n"));
        }
        println!("{}", whole_buf.join("\n\n"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month, parse_int, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_int() {
        // Parse positive int as usize
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        // Parse negative int as i32
        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        // Fail on a string
        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd(0, 1, 1);
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd(2021, 4, 7);
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(last_day_in_month(2020, 1), NaiveDate::from_ymd(2020, 1, 31));
        assert_eq!(last_day_in_month(2020, 2), NaiveDate::from_ymd(2020, 2, 29));
        assert_eq!(last_day_in_month(2020, 4), NaiveDate::from_ymd(2020, 4, 30));
    }
}
