use std::env;
use std::fs::{File, read_to_string};
use std::path::Path;
use std::io::{self, BufRead};
use std::process::exit;

use regex::Regex;
use inquire::Confirm;
use serde_json::Value;

pub fn print_success(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn print_error(msg: &str) {
    println!("\x1b[31m{}\x1b[0m", msg);
}

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
    let home_dir = home + "\\.ssh";

    return home_dir;
}

pub fn open_config() -> File {
    let home_dir = home_dir() + "\\config";
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

pub fn get_hosts_all(file: File) -> Vec<String> {
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

pub fn get_cmd_json(file: &str) -> Value {
    let home_dir = home_dir() + "\\" + file;
    let path = Path::new(&home_dir);
    let data = read_to_string(path);

    let data = match data {
        Ok(data) => data,
        Err(_) => {
            // 创建新文件
            // 往其中写入json!({})
            create_file(&path);
            // 创建一个空的 JSON 对象
            let empty_json = serde_json::Value::Object(Default::default());
            // 将 JSON 对象转换为字符串
            let json_string = serde_json::to_string(&empty_json).unwrap();
            // 将字符串写入到文件中
            std::fs::write(&path, json_string).expect("Unable to write file");
            // 返回空的 JSON 对象
            return empty_json;
        },
    };

    let cmd_json: serde_json::Value = serde_json::from_str(&data).unwrap();

    return cmd_json;
}