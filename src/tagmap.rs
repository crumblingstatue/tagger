use std::{io, fs};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct TagMap<T: Eq + Hash, TAG: Eq> {
    entries: HashMap<T, Vec<TAG>>,
}

impl<T: Eq + Hash, TAG: Eq> TagMap<T, TAG> {
    pub fn new() -> Self {
        TagMap { entries: HashMap::new() }
    }
    pub fn entries_matching_tags(&self, tags: &[TAG]) -> Vec<&T> {
        let mut vec = Vec::new();
        'entries: for (k, v) in &self.entries {
            for tag in tags.iter() {
                let mut has_tag = false;
                for entry_tag in v.iter() {
                    if tag == entry_tag {
                        has_tag = true;
                    }
                }
                if !has_tag {
                    continue 'entries;
                }
            }
            vec.push(k);
        }
        vec
    }
}

impl TagMap<String, String> {
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let reader = BufReader::new(try!(fs::File::open(path)));
        let mut map = TagMap::new();
        for line in reader.lines() {
            let line = try!(line);
            let quot1 = line.find('"').unwrap();
            let quot2 = line[quot1 + 1..].find('"').unwrap();
            let filename = &line[quot1 + 1..quot2 + 1];
            let tags = line[quot2 + 2..]
                           .split_whitespace()
                           .map(|s| s.to_owned())
                           .collect::<Vec<_>>();
            map.entries.insert(filename.into(), tags);
      }
      Ok(map)
  }

  /// Add entries in a directory that aren't present in the List yet.
    pub fn update_from_dir<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        for entry in try!(fs::read_dir(path)) {
            let entry = try!(entry);
            let name = entry.file_name().into_string().unwrap();
            self.entries.entry(name).or_insert_with(Vec::new);
        }
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
          let mut writer = BufWriter::new(try!(fs::File::create(path)));
          for (k, v) in &self.entries {
              try!(write!(writer, "\"{}\" ", k));
              for tag in v.iter() {
                  try!(write!(writer, "{} ", tag));
              }
              try!(write!(writer, "\n"));
          }
          Ok(())
      }
}
