use crate::btree::BTree;
use std::fs::{File, OpenOptions};
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::mem;
use std::io::sink;

const LOG_FILE: &str = "log.txt";

pub struct LogManager<K, V>
where
    K: Ord + Clone + Debug + FromStr + Display,
    V: Clone + Debug + FromStr + Display,
    <K as FromStr>::Err: Debug,
    <V as FromStr>::Err: Debug,
{
    btree: BTree<K, V>,
    log_file: File,
}

impl<K: Ord + Clone + Debug + FromStr, V: Clone + Debug + FromStr> LogManager<K, V>
where
    K: Ord + Clone + Debug + FromStr + Display,
    V: Clone + Debug + FromStr + Display,
    <K as FromStr>::Err: Debug,
    <V as FromStr>::Err: Debug,
{
    pub fn new() -> Self {
        let btree = BTree::new();

        // Open or create the log file
        let log_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(LOG_FILE)
            .unwrap();

        let mut log_manager = LogManager { btree, log_file };

        // Recover the state from the log file
        log_manager.recover_state();

        log_manager
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.btree.insert(key.clone(), value.clone());
        Self::write_log(&mut self.log_file, format!("INSERT {} {}\n", key, value));
    }

    pub fn delete(&mut self, key: &K) {
        self.btree.delete(key);
        Self::write_log(&mut self.log_file, format!("DELETE {:?}\n", key));
    }

    pub fn search(&self, key: &K) -> Option<V> {
        self.btree.search(key).map(|v| v.clone())
    }

    fn recover_state(&mut self) {
        let reader = BufReader::new(&self.log_file);

        for line in reader.lines() {
            let line = line.unwrap();
            let mut tokens = line.split_whitespace();

            match tokens.next() {
                Some("INSERT") => {
                    let key_str = tokens.next().unwrap();
                    let value_str = tokens.next().unwrap();

                    println!("INSERT: key_str = {:?}, value_str = {:?}", key_str, value_str);

                    let key = key_str.parse::<K>().unwrap();
                    let value = value_str.parse::<V>().unwrap();

                    println!("INSERT: key = {:?}, value = {:?}", key, value);

                    self.btree.insert(key, value);
                }
                Some("DELETE") => {
                    let key_str = tokens.next().unwrap();
                    println!("DELETE: key_str = {:?}", key_str);

                    let key = key_str.parse::<K>().unwrap();

                    println!("DELETE: key = {:?}", key);

                    self.btree.delete(&key);
                }
                _ => panic!("Invalid log entry: {}", line),
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.persist_data();
    }


    fn write_log(log_file: &mut File, entry: String) {
        println!("Writing log entry: {}", entry);
        log_file.write_all(entry.as_bytes()).unwrap();
        log_file.flush().unwrap();
    }

    fn persist_data(&mut self) {
        // Create a new temporary log file
        let temp_log_path = "temp_log.txt";
        let mut temp_log_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(temp_log_path)
            .unwrap();

        let kv_pairs: Vec<_> = self.btree.traverse();
        println!("Persisting: {:?}", kv_pairs);

        // Write key-value pairs to the temporary log file
        for (key, value) in kv_pairs {
            println!("Persisting: {:?} {:?}", key, value);
            Self::write_log(&mut temp_log_file, format!("INSERT {} {}\n", key, value));
        }

        // Close the temporary log file
        drop(temp_log_file);

        // Replace the old log file with a sink (dummy writer) to enable dropping it
        let dummy_file_path = "dummy.txt";
        let old_log_file = std::mem::replace(&mut self.log_file, File::create(dummy_file_path).unwrap());
        drop(old_log_file); // Drop the old log file

        // Replace the old log file with the temporary log file
        fs::rename(temp_log_path, LOG_FILE).unwrap();

        // Open the new log file
        self.log_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(LOG_FILE)
            .unwrap();

        // Remove the dummy.txt file
        fs::remove_file(dummy_file_path).unwrap();
    }
}
