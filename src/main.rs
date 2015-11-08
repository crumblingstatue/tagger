extern crate tagmap;
extern crate clap;

use std::env;
use std::io::prelude::*;
use std::io::stderr;
use tagger_map::TaggerMap;
use infix::parse_infix;
use clap::{App, SubCommand, AppSettings};

mod tagger_map;
mod infix;

pub const LIST_DEFAULT_FILENAME: &'static str = "tagger.list";

fn run() -> i32 {
    let matches = App::new("tagger")
                      .setting(AppSettings::SubcommandRequiredElseHelp)
                      .subcommand(SubCommand::with_name("gen"))
                      .subcommand(SubCommand::with_name("update"))
                      .subcommand(SubCommand::with_name("filt").args_from_usage("[TAGS]..."))
                      .get_matches();
    if let Some(_) = matches.subcommand_matches("gen") {
        // TODO: Only allow gen if tagger.list doesn't exist.
        // Use "update" subcommand to update existing list.
        // Use --force to generate new list anyway.
        if let Ok(_) = std::fs::metadata(LIST_DEFAULT_FILENAME) {
            writeln!(stderr(),
                     "Error: {} already exists. Use `update` subcommand to update an existing \
                      list.",
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
    } else if let Some(_) = matches.subcommand_matches("update") {
        let mut list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => list,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        if let Err(e) = list.update_from_dir(env::current_dir().unwrap()) {
            writeln!(stderr(), "Error: {}", e).unwrap();
            return 1;
        }
        list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("filt") {
        let list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => list,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        let values = matches.values_of("TAGS").unwrap();
        let expr = values.join(" ");
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
    0
}

fn main() {
    std::process::exit(run());
}
