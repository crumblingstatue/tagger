extern crate tagmap;

use std::env;
use std::io::prelude::*;
use std::io::stderr;
use tagger_map::TaggerMap;
use infix::parse_infix;

mod tagger_map;
mod infix;

fn usage(cmd_name: &str) -> String {
    format!("Usage: {} gen/filt/update", cmd_name)
}

pub const LIST_DEFAULT_FILENAME: &'static str = "tagger.list";

fn run() -> i32 {
    let mut args = env::args();
    let cmd_name = args.next().unwrap();
    if let Some(subcommand) = args.next() {
        match &subcommand[..] {
            "gen" => {
                // TODO: Only allow gen if tagger.list doesn't exist.
                // Use "update" subcommand to update existing list.
                // Use --force to generate new list anyway.
                if let Ok(_) = std::fs::metadata(LIST_DEFAULT_FILENAME) {
                    writeln!(stderr(),
                             "Error: {} already exists. Use `update` subcommand to update an \
                              existing list.",
                             LIST_DEFAULT_FILENAME)
                        .unwrap();
                    return 1;
                }
                let mut list = TaggerMap::new();
                if let Err(e) = list.update_from_dir(env::current_dir().unwrap()) {
                    writeln!(stderr(), "Error: {}", e).unwrap();
                    return 1;
                }
                list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
            }
            "update" => {
                let mut list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
                    Ok(list) => list,
                    Err(e) => {
                        writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e)
                            .unwrap();
                        return 1;
                    }
                };
                if let Err(e) = list.update_from_dir(env::current_dir().unwrap()) {
                    writeln!(stderr(), "Error: {}", e).unwrap();
                    return 1;
                }
                list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
            }
            "filt" => {
                let list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
                    Ok(list) => list,
                    Err(e) => {
                        writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e)
                            .unwrap();
                        return 1;
                    }
                };
                let expr = args.collect::<Vec<_>>().join(" ");
                let rule = match parse_infix(&expr) {
                    Ok(rule) => rule,
                    Err(e) => {
                        writeln!(stderr(), "Error parsing infix expression: {}", e).unwrap();
                        return 1;
                    }
                };
                for entry in list.tag_map.matching(&rule) {
                    println!("{}", entry);
                }
            }
            unk => {
                writeln!(stderr(), "Unknown subcommand: '{}'", unk).unwrap();
                return 1;
            }
        }
    } else {
        writeln!(stderr(), "{}", usage(&cmd_name)).unwrap();
        return 1;
    }
    0
}

fn main() {
    std::process::exit(run());
}
