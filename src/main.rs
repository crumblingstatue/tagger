use std::env;
use std::io::prelude::*;
use std::io::stderr;

mod list;

fn usage(cmd_name: &str) -> String {
    format!("Usage: {} gen/filt", cmd_name)
}

const LIST_DEFAULT_FILENAME: &'static str = "tagger.list";

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
                    writeln!(stderr(), "Error: {} already exists. Use `update` subcommand \
                                        to update an existing list.", LIST_DEFAULT_FILENAME).unwrap();
                    return 1;
                }
                let mut list = list::List::new();
                if let Err(e) = list.update_from_dir(env::current_dir().unwrap()) {
                    writeln!(stderr(), "Error: {}", e).unwrap();
                    return 1;
                }
                list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
            }
            "filt" => {
                let list = list::List::from_file(LIST_DEFAULT_FILENAME).unwrap();
                let tags = args.collect::<Vec<String>>();
                for entry in list.entries_matching_tags(&tags) {
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
