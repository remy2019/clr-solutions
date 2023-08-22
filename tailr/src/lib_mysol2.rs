use crate::TakeValue::*;
use clap::{App, Arg};
use once_cell::sync::OnceCell;
use regex::Regex;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, Read, Seek},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

static NUM_RE: OnceCell<Regex> = OnceCell::new();

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
    let num_re = NUM_RE.get_or_init(|| Regex::new(r"^([+-])?(\d+)$").unwrap());

    match num_re.captures(val) {
        Some(caps) => {
            let sign = caps.get(1).map_or("-", |m| m.as_str());
            let num = format!("{}{}", sign, caps.get(2).unwrap().as_str());
            if let Ok(val) = num.parse() {
                if sign == "+" && val == 0 {
                    Ok(PlusZero)
                } else {
                    Ok(TakeNum(val))
                }
            } else {
                Err(From::from(val))
            }
        }
        _ => Err(From::from(val)),
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();

    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    );
                }
                let (total_lines, total_bytes) = count_lines_bytes(filename)?;
                let reader = std::io::BufReader::new(file);
                if let Some(ref byte) = config.bytes {
                    print_bytes(reader, byte, total_bytes)?;
                } else {
                    print_lines(reader, &config.lines, total_lines)?;
                }
            }
        }
    }
    Ok(())
}

fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let file = std::io::BufReader::new(File::open(filename)?);
    let lines = file.lines().count();
    let file = std::io::BufReader::new(File::open(filename)?);
    let bytes = file.bytes().count();
    Ok((lines as i64, bytes as i64))
}

fn print_bytes<T>(mut file: T, num_bytes: &TakeValue, total_bytes: i64) -> MyResult<()>
where
    T: Read + Seek,
{
    if let Some(n) = get_start_index(num_bytes, total_bytes) {
        let buffer: Vec<u8> = file
            .bytes()
            .skip(n as usize)
            .filter_map(Result::ok)
            .collect();
        print!("{}", String::from_utf8_lossy(&buffer));
    }
    Ok(())
}

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(n) = get_start_index(num_lines, total_lines) {
        let mut buffer = String::new();
        let mut counter = 0;
        loop {
            let bytes = file.read_line(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            if counter < n {
                counter += 1;
                buffer.clear();
                continue;
            }
            print!("{}", buffer);
            buffer.clear();
        }
    }
    Ok(())
}

fn get_start_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    let empty = total == 0;
    let exceed_or_take_zero = if let TakeNum(n) = take_val {
        (total - n).is_negative() || n == &0
    } else {
        false
    };
    (!empty && !exceed_or_take_zero).then(|| match take_val {
        PlusZero => 0,
        &TakeNum(n) if n > 0 => n as u64 - 1,
        &TakeNum(n) => (total as u64).saturating_sub(n.wrapping_neg() as u64),
    })
}

#[cfg(test)]
mod tests {
    use super::{count_lines_bytes, get_start_index, parse_num, TakeValue::*};

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
    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&TakeNum(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&TakeNum(1), 10), Some(0));
        assert_eq!(get_start_index(&TakeNum(2), 10), Some(1));
        assert_eq!(get_start_index(&TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&TakeNum(-1), 10), Some(9));
        assert_eq!(get_start_index(&TakeNum(-2), 10), Some(8));
        assert_eq!(get_start_index(&TakeNum(-3), 10), Some(7));

        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&TakeNum(-20), 10), Some(0));
    }
}
