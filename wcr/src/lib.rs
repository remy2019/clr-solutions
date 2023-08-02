use clap::{App, Arg};
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    num_lines: usize,
    num_words: usize,
    num_bytes: usize,
    num_chars: usize,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("wcr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust wc")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .multiple(true)
                .help("Input file(s)")
                .default_value("-"),
        )
        .arg(
            Arg::with_name("lines")
                .short("l")
                .long("lines")
                .takes_value(false)
                .help("Show line count"),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .takes_value(false)
                .help("Show word count"),
        )
        .arg(
            Arg::with_name("bytes")
                .short("c")
                .long("bytes")
                .takes_value(false)
                .help("Show byte count"),
        )
        .arg(
            Arg::with_name("chars")
                .short("m")
                .long("chars")
                .takes_value(false)
                .conflicts_with("bytes")
                .help("Show character count"),
        )
        .get_matches();

    let mut lines = matches.is_present("lines");
    let mut words = matches.is_present("words");
    let mut bytes = matches.is_present("bytes");
    let chars = matches.is_present("chars");

    if [lines, words, bytes, chars].iter().all(|v| v == &false) {
        lines = true;
        words = true;
        bytes = true;
    }

    Ok(Config {
        files: matches.values_of_lossy("files").unwrap(),
        lines,
        words,
        bytes,
        chars,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let mut total_lines = 0;
    let mut total_words = 0;
    let mut total_bytes = 0;
    let mut total_chars = 0;

    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                let fileinfo = count(file)?;

                total_lines += fileinfo.num_lines;
                total_words += fileinfo.num_words;
                total_bytes += fileinfo.num_bytes;
                total_chars += fileinfo.num_chars;

                if config.lines {
                    print!("{:>8}", fileinfo.num_lines);
                }
                if config.words {
                    print!("{:>8}", fileinfo.num_words);
                }
                if config.bytes {
                    print!("{:>8}", fileinfo.num_bytes);
                }
                if config.chars {
                    print!("{:>8}", fileinfo.num_chars);
                }
                if filename != "-" {
                    println!(" {}", filename);
                } else {
                    println!();
                }
            }
        }
    }

    if config.files.len() > 1 {
        if config.lines {
            print!("{:>8}", total_lines);
        }
        if config.words {
            print!("{:>8}", total_words);
        }
        if config.bytes {
            print!("{:>8}", total_bytes);
        }
        if config.chars {
            print!("{:>8}", total_chars);
        }
        println!(" total");
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn count(mut file: impl BufRead) -> MyResult<FileInfo> {
    let mut num_lines = 0;
    let mut num_words = 0;
    let mut num_bytes = 0;
    let mut num_chars = 0;

    let mut buffer = String::new();
    loop {
        let byte = file.read_line(&mut buffer)?;
        if byte == 0 {
            break;
        }

        num_lines += 1;
        let mut prev = char::MAX;
        for c in buffer.chars() {
            if prev.is_ascii_whitespace() || prev == char::MAX {
                if c.is_alphanumeric() || c.is_ascii_punctuation() {
                    num_words += 1;
                }
            }
            prev = c;
            num_chars += 1;
        }
        num_bytes += buffer.bytes().count();
        buffer.clear();
    }

    Ok(FileInfo {
        num_lines,
        num_words,
        num_bytes,
        num_chars,
    })
}

#[cfg(test)]
mod tests {
    use super::{count, FileInfo};
    use std::io::Cursor;

    #[test]
    fn test_count() {
        let text = "I don't want the world. I just want your half.\r\n";
        let info = count(Cursor::new(text));
        assert!(info.is_ok());
        let expected = FileInfo {
            num_lines: 1,
            num_words: 10,
            num_chars: 48,
            num_bytes: 48,
        };
        assert_eq!(info.unwrap(), expected);
    }
}
