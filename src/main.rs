extern crate plist;

use std::process::Command;
use std::{thread, time};

mod diff;
mod ord;

use plist::Value;
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Cursor;

fn get_domains() -> Vec<String> {
    let output = Command::new("defaults")
        .args(&["domains"])
        .output()
        .expect("failed to execute defaults domains");
    let decoded = String::from_utf8(output.stdout).expect("failed to decode defaults output");
    decoded.split(", ").map(String::from).collect()
}

fn get_domain_output(domain: String) -> Value {
    let output = Command::new("defaults")
        .args(&["export", domain.as_str(), "-"])
        .output()
        .expect(format!("failed to execute defaults export for domain {}", domain).as_str());

    let cursor = Cursor::new(output.stdout);

    Value::from_reader(cursor)
        .expect(format!("failed to parse defaults export for domain {}", domain).as_str())
}

fn main() -> ! {
    let domains = get_domains();
    println!("Got {} domains", domains.len());
    let one_second = time::Duration::from_secs(1);
    let mut values: HashMap<_, _> = domains
        .par_iter()
        .map(|i| (i, get_domain_output(i.to_string())))
        .collect();

    loop {
        thread::sleep(one_second);
        let new_values: HashMap<_, _> = domains
            .par_iter()
            .map(|i| (i, get_domain_output(i.to_string())))
            .collect();
        let mut recorder = diff::Recorder::new();

        for (key, new_value) in new_values.iter() {
            let old_value = values.get(key).unwrap();
            diff::diff(&old_value, &new_value, &mut recorder, key.to_string());
        }
        values = new_values;
        println!("Results: {:?}", recorder);
    }
}
