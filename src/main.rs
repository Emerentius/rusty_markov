use markov::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    const MEMORY_FILE: &str = "memory.zip";
    let memory = if Path::new(MEMORY_FILE).exists() {
        println!("Loading memory from file");
        let memory = Memory::load(MEMORY_FILE).expect("Could not read memory");
        println!("Done");
        memory
    } else {
        println!("Starting with a clean slate");
        let mut memory = Memory::default();
        println!("Learning...");
        let logs = File::open("logs.txt").expect("Could not open logs");
        let logs_reader = BufReader::new(logs);
        let mut count = 0;
        let start = time::precise_time_s();
        for line in logs_reader.lines() {
            if let Some(text) = line.as_ref().ok().and_then(|l| l.split('>').nth(1)) {
                count += 1;
                if count % 10000 == 0 {
                    println!("Line {}", count);
                }
                memory.learn(text);
            }
        }
        let end = time::precise_time_s();
        let time_diff = end - start;
        println!(
            "Learned {} lines in {:.2} s ({:.2} lines/sec)",
            count,
            time_diff,
            count as f64 / time_diff
        );
        println!("Saving memory");
        memory.save(MEMORY_FILE).expect("Could not save memory");
        memory
    };

    for word in &["hello", "Hmm", "butt", "sluttier", "nice"] {
        let line = memory.speak(word);
        println!("{:?} -> {:?}", word, line);
    }
}
