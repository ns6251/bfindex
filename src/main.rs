use std::{
    env, fs,
    hash::Hash,
    io::{self, stdin, Write},
    mem,
    path::{Path, PathBuf},
    process::exit,
};

use blake2::{Blake2b, Digest};

const HASH_N: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BfIndex(u64);

impl BfIndex {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn from_str(word: &str, keys: &[String; HASH_N]) -> Self {
        let mut x = Self::new();
        x.update(word, keys);
        x
    }

    pub fn update(&mut self, word: &str, keys: &[String; HASH_N]) {
        eprintln!("word: {:?}", word);
        for key in keys {
            let hash = Blake2b::new().chain(key).chain(word).finalize();

            let mut n = [0u8; 8];
            n.clone_from_slice(&hash.as_slice()[..8]);
            let n = unsafe { mem::transmute::<[u8; 8], u64>(n) };
            self.0 |= 1 << dbg!(n % 64);
        }
    }

    pub fn positive(&self, other: &Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Debug)]
pub struct Entry {
    bfindex: BfIndex,
    path: PathBuf,
}

impl Entry {
    pub fn new(path: &Path, keys: &[String; HASH_N]) -> io::Result<Self> {
        let buf = fs::read_to_string(path)?;

        let mut bfindex = BfIndex::new();
        buf.split_ascii_whitespace()
            .for_each(|word| bfindex.update(word, keys));

        let path = PathBuf::from(path);

        Ok(Self { bfindex, path })
    }

    pub fn contains(&self, s: &str) -> io::Result<bool> {
        let buf = fs::read_to_string(&self.path)?;
        Ok(buf.contains(s))
    }
}

#[derive(Debug)]
pub struct DataStore {
    data: Vec<Entry>,
    keys: [String; HASH_N],
}

impl DataStore {
    pub fn with_keys(keys: &[String; HASH_N]) -> Self {
        Self {
            data: Vec::new(),
            keys: keys.clone(),
        }
    }

    pub fn register(&mut self, path: &Path) -> io::Result<()> {
        let entry = Entry::new(path, &self.keys)?;
        self.data.push(entry);
        Ok(())
    }

    pub fn search(&self, word: &str) -> io::Result<()> {
        if word.len() == 0 {
            return Ok(());
        }

        let index = BfIndex::from_str(word, &self.keys);
        for entry in self.data.iter() {
            print!("{}:\t", entry.path.to_str().unwrap());
            if entry.bfindex.positive(&index) {
                if entry.contains(word)? {
                    println!("true positive!")
                } else {
                    println!("false positive!");
                }
            } else {
                println!("negative!");
            }
        }

        Ok(())
    }
}

fn main() {
    let keys = ["key1".into(), "key2".into(), "key3".into()];
    let mut db = DataStore::with_keys(&keys);

    let args = env::args().collect::<Vec<String>>();

    println!("register files ...");
    io::stdout().flush().unwrap();

    for path in args[1..].iter() {
        let path = Path::new(&path);
        if let Err(e) = db.register(&path) {
            eprintln!("{:?}: {:?}", path, e);
            exit(1);
        }
    }

    println!("done.");

    let mut buf = String::new();

    while {
        print!("enter a word: ");
        io::stdout().flush().unwrap();

        0 != stdin()
            .read_line(&mut buf)
            .expect("Failed to read from stdin.")
    } {
        if let Err(e) = db.search(&buf.trim()) {
            eprintln!("{:?}", e);
            exit(1);
        }
        buf.clear();
    }
}
