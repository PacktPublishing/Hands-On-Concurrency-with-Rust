extern crate naive_hashmap;

use std::io;
use std::io::prelude::*;
use std::str::FromStr;

fn main() {
    let mut hash_map = naive_hashmap::HashMapU8::new();

    let n = io::stdin();
    for line in n.lock().lines() {
        if let Ok(line) = line {
            let mut cmd = line.split(" ");
            match cmd.next() {
                Some("LOOKUP") => {
                    if let Some(key) = cmd.next() {
                        if let Ok(key) = u8::from_str(key) {
                            let _ = hash_map.get(&key);
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                Some("INSERT") => {
                    if let Some(key) = cmd.next() {
                        if let Ok(key) = u8::from_str(key) {
                            if let Some(val) = cmd.next() {
                                let _ = hash_map.insert(key, val.to_string());
                            } else {
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            }
        } else {
            break;
        }
    }
}
