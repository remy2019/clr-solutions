use crate::EntryType::*;
use clap::{App, Arg};
use regex::Regex;
use std::error::Error;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Eq, PartialEq)]
enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    entry_types: Vec<EntryType>,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("findr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust find")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .multiple(true)
                .default_value(".")
                .help("Search paths"),
        )
        .arg(
            Arg::with_name("name")
                .value_name("NAME")
                .short("n")
                .long("name")
                .multiple(true)
                .takes_value(true)
                .help("Name"),
        )
        .arg(
            Arg::with_name("type")
                .value_name("TYPE")
                .short("t")
                .long("type")
                .takes_value(true)
                .multiple(true)
                .possible_values(&["f", "d", "l"])
                .help("Entry type"),
        )
        .get_matches();

    let mut names = vec![];
    if let Some(v) = matches.values_of_lossy("name") {
        for s in v {
            names.push(Regex::new(&s).map_err(|_e| format!("Invalid --name \"{}\"", s))?)
        }
    }

    let entry_types = matches.values_of_lossy("type").map_or(vec![], |t| {
        t.into_iter()
            .map(|t| match t.as_str() {
                "f" => File,
                "d" => Dir,
                "l" => Link,
                _ => panic!(),
            })
            .collect()
    });

    Ok(Config {
        paths: matches.values_of_lossy("path").unwrap(),
        names,
        entry_types,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:#?}", config);
    Ok(())
}
