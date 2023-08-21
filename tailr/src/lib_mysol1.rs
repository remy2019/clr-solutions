use crate::TakeValue::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
enum TakeValue {
    PlusZero,
    TakeNum(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TakeValue,
    bytes: Option<TakeValue>,
    quiet: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("tailr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust tail")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .multiple(true)
                .required(true)
                .help("Input file(s)"),
        )
        .arg(
            Arg::with_name("quiet")
                .takes_value(false)
                .short("q")
                .long("quiet")
                .help("Suppress headers"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .takes_value(true)
                .short("c")
                .long("bytes")
                .conflicts_with("lines")
                .help("Number of bytes"),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .takes_value(true)
                .short("n")
                .long("lines")
                .default_value("10")
                .help("Number of lines"),
        )
        .get_matches();

    let files = matches.values_of_lossy("files").unwrap();
    let quiet = matches.is_present("quiet");
    let bytes = matches
        .value_of("bytes")
        .map(parse_num)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;
    let lines = matches
        .value_of("lines")
        .map(parse_num)
        .unwrap()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    Ok(Config {
        files,
        lines,
        bytes,
        quiet,
    })
}

fn parse_num(val: &str) -> MyResult<TakeValue> {
    let num_re = Regex::new(r"^(-|\+)?(\d+)$").unwrap();
    num_re
        .captures(val)
        .ok_or(From::from(val))
        .and_then(|captures| {
            let sign = captures.get(1).map_or("-", |s| s.as_str());
            let num = captures.get(2).unwrap().as_str();

            match (sign, num) {
                ("+", "0") => Ok(PlusZero),
                _ => Ok(TakeNum(
                    format!("{}{}", sign, num).parse().map_err(|_| val)?,
                )),
            }
        })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_num, TakeValue::*};

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PlusZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), TakeNum(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any noninteger string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
