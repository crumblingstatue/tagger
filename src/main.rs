#![feature(stmt_expr_attributes)]

extern crate tagmap;
extern crate clap;
extern crate rustyline;
#[cfg(feature = "random")]
extern crate rand;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate gdk;

use clap::{App, AppSettings, Arg, SubCommand};
use infix::parse_infix;
use rustyline::Editor;
use rustyline::completion::Completer;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::env;
use std::io::prelude::*;
use std::io::stderr;
use std::process::Command;
use tagger_map::TaggerMap;

mod tagger_map;
mod infix;
mod gui;

pub const LIST_DEFAULT_FILENAME: &'static str = "tagger.list";

struct TagCompleter {
    tags: BTreeSet<String>,
}

impl TagCompleter {
    fn new(tags: BTreeSet<String>) -> Self {
        TagCompleter { tags: tags }
    }
}

struct TagCompleterRefCell(RefCell<TagCompleter>);

impl Completer for TagCompleterRefCell {
    fn complete(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<String>)> {
        // Beginning of word is either space before it, or 0
        let begin = line.rfind(' ').map_or(0, |p| p + 1);
        let tags = &self.0.borrow().tags;
        let word = &line[begin..pos];
        let mut candidates = Vec::new();
        for t in tags {
            if t.starts_with(word) {
                candidates.push(t.to_owned());
            }
        }
        Ok((begin, candidates))
    }
}

fn run() -> i32 {
    let mut app = App::new("tagger");
    app = app.setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("gen"))
        .subcommand(SubCommand::with_name("update"))
        .subcommand(SubCommand::with_name("filt").args_from_usage("[TAGS]..."))
        .subcommand(SubCommand::with_name("add-tags").arg(Arg::with_name("TOOL")
            .short("w")
            .long("with")
            .required(true)
            .takes_value(true)
            .value_name("TOOL")))
        .subcommand(SubCommand::with_name("mv")
            .arg(Arg::with_name("src").required(true))
            .arg(Arg::with_name("dst").required(true)))
        .subcommand(SubCommand::with_name("list-tags"))
        .subcommand(SubCommand::with_name("gui"));
    if cfg!(feature = "random") {
        app = app.subcommand(SubCommand::with_name("random").args_from_usage("[TAGS]..."));
    }
    let matches = app.get_matches();
    macro_rules! load_map {
        () => {
            match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
                Ok(list) => list,
                Err(e) => {
                    writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                    return 1;
                }
            }
        }
    }
    macro_rules! parse_rule {
        ($matches:expr) => {{
            let expr = match $matches.values_of("TAGS") {
                Some(tags) => tags.collect::<Vec<_>>().join(" "),
                None => String::new(),
            };
            match parse_infix(&expr) {
                Ok(rule) => rule,
                Err(e) => {
                    writeln!(stderr(), "Error parsing infix expression: {}", e).unwrap();
                    return 1;
                }
            }
        }}
    }
    if matches.subcommand_matches("gen").is_some() {
        // TODO: Only allow gen if tagger.list doesn't exist.
        // Use "update" subcommand to update existing list.
        // Use --force to generate new list anyway.
        if std::fs::metadata(LIST_DEFAULT_FILENAME).is_ok() {
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
    } else if matches.subcommand_matches("update").is_some() {
        let mut list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => list,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        match list.update_from_dir(env::current_dir().unwrap()) {
            Ok(count) => {
                if count > 0 {
                    println!("Added {} entries.", count);
                } else {
                    println!("Already up to date.");
                }
            }
            Err(e) => {
                writeln!(stderr(), "Error: {}", e).unwrap();
                return 1;
            }
        }
        list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("filt") {
        let list = load_map!();
        let rule = parse_rule!(matches);
        for entry in list.tag_map.matching(&rule) {
            println!("{}", entry);
        }
    } else if let Some(matches) = matches.subcommand_matches("random") {
        #[cfg(feature = "random")]
        {
            use rand::{Rng, thread_rng};

            let list = load_map!();
            let rule = parse_rule!(matches);
            let matching = list.tag_map.matching(&rule).collect::<Vec<_>>();
            if let Some(choice) = thread_rng().choose(&matching) {
                println!("{}", choice);
            }
        }

    } else if let Some(matches) = matches.subcommand_matches("add-tags") {
        let tool_path = matches.value_of("TOOL").unwrap();
        let mut taggermap = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(taggermap) => taggermap,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        let completer = TagCompleterRefCell(RefCell::new(TagCompleter::new(taggermap.tags())));
        let mut editor = Editor::new();
        editor.set_completer(Some(&completer));
        for (k, v) in &mut taggermap.tag_map.entries {
            if v.is_empty() {
                let mut cmd = Command::new(tool_path).arg(k).spawn().unwrap();
                let line = editor.readline(&format!("Tags for {}: ", k)).unwrap();
                cmd.kill().unwrap();
                for word in line.split_whitespace() {
                    v.push(word.to_owned());
                    completer.0.borrow_mut().tags.insert(word.to_owned());
                }
                editor.add_history_entry(&line);
            }
        }
        taggermap.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
    } else if let Some(matches) = matches.subcommand_matches("mv") {
        let src = matches.value_of("src").unwrap();
        let dst = matches.value_of("dst").unwrap();
        let mut list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => list,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        let value = list.tag_map.entries.remove(src).expect("Didn't find entry.");
        list.tag_map.entries.insert(dst.to_owned(), value);
        std::fs::rename(src, dst).unwrap();
        list.save_to_file(LIST_DEFAULT_FILENAME).unwrap();
    } else if matches.subcommand_matches("list-tags").is_some() {
        let list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => list,
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        let tags = list.tags();
        for tag in tags {
            println!("{}", tag);
        }
    } else if matches.subcommand_matches("gui").is_some() {
        use std::rc::Rc;
        use std::cell::RefCell;

        let list = match TaggerMap::from_file(LIST_DEFAULT_FILENAME) {
            Ok(list) => Rc::new(RefCell::new(list)),
            Err(e) => {
                writeln!(stderr(), "Error opening {}: {}", LIST_DEFAULT_FILENAME, e).unwrap();
                return 1;
            }
        };
        gui::run(list);
    }
    0
}

fn main() {
    std::process::exit(run());
}
