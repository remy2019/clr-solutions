use clap::{App, Arg};
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("headr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust head")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .default_value("-")
                .multiple(true)
                .help("Input file(s)"),
        )
        .arg(
            Arg::with_name("lines")
                .value_name("LINES")
                .short("n")
                .long("lines")
                .default_value("10")
                .help("Number of lines"),
        )
        .arg(
            Arg::with_name("bytes")
                .value_name("BYTES")
                .short("c")
                .long("bytes")
                .takes_value(true)
                .help("Number of bytes")
                .conflicts_with("lines"),
        )
        .get_matches();

    let lines = matches
        .value_of("lines")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal line count -- {}", e))?;

    let bytes = matches
        .value_of("bytes")
        .map(parse_positive_int)
        .transpose()
        .map_err(|e| format!("illegal byte count -- {}", e))?;

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines: lines.unwrap(),
        bytes,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let num_files = config.files.len();
    let first_file = config.files[0].clone();
    let mut result: Vec<&[u8]> = vec![];
    for filename in config.files {
        match open(&filename) {
            Err(err) => eprintln!("head: {}: {}", filename, err),
            Ok(f) => {
                if num_files > 1 {
                    if filename != first_file {
                        println!();
                    }
                    println!("==> {} <==", filename);
                }
                if let Some(c) = config.bytes {
                    read_bytes(f, c);
                } else {
                    read_lines(f, config.lines);
                }
            }
        }
    }
    Ok(())
}

fn read_lines(f: Box<dyn BufRead>, n: usize) {
    for line in f.split(0xA).take(n) {
        let line = line.unwrap();
        let result = String::from_utf8_lossy(&line);
        println!("{}", result);
    }
}

fn read_bytes(f: Box<dyn BufRead>, c: usize) {
    let mut u8buffer = vec![];
    let mut counter = c;
    'outer: for line in f.split(0xA) {
        let mut line = line.unwrap();
        line.push(0xA);
        let mut line = VecDeque::from(line);
        while let Some(x) = line.pop_front() {
            u8buffer.push(x);
            counter -= 1;
            if counter == 0 {
                break 'outer;
            }
        }
    }
    let result = String::from_utf8_lossy(&u8buffer);
    print!("{}", result);
}

fn parse_positive_int(val: &str) -> MyResult<usize> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(From::from(val)),
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

#[test]
fn test_parse_positive_int() {
    // 3 is an OK integer
    let res = parse_positive_int("3");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 3);

    // Any string is an error
    let res = parse_positive_int("foo");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "foo".to_string());

    // A zero is an error
    let res = parse_positive_int("0");
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "0".to_string());
}
