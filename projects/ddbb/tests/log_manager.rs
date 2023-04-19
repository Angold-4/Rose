use ddbb::btree::BTree;
use ddbb::log::LogManager;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::path::Path;
use std::fs;

const LOG_FILE: &str = "log.txt";

#[test]
fn test_log_manager_complex() {
    if Path::new(LOG_FILE).exists() {
        fs::remove_file(LOG_FILE).unwrap();
    }

    let mut log_manager = LogManager::new();

    // Insert 100 key-value pairs
    for i in 1..=100 {
        log_manager.insert(format!("key{}", i), i);
    }

    // Check if all key-value pairs were inserted correctly
    for i in 1..=100 {
        assert_eq!(log_manager.search(&format!("key{}", i)), Some(i));
    }

    // Delete some key-value pairs
    for i in 1..=100 {
        if i % 2 == 0 {
            log_manager.delete(&format!("key{}", i));
        }
    }

    // Check if the deleted key-value pairs are removed
    for i in 1..=100 {
        if i % 2 == 0 {
            assert_eq!(log_manager.search(&format!("key{}", i)), None);
        } else {
            assert_eq!(log_manager.search(&format!("key{}", i)), Some(i));
        }
    }

    log_manager.shutdown();
    println!("Log manager shutdown");

    let log_manager2 = LogManager::new();

    // Check if the state is preserved after shutdown
    for i in 1..=100 {
        if i % 2 == 0 {
            assert_eq!(log_manager2.search(&format!("key{}", i)), None);
        } else {
            assert_eq!(log_manager2.search(&format!("key{}", i)), Some(i));
        }
    }
}
