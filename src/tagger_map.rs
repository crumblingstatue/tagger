use std::collections::BTreeSet;
use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::io::prelude::*;
use std::path::Path;
use tagmap::TagMap;

pub struct TaggerMap {
    pub tag_map: TagMap<String, String>,
}

impl Default for TaggerMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TaggerMap {
    pub fn new() -> Self {
        TaggerMap {
            tag_map: TagMap::new(),
        }
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let reader = BufReader::new(fs::File::open(path)?);
        let mut map = TagMap::new();
        for line in reader.lines() {
            let line = line?;
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
    ///
    /// Returns how much entries were added.
    pub fn update_from_dir<P: AsRef<Path>>(&mut self, path: P) -> io::Result<usize> {
        use std::collections::btree_map::Entry;
        let mut added_count = 0;
        // Check for files that aren't part of the list and add them
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name().into_string().unwrap();
            if name != ::LIST_DEFAULT_FILENAME {
                if let Entry::Vacant(entry) = self.tag_map.entries.entry(name.clone()) {
                    println!("Adding {}", name);
                    entry.insert(Vec::new());
                    added_count += 1;
                }
            }
        }
        // Check for list entries that don't point to existing files and remove them
        let mut to_remove: Vec<String> = Vec::new();
        for k in self.tag_map.entries.keys() {
            if fs::metadata(k).is_err() {
                to_remove.push(k.clone());
            }
        }
        for k in to_remove {
            println!("Removing {}", k);
            self.tag_map.entries.remove(&k);
        }
        Ok(added_count)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut writer = BufWriter::new(fs::File::create(path)?);
        for (k, v) in &self.tag_map.entries {
            write!(writer, "\"{}\" ", k)?;
            for tag in v.iter() {
                write!(writer, "{} ", tag)?;
            }
            write!(writer, "\n")?;
        }
        Ok(())
    }

    /// Returns all the different tags that are present in the database.
    pub fn tags(&self) -> BTreeSet<String> {
        let mut set = BTreeSet::new();

        for tags in self.tag_map.entries.values() {
            for t in tags {
                set.insert(t.clone());
            }
        }

        set
    }
}
