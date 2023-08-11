use crate::Extract::*;
use clap::{App, Arg, ArgGroup};
use std::{clone, error::Error, ops::Range};

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("cutr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust cut")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .default_value("-")
                .multiple(true)
                .help("Input file(s)"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("b")
                .long("bytes")
                .takes_value(true)
                .conflicts_with_all(&["chars", "fields"])
                .help("Selected bytes"),
        )
        .arg(
            Arg::with_name("chars")
                .value_name("CHARS")
                .short("c")
                .long("chars")
                .takes_value(true)
                .conflicts_with_all(&["bytes", "fields"])
                .help("Selected characters"),
        )
        .arg(
            Arg::with_name("delimiter")
                .value_name("DELIMITER")
                .short("d")
                .long("delim")
                .takes_value(true)
                .default_value("\t")
                .help("Field delimiter"),
        )
        .arg(
            Arg::with_name("fields")
                .value_name("FIELDS")
                .short("f")
                .long("fields")
                .takes_value(true)
                .conflicts_with_all(&["bytes", "chars"])
                .help("Selected fields"),
        )
        .get_matches();

    let delimiter = matches
        .value_of("delimiter")
        .map(|x| {
            x.parse::<char>()
                .map_err(|_| format!("--delim \"{}\" must be a single byte", x))
        })
        .unwrap()? as u8;
    let extract = if matches.is_present("bytes") {
        Bytes(parse_pos(matches.value_of("bytes").unwrap())?)
    } else if matches.is_present("chars") {
        Chars(parse_pos(matches.value_of("chars").unwrap())?)
    } else if matches.is_present("fields") {
        Fields(parse_pos(matches.value_of("fields").unwrap())?)
    } else {
        return Err("Must have --fields, --bytes, or --chars".into());
    };
    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        delimiter,
        extract,
    })
}

fn parse_pos(range: &str) -> MyResult<PositionList> {
    if range.is_empty() {
        return Err(From::from("Position cannot be empty"));
    }
    let mut buffer: PositionList = vec![];
    let temp = range.split(',');
    for r in temp {
        let result: Range<usize>;
        let numbers = r.split('-').collect::<Vec<_>>();
        let numbers = numbers
            .into_iter()
            .map(|number| {
                if !number.chars().all(|c| c.is_numeric()) {
                    Err(From::from(format!("illegal list value: \"{}\"", r)))
                } else {
                    if let Ok(x) = number.parse::<usize>() {
                        match x {
                            0 => Err(From::from(format!("illegal list value: \"{}\"", number))),
                            _ => Ok(x),
                        }
                    } else {
                        Err(From::from(format!("illegal list value: \"{}\"", number)))
                    }
                }
            })
            .collect::<Vec<MyResult<usize>>>();
        for number in &numbers {
            if let Err(e) = number {
                let string = e.to_string();
                return Err(string.into());
            }
        }
        let numbers = numbers.into_iter().flatten().collect::<Vec<usize>>();
        if numbers.len() == 1 {
            buffer.push(Range {
                start: numbers[0] - 1,
                end: numbers[0],
            });
        } else {
            if numbers[0] >= numbers[1] {
                return Err(From::from(format!(
                    "First number in range ({}) must be lower than second number ({})",
                    numbers[0], numbers[1]
                )));
            } else {
                buffer.push(Range {
                    start: numbers[0] - 1,
                    end: numbers[1],
                });
            }
        }
    }
    Ok(buffer)
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", &config);
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}
