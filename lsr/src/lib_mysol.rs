use chrono::format;
use clap::{App, Arg};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::{error::Error, path::PathBuf};
use tabular::{Row, Table};

type MyResult<T> = Result<T, Box<dyn Error>>;

const RIGHT_MASKS: [u32; 9] = [
    0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001,
];

const RIGHT_SIGN: [&str; 3] = ["r", "w", "x"];

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    long: bool,
    show_hidden: bool,
}

pub fn get_args() -> MyResult<Config> {
    let matches = App::new("lsr")
        .version("0.1.0")
        .author("remy2019 <remy2019@gmail.com>")
        .about("Rust ls")
        .arg(
            Arg::with_name("paths")
                .value_name("PATH")
                .multiple(true)
                .default_value(".")
                .help("Files and/or directories"),
        )
        .arg(
            Arg::with_name("long")
                .short("l")
                .long("long")
                .help("Long listing"),
        )
        .arg(
            Arg::with_name("all")
                .short("a")
                .long("all")
                .help("Show all files"),
        )
        .get_matches();

    Ok(Config {
        paths: matches.values_of_lossy("paths").unwrap(),
        long: matches.is_present("long"),
        show_hidden: matches.is_present("all"),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let paths = find_files(&config.paths, config.show_hidden)?;
    if config.long {
        println!("{}", format_output(&paths)?);
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}

fn find_files(paths: &[String], show_hidden: bool) -> MyResult<Vec<PathBuf>> {
    let mut files = vec![];

    for path in paths {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    files.push(PathBuf::from(path));
                } else if metadata.is_dir() {
                    files.extend(
                        fs::read_dir(path)
                            .unwrap()
                            .filter_map(Result::ok)
                            .map(|entry| entry.path())
                            .filter_map(|entry| {
                                let name = entry.as_path().file_name().unwrap();
                                let is_hidden = name.to_string_lossy().starts_with('.');
                                (show_hidden || !is_hidden).then_some(entry)
                            }),
                    )
                }
            }
            Err(e) => eprintln!("{}: {}", path, e),
        }
    }
    Ok(files)
}

fn format_output(paths: &[PathBuf]) -> MyResult<String> {
    //         1   2     3     4     5     6     7     8
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let meta = path.metadata()?;
        table.add_row(
            Row::new()
                .with_cell(if meta.is_dir() { "d" } else { "-" }) // 1 "d" or "-"
                .with_cell(format_mode(meta.mode())) // 2 permissions
                .with_cell(meta.nlink()) // 3 number of links
                .with_cell(
                    users::get_user_by_uid(meta.uid())
                        .unwrap()
                        .name()
                        .to_string_lossy(),
                ) // 4 user name
                .with_cell(
                    users::get_group_by_gid(meta.gid())
                        .unwrap()
                        .name()
                        .to_string_lossy(),
                ) // 5 group name
                .with_cell(meta.len()) // 6 size
                .with_cell(
                    chrono::DateTime::<chrono::Utc>::from(meta.modified().unwrap())
                        .format("%b %e %y %R"),
                ) // 7 modification
                .with_cell(path.display()), // 8 path
        );
    }

    Ok(format!("{}", table))
}

/// Given a file mode in octal format like 0o751,
/// return a string like "rwxr-x--x"
fn format_mode(mode: u32) -> String {
    let mut result = vec![];
    for (sign, mask) in RIGHT_SIGN.repeat(3).into_iter().zip(RIGHT_MASKS.iter()) {
        if mode & mask != 0 {
            result.push(sign);
        } else {
            result.push("-");
        }
    }
    result.join("")
}

#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Find all nonhidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Find all entries in a directory
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(parts.len() > 0 && parts.len() <= 10);

        let perms = parts.get(0).unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(&line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> = out.split("\n").filter(|s| !s.is_empty()).collect();
        lines.sort();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            &empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(&dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }
}
