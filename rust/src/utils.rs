use std::env;
use std::fs::{File, read_to_string, read_dir};
use std::path::{Path, PathBuf};
use std::io::{self, BufRead};
use std::process::exit;
use std::process::Command;

use regex::Regex;
use inquire::Confirm;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Inquire error: {0}")]
    InquireError(#[from] inquire::InquireError),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Command execution failed: {0}")]
    CommandError(String),
}


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

pub fn home_dir() -> PathBuf {
    let home = match env::var(get_cfg()) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("couldn't interpret home directory: {}", e);
            std::process::exit(1);
        }
    };
    
    let mut path = PathBuf::from(home);
    path.push(".ssh");
    path
}

pub fn open_config() -> File {
    let mut config_path = home_dir();
    config_path.push("config");

    let file = match File::open(&config_path) {
        Err(why) => {
            if why.kind() == io::ErrorKind::NotFound {
                create_file(&config_path)
            } else {
                panic!("couldn't open {}: {}", config_path.display(), why)
            }
        }
        Ok(file) => file,
    };

    file
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

pub fn get_cmd_json(file: &str) -> Result<Value, AppError> {
    let mut file_path = home_dir();
    file_path.push(file);
    
    let data = match read_to_string(&file_path) {
        Ok(data) => data,
        Err(_) => {
            create_file(&file_path);
            let empty_json = serde_json::Value::Object(Default::default());
            let json_string = serde_json::to_string(&empty_json)?;
            std::fs::write(&file_path, json_string)?;
            return Ok(empty_json);
        },
    };

    let cmd_json: Value = serde_json::from_str(&data)?;
    Ok(cmd_json)
}


pub fn _find_pub_files(dir: &str) -> Result<Vec<String>, std::io::Error> {
    let mut pub_files = Vec::new();

    // 遍历目录
    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // 检查文件是否以 .pub 结尾
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pub") {
            pub_files.push(path.to_string_lossy().into_owned());
        }
    }

    Ok(pub_files)
}

pub fn _push_s_key(user: &str, hostname: &str, port: &str, key: &str) {
    // type %USERPROFILE%\.ssh\id_rsa.pub | ssh root@91.103.123.141 "mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys"
    let plat = cfg!(target_os = "windows");
    if plat {
        let cmd = format!("type %USERPROFILE%\\.ssh\\{}.pub | ssh {}@{} -p {} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"", key, user, hostname, port);
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
            .status()
            .expect("failed to execute process");

        if output.success() {
            print_success("Public key added successfully");
        } else {
            print_error("Failed to add public key");
        }
    } else {
        let cmd = format!("cat ~/.ssh/{}.pub | ssh {}@{} -p {} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"", key, user, hostname, port);
        let output = Command::new("sh")
            .args(&["-c", &cmd])
            .status()
            .expect("failed to execute process");

        if output.success() {
            print_success("Public key added successfully");
        } else {
            print_error("Failed to add public key");
        }
    }
}

pub fn genrsa(email: &str) {
    // ssh-keygen -t rsa -b 4096 -C "your_email@example.com"
    let cmd = format!("ssh-keygen -t rsa -b 4096 -C \"{}\"", email);

    let plat = cfg!(target_os = "windows");
    if plat {
        let output = Command::new("cmd")
            .args(&["/C", &cmd])
            .output()
            .expect("failed to execute process");

        if output.status.success() {
            print_success("RSA key generated successfully");
        } else {
            print_error("Failed to generate RSA key");
        }
    } else {
        let cmd = "ssh-keygen -t rsa -b 4096 -C \"{}\"".to_string();
        let output = Command::new("sh")
            .args(&["-c", &cmd])
            .output()
            .expect("failed to execute process");

        if output.status.success() {
            print_success("RSA key generated successfully");
        } else {
            print_error("Failed to generate RSA key");
        }
    }
}
