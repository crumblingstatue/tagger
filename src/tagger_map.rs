use tagmap::TagMap;
use std::io::prelude::*;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;
use std::fs;

pub struct TaggerMap {
    pub tag_map: TagMap<String, String>,
}

impl TaggerMap {
    pub fn new() -> Self {
        TaggerMap { tag_map: TagMap::new() }
    }
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
        Ok(TaggerMap { tag_map: map })
    }

    /// Add entries in a directory that aren't present in the List yet.
    pub fn update_from_dir<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        for entry in try!(fs::read_dir(path)) {
            let entry = try!(entry);
            let name = entry.file_name().into_string().unwrap();
            self.tag_map.entries.entry(name).or_insert_with(Vec::new);
        }
        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut writer = BufWriter::new(try!(fs::File::create(path)));
        for (k, v) in &self.tag_map.entries {
            try!(write!(writer, "\"{}\" ", k));
            for tag in v.iter() {
                try!(write!(writer, "{} ", tag));
            }
            try!(write!(writer, "\n"));
        }
        Ok(())
    }
}
