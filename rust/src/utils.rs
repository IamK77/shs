use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead};

use regex::Regex;


pub fn open_config() -> File {
    let mut home = String::new();

    match env::var("USERPROFILE") {
        Ok(val) => home = val,
        Err(e) => println!("couldn't interpret: {}", e),
    }
    let home_dir = home + "\\.ssh\\config";
    let path = Path::new(&home_dir);

    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    return file;
}

pub fn get_hosts(file: File) -> Vec<String> {
    let reader = io::BufReader::new(file);
    let mut confs = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap(); // Ignore errors.
        if let Some(found) = line.find("#") {
            if found == 0 {
                continue;
            }
            let (line, _after) = line.split_at(found);
            let line = line.trim().to_string();
            confs.push(line);
        } else {
            let line = line.trim().to_string();
            confs.push(line);
        }
    }

    return confs;
}

pub fn hosts_sort(confs: Vec<String>) -> Vec<String> {
    let mut hosts: Vec<String> = Vec::new();
    let re = Regex::new(r"Host\s+(?P<host>\S+)").unwrap();
    for conf in confs {
        let caps = re.captures(&conf);
        if let Some(caps) = caps {
            hosts.push(caps["host"].to_string());
        }
    }

    hosts.sort_by(|a, b| {
        let a_is_digit = a.chars().next().unwrap().is_numeric();
        let b_is_digit = b.chars().next().unwrap().is_numeric();

        if a_is_digit && !b_is_digit {
            std::cmp::Ordering::Greater
        } else if !a_is_digit && b_is_digit {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    return hosts;
}