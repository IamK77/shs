use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{self, BufRead};
use std::process::exit;

use regex::Regex;
use inquire::Confirm;

fn create_file(path: &Path) -> File {
    let ans = Confirm::new("Do you want to create a new config file?")
        .with_default(true)
        .with_help_message("This will create a new config file in your home directory(default is yes)")
        .prompt();

    match ans {
        Ok(ans) => {
            if ans {
                let file = match File::create(&path) {
                    Err(why) => panic!("couldn't create {}: {}", path.display(), why),
                    Ok(file) => {
                        println!("\x1b[32mCreated a new config file in {}\x1b[32m", path.display());
                        file
                    },
                };
                return file;
            } else {
                println!("You can't proceed without a config file");
                std::process::exit(1);
            }
        }
        Err(_) => {
            println!("You can't proceed without a config file");
            std::process::exit(1);
        }
    }
}

fn get_cfg() -> String{
    if cfg!(target_os = "windows") {
        return "USERPROFILE".to_string();
    } else if cfg!(target_os = "macos") {
        return "HOME".to_string();
    } else if cfg!(target_os = "linux") {
        return "HOME".to_string();
    } else {
        return "HOME".to_string();
    }
}

pub fn home_dir() -> String {
    let mut home = String::new();

    match env::var(get_cfg()) {
        Ok(val) => home = val,
        Err(e) => println!("couldn't interpret: {}", e),
    }
    let home_dir = home + "\\.ssh\\config";

    return home_dir;
}

pub fn open_config() -> File {
    let home_dir = home_dir();
    let path = Path::new(&home_dir);

    let file = match File::open(&path) {
        Err(why) => {
            if why.kind() == io::ErrorKind::NotFound {
                create_file(&path)
            } else {
                panic!("couldn't open {}: {}", path.display(), why)
            }
        }
        Ok(file) => file,
    };

    return file;
}

pub fn get_hosts(file: File) -> Vec<String> {
    let reader = io::BufReader::new(file);
    let mut confs = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap_or_else(|_| {
            eprintln!("\x1b[31mThe configuration file is empty. Please add a new host.\x1b[0m");
            exit(0);
        });
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