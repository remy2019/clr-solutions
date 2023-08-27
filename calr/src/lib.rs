use chrono::{naive::NaiveDate, Datelike, Local};
use clap::{App, Arg};
use std::{error::Error, str::FromStr};

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

    let today = Local::today();
    let month = if matches.is_present("show_year")
        || (matches.is_present("year") && !matches.is_present("month"))
    {
        None
    } else {
        matches
            .value_of("month")
            .map(parse_month)
            .transpose()?
            .or(Some(today.month()))
    };
    let year = if let Some(y) = matches.value_of("year") {
        parse_year(y)?
    } else {
        today.year()
    };

    Ok(Config {
        month,
        year,
        today: today.naive_local(),
    })
}

fn parse_int<T: FromStr>(val: &str) -> MyResult<T> {
    val.parse::<T>()
        .map_err(|_| format!("Invalid integer \"{}\"", val).into())
}

fn parse_year(year: &str) -> MyResult<i32> {
    let year = parse_int::<i32>(year)?;
    match year {
        1..=9999 => Ok(year),
        _ => Err(format!("year \"{}\" not in the range 1 through 9999", year).into()),
    }
}

fn parse_month(month: &str) -> MyResult<u32> {
    match parse_int::<u32>(month) {
        Ok(m) => match m {
            1..=12 => Ok(m),
            _ => Err(format!("month \"{}\" not in the range 1 through 12", month).into()),
        },
        Err(_) => {
            let mut m = None;
            for (i, name) in MONTH_NAMES.into_iter().enumerate() {
                if name
                    .to_lowercase()
                    .starts_with(month.to_lowercase().as_str())
                {
                    if m.is_some() {
                        m = None;
                        break;
                    }
                    m = Some(i as u32 + 1);
                }
            }
            m.ok_or(format!("Invalid month \"{}\"", month).into())
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_int, parse_month, parse_year};

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
}
