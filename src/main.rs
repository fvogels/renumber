use std::fs::{self, DirEntry};
use clap::{command, Arg, ArgAction, Command};

fn main()
{
    let matches = cli().get_matches();
    let dry_run = matches.get_flag("dry-run");
    let minimal_index_width = matches.get_one::<usize>("min-width").unwrap();

    match fs::read_dir(".") {
        Err(error) => {
            println!("An error occurred while reading the current directory: {error}");
        },
        Ok(directory_contents) => {
            let mut paths: Vec<(DirEntry, IndexedFileName)> = Vec::new();

            for path in directory_contents {
                match path {
                    Err(error) => {
                        println!("An error occurred while iterating over directory contents: {error}")
                    },
                    Ok(directory_entry) => {
                        let filename = directory_entry.file_name();
                        let filename_string = filename.to_str().unwrap();

                        match extract_index_from_file_name(filename_string) {
                            Some(index) => {
                                paths.push((directory_entry, index));
                            },
                            None => { }
                        }
                    }
                }
            }

            paths.sort_by(|a, b| a.1.cmp(&b.1));
            let mut width = 1;
            {
                let mut n = 10;

                while n < paths.len() {
                    width += 1;
                    n *= 10;
                }
            }

            width = width.max(*minimal_index_width);

            for (new_index, (entry, indexed_file_name)) in paths.iter().enumerate() {
                let original_file_name = entry.file_name().into_string().unwrap();
                let new_file_name = format!("{:0width$}-{}", new_index, indexed_file_name.name, width=width);

                if dry_run {
                    if original_file_name != new_file_name {
                        println!("mv {} {}", original_file_name, new_file_name);
                    }
                }
                else {
                    if original_file_name == new_file_name {
                        println!("{}: already has correct file name", original_file_name);
                    }
                    else {
                        match std::fs::rename(&original_file_name, &new_file_name) {
                            Ok(_) => {
                                println!("{}: successfully renamed to {}", original_file_name, new_file_name);
                            },
                            Err(error) => {
                                println!("{}: FAILED to rename to {}: {}", original_file_name, new_file_name, error);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Eq, PartialEq)]
struct IndexedFileName {
    indices: Vec<u32>,
    name: String,
}

impl PartialOrd for IndexedFileName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return Some(self.cmp(other))
    }
}

impl Ord for IndexedFileName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut index = 0;

        loop {
            match (self.indices.get(index), other.indices.get(index)) {
                (None, None) => {
                    return std::cmp::Ordering::Equal;
                },
                (None, _) => {
                    return std::cmp::Ordering::Less;
                },
                (_, None) => {
                    return std::cmp::Ordering::Greater;
                },
                (Some(i), Some(j)) => {
                    match i.cmp(j) {
                        std::cmp::Ordering::Less => {
                            return std::cmp::Ordering::Less;
                        },
                        std::cmp::Ordering::Greater => {
                            return std::cmp::Ordering::Greater;
                        },
                        std::cmp::Ordering::Equal => {
                            // NOP
                        }
                    }
                }
            }

            index += 1;
        }
    }
}

fn extract_index_from_file_name(file_name: &str) -> Option<IndexedFileName> {
    let mut indices: Vec<u32> = Vec::new();
    let mut indices_i: usize = 0;
    let mut acc = 0;

    for (char_index, char) in file_name.chars().enumerate() {
        if char == '-' {
            indices.push(acc);
            indices_i = char_index + 1;
        }
        else {
            match char.to_digit(10) {
                None => {
                    if indices.len() == 0 {
                        return None
                    }
                    else {
                        break;
                    }
                },
                Some(digit) => {
                    acc = acc * 10 + digit;
                }
            }
        }
    }

    let file_name_remainder = String::from(&file_name[indices_i..]);
    return Some(IndexedFileName{indices: indices, name: file_name_remainder})
}

fn cli() -> Command {
    command!().args([
        Arg::new("dry-run")
            .long("dry-run")
            .short('n')
            .action(ArgAction::SetTrue).help("dry run mode"),
        Arg::new("min-width")
            .long("min-width")
            .short('m')
            .default_value("2")
            .value_parser(clap::value_parser!(usize))
            .action(ArgAction::Set).help("minimal index width"),
    ])
}