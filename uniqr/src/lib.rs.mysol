use clap::{App, Arg};
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("uniqr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com")
        .about("Rust uniq")
        .arg(
            Arg::with_name("in_file")
                .value_name("IN_FILE")
                .default_value("-")
                .help("Input file"),
        )
        .arg(
            Arg::with_name("out_file")
                .value_name("OUT_FILE")
                .required(false)
                .help("Output file"),
        )
        .arg(
            Arg::with_name("count")
                .takes_value(false)
                .short("c")
                .long("count")
                .help("Show counts"),
        )
        .get_matches();

    Ok(Config {
        in_file: matches.value_of("in_file").unwrap().to_string(),
        out_file: matches.value_of("out_file").map(String::from),
        count: matches.is_present("count"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;
    let mut line = String::new();
    let mut prev = String::new();
    let mut count = 1;
    let mut out: Box<dyn Write>;

    if let Some(s) = config.out_file {
        out = Box::new(File::create(s)?);
    } else {
        out = Box::new(io::stdout());
    }

    loop {
        let bytes = file.read_line(&mut line)?;

        if bytes == 0 {
            if config.count && !prev.is_empty() {
                write!(out, "{:>4} ", count.to_string())?;
            }
            write!(out, "{}", prev)?;
            break;
        }
        if prev.trim() == line.trim() {
            count += 1;
        } else {
            if config.count && !prev.is_empty() {
                write!(out, "{:>4} ", count.to_string())?;
            }
            write!(out, "{}", prev)?;
            count = 1;
            prev = line.clone();
        }

        line.clear();
    }
    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
