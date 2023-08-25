use clap::{App, Arg};
use rand::{seq::SliceRandom, SeedableRng};
use regex::{Regex, RegexBuilder};
use std::{error::Error, fs, io::BufRead, path::PathBuf};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    source: String,
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("fortuner")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust fortune")
        .arg(
            Arg::with_name("files")
                .value_name("FILE")
                .multiple(true)
                .required(true)
                .help("Input files or directories"),
        )
        .arg(
            Arg::with_name("insensitive")
                .short("i")
                .long("insensitive")
                .takes_value(false)
                .help("Case-insensitive pattern matching"),
        )
        .arg(
            Arg::with_name("pattern")
                .value_name("PATTERN")
                .short("m")
                .long("pattern")
                .takes_value(true)
                .help("Pattern"),
        )
        .arg(
            Arg::with_name("seed")
                .value_name("SEED")
                .short("s")
                .long("seed")
                .takes_value(true)
                .help("Random seed"),
        )
        .get_matches();

    let sources = matches.values_of_lossy("files").unwrap();
    let pattern = matches
        .value_of("pattern")
        .map(|re| {
            RegexBuilder::new(re)
                .case_insensitive(matches.is_present("insensitive"))
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", re))
        })
        .transpose()?;
    let seed = matches.value_of("seed").map(parse_u64).transpose()?;

    Ok(Config {
        sources,
        pattern,
        seed,
    })
}

fn parse_u64(val: &str) -> MyResult<u64> {
    val.parse()
        .map_err(|_| format!("\"{}\" not a valid integer", val).into())
}

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let mut results = Vec::new();

    for path in paths {
        match fs::metadata(path) {
            Ok(metadata) if metadata.is_dir() => {
                for entry in WalkDir::new(path)
                    .into_iter()
                    .flatten()
                    .filter(|e| e.file_type().is_file())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|p| p.to_str().unwrap())
                            .unwrap_or("")
                            != "dat"
                    })
                {
                    results.push(entry.into_path());
                }
            }
            Ok(_) => results.push(PathBuf::from(path)),
            Err(err) => return Err(format!("{}: {}", path, err).into()),
        }
    }

    results.sort();
    results.dedup();
    Ok(results)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut results = vec![];

    for path in paths {
        let mut file = std::io::BufReader::new(
            fs::File::open(path).map_err(|e| format!("{}: {}", path.display(), e))?,
        );
        let mut buf = vec![];
        loop {
            let bytes = file.read_until(b'%', &mut buf)?;
            if bytes == 0 {
                break;
            }
            let mut text = String::from_utf8_lossy(&buf).trim().to_string();
            if &text == "%" || &text == "" {
                break;
            }
            text.pop();
            text = text.trim().to_owned();
            let result = Fortune {
                source: path.file_name().unwrap().to_string_lossy().to_string(),
                text,
            };
            results.push(result);
            buf.clear();
        }
    }
    Ok(results)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    match seed {
        Some(s) => {
            let mut seed = rand::rngs::StdRng::seed_from_u64(s);
            fortunes
                .choose(&mut seed)
                .map(|fortune| fortune.text.clone())
        }
        None => {
            let mut seed = rand::thread_rng();
            fortunes
                .choose(&mut seed)
                .map(|fortune| fortune.text.clone())
        }
    }
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    let mut prev_path = String::new();

    if let Some(pattern) = config.pattern {
        for fortune in fortunes {
            if pattern.is_match(&fortune.text) {
                if prev_path != fortune.source {
                    eprintln!("({})", fortune.source);
                    eprintln!("%");
                    prev_path = fortune.source;
                }
                println!("{}", fortune.text);
                println!("%");
            }
        }
    } else {
        if let Some(s) = pick_fortune(&fortunes, config.seed) {
            println!("{}", s);
        } else {
            println!("No fortunes found");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{find_files, parse_u64, pick_fortune, read_fortunes, Fortune};
    use std::path::PathBuf;

    #[test]
    fn test_parse_u64() {
        let res = parse_u64("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "\"a\" not a valid integer");

        let res = parse_u64("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = parse_u64("4");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 4);
    }

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                source: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups.".to_string(),
            },
            Fortune {
                source: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
