use std::env;
use std::fs::{File, read_to_string, read_dir};
use std::path::{Path, PathBuf};
use std::io::{self, BufRead};
use std::process::Command;

use regex::Regex;
use inquire::Confirm;
use serde_json::Value;

use crate::error::{Result, ShsError};


pub fn print_success(msg: &str) {
    println!("\x1b[32m{}\x1b[0m", msg);
}

pub fn print_error(msg: &str) {
    println!("\x1b[31m{}\x1b[0m", msg);
}

fn create_file(path: &Path) -> Result<File> {
    let ans = Confirm::new("Do you want to create a new config file?")
        .with_default(true)
        .with_help_message(&format!(
            "This will create {} (default is yes)",
            path.display()
        ))
        .prompt()?;

    if !ans {
        return Err(ShsError::Aborted(
            "You can't proceed without a config file".into(),
        ));
    }

    let file = File::create(path)?;
    print_success(&format!("Created a new config file in {}", path.display()));
    Ok(file)
}

fn home_env_var() -> &'static str {
    if cfg!(target_os = "windows") {
        "USERPROFILE"
    } else {
        "HOME"
    }
}

pub fn home_dir() -> Result<PathBuf> {
    let home = env::var(home_env_var()).map_err(|e| {
        ShsError::Config(format!("couldn't determine home directory: {}", e))
    })?;
    Ok(PathBuf::from(home).join(".ssh"))
}

pub fn open_config() -> Result<File> {
    let path = home_dir()?.join("config");

    match File::open(&path) {
        Ok(file) => Ok(file),
        Err(why) if why.kind() == io::ErrorKind::NotFound => create_file(&path),
        Err(why) => Err(ShsError::Io(why)),
    }
}

pub fn get_hosts_all(file: File) -> Vec<String> {
    parse_hosts_config(io::BufReader::new(file))
}

pub(crate) fn parse_hosts_config<R: BufRead>(reader: R) -> Vec<String> {
    let mut confs = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Some(found) = line.find('#') {
            if found == 0 {
                continue;
            }
            let (line, _after) = line.split_at(found);
            confs.push(line.trim().to_string());
        } else {
            confs.push(line.trim().to_string());
        }
    }

    confs
}

pub fn hosts_sort(confs: Vec<String>) -> Vec<String> {
    let mut hosts: Vec<String> = Vec::new();
    let re = Regex::new(r"Host\s+(?P<host>\S+)").unwrap();
    for conf in confs {
        if let Some(caps) = re.captures(&conf) {
            hosts.push(caps["host"].to_string());
        }
    }

    hosts.sort_by(|a, b| {
        let a_is_digit = a.chars().next().map(|c| c.is_numeric()).unwrap_or(false);
        let b_is_digit = b.chars().next().map(|c| c.is_numeric()).unwrap_or(false);

        if a_is_digit && !b_is_digit {
            std::cmp::Ordering::Greater
        } else if !a_is_digit && b_is_digit {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    hosts
}

pub fn get_cmd_json(file: &str) -> Result<Value> {
    let path = home_dir()?.join(file);
    let data = match read_to_string(&path) {
        Ok(d) => d,
        Err(_) => {
            create_file(&path)?;
            let empty_json = Value::Object(Default::default());
            std::fs::write(&path, serde_json::to_string(&empty_json)?)?;
            return Ok(empty_json);
        }
    };

    serde_json::from_str(&data).map_err(|e| {
        ShsError::Config(format!(
            "{} is not valid JSON ({}). Please fix it manually or delete the file.",
            path.display(),
            e,
        ))
    })
}


pub fn find_pub_files(dir: &Path) -> Result<Vec<String>> {
    let mut pub_files = Vec::new();

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("pub") {
            pub_files.push(path.to_string_lossy().into_owned());
        }
    }

    Ok(pub_files)
}

pub fn push_pub_key(host_alias: &str, key_path: &Path) -> Result<()> {
    let key = key_path.to_string_lossy();
    let cmd = format!(
        "{cat} \"{key}\" | ssh {host} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"",
        cat = if cfg!(target_os = "windows") { "type" } else { "cat" },
        key = key,
        host = host_alias,
    );
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &cmd]).status()?
    } else {
        Command::new("sh").args(["-c", &cmd]).status()?
    };

    if status.success() {
        print_success("Public key pushed successfully");
    } else {
        print_error("Failed to push public key");
    }
    Ok(())
}

pub fn genrsa(email: &str) -> Result<()> {
    let cmd = format!("ssh-keygen -t rsa -b 4096 -C \"{}\"", email);

    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", &cmd]).status()?
    } else {
        Command::new("sh").args(["-c", &cmd]).status()?
    };

    if status.success() {
        print_success("RSA key generated successfully");
    } else {
        print_error("Failed to generate RSA key");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_hosts_config_drops_full_line_and_inline_comments() {
        let input = "\
# leading comment
Host alpha
HostName example.com
Host beta # inline note
HostName 1.2.3.4
";
        let confs = parse_hosts_config(Cursor::new(input));
        assert!(confs.contains(&"Host alpha".to_string()));
        assert!(confs.contains(&"Host beta".to_string()));
        assert!(confs.iter().all(|l| !l.starts_with('#')));
        assert!(confs.iter().all(|l| !l.contains("inline note")));
    }

    #[test]
    fn parse_hosts_config_trims_surrounding_whitespace() {
        let input = "   Host alpha   \n\tHostName example.com\n";
        let confs = parse_hosts_config(Cursor::new(input));
        assert!(confs.contains(&"Host alpha".to_string()));
        assert!(confs.contains(&"HostName example.com".to_string()));
    }

    #[test]
    fn hosts_sort_puts_numeric_after_alpha_and_alphabetises() {
        let input = vec![
            "Host beta".to_string(),
            "Host 10.0.0.1".to_string(),
            "Host alpha".to_string(),
        ];
        assert_eq!(hosts_sort(input), vec!["alpha", "beta", "10.0.0.1"]);
    }

    #[test]
    fn hosts_sort_ignores_lines_that_arent_host_definitions() {
        let input = vec![
            String::new(),
            "HostName example.com".to_string(),
            "Host gamma".to_string(),
        ];
        assert_eq!(hosts_sort(input), vec!["gamma"]);
    }

    #[test]
    fn hosts_sort_alpha_only_is_alphabetised() {
        let input = vec![
            "Host charlie".to_string(),
            "Host alpha".to_string(),
            "Host bravo".to_string(),
        ];
        assert_eq!(hosts_sort(input), vec!["alpha", "bravo", "charlie"]);
    }

    #[test]
    fn hosts_sort_numeric_only_keeps_relative_order_within_numeric_bucket() {
        let input = vec![
            "Host 2.2.2.2".to_string(),
            "Host 1.1.1.1".to_string(),
        ];
        assert_eq!(hosts_sort(input), vec!["1.1.1.1", "2.2.2.2"]);
    }
}
