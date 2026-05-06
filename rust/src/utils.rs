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

pub fn open_config() -> Result<PathBuf> {
    let path = home_dir()?.join("config");

    match File::open(&path) {
        Ok(_) => Ok(path),
        Err(why) if why.kind() == io::ErrorKind::NotFound => {
            create_file(&path)?;
            Ok(path)
        }
        Err(why) => Err(ShsError::Io(why)),
    }
}

pub fn get_hosts_all(path: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut visited = Vec::new();
    walk_config(path, &mut out, &mut visited);
    out
}

fn walk_config(path: &Path, out: &mut Vec<String>, visited: &mut Vec<PathBuf>) {
    // canonicalize is also our existence/permission gate; ssh's own behaviour
    // is to silently skip Includes that don't exist.
    let canonical = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return,
    };
    if visited.contains(&canonical) {
        return;
    }
    visited.push(canonical);

    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let reader = io::BufReader::new(file);
    let parent = path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    for line in reader.lines() {
        let raw = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let stripped = strip_comment(&raw).trim().to_string();
        if stripped.is_empty() {
            continue;
        }

        if let Some(rest) = strip_keyword(&stripped, "Include") {
            for spec in rest.split_whitespace() {
                for resolved in expand_include(spec, &parent) {
                    walk_config(&resolved, out, visited);
                }
            }
            continue;
        }

        out.push(stripped);
    }
}

fn strip_comment(line: &str) -> &str {
    match line.find('#') {
        Some(0) => "",
        Some(idx) => &line[..idx],
        None => line,
    }
}

/// Returns Some(rest) if `line` (after leading whitespace) starts with
/// `keyword` (ASCII case-insensitive) followed by whitespace.
fn strip_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let trimmed = line.trim_start();
    let kw_len = keyword.len();
    if trimmed.len() <= kw_len {
        return None;
    }
    if !trimmed[..kw_len].eq_ignore_ascii_case(keyword) {
        return None;
    }
    let rest = &trimmed[kw_len..];
    if !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    Some(rest.trim_start())
}

/// Expand an `Include` operand into one or more concrete paths. Supports
/// `~/...`, absolute paths, paths relative to the config's directory, and
/// shell-style glob wildcards (`*`, `?`, `[...]`).
fn expand_include(spec: &str, parent_dir: &Path) -> Vec<PathBuf> {
    let expanded: PathBuf = if let Some(rest) = spec.strip_prefix("~/") {
        match env::var(home_env_var()) {
            Ok(h) => PathBuf::from(h).join(rest),
            Err(_) => return Vec::new(),
        }
    } else if Path::new(spec).is_absolute() {
        PathBuf::from(spec)
    } else {
        parent_dir.join(spec)
    };

    let pattern = expanded.to_string_lossy();
    match glob::glob(&pattern) {
        Ok(paths) => paths.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
pub(crate) fn parse_hosts_config<R: BufRead>(reader: R) -> Vec<String> {
    let mut confs = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let stripped = strip_comment(&line).trim();
        if stripped.is_empty() {
            continue;
        }
        confs.push(stripped.to_string());
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

    #[test]
    fn strip_keyword_matches_case_insensitively_with_whitespace_separator() {
        assert_eq!(strip_keyword("Include foo bar", "Include"), Some("foo bar"));
        assert_eq!(strip_keyword("INCLUDE\tfoo", "Include"), Some("foo"));
        assert_eq!(strip_keyword("  include   foo", "Include"), Some("foo"));
    }

    #[test]
    fn strip_keyword_rejects_no_whitespace_or_unrelated_lines() {
        // "Includepath" should not match "Include"
        assert_eq!(strip_keyword("Includepath", "Include"), None);
        // Just the keyword with no operand
        assert_eq!(strip_keyword("Include", "Include"), None);
        // Different keyword
        assert_eq!(strip_keyword("Host alpha", "Include"), None);
    }

    #[test]
    fn strip_comment_handles_leading_and_inline_comments() {
        assert_eq!(strip_comment("# whole line"), "");
        assert_eq!(strip_comment("Host x # inline"), "Host x ");
        assert_eq!(strip_comment("Host x"), "Host x");
    }

    #[test]
    fn get_hosts_all_follows_include_directives() {
        use std::fs;
        use std::io::Write as _;

        let dir = std::env::temp_dir().join(format!("shs-include-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        let included = dir.join("extra.conf");
        let mut f = fs::File::create(&included).unwrap();
        writeln!(f, "Host beta").unwrap();
        writeln!(f, "HostName 10.0.0.2").unwrap();

        let main = dir.join("config");
        let mut f = fs::File::create(&main).unwrap();
        writeln!(f, "Host alpha").unwrap();
        writeln!(f, "HostName 10.0.0.1").unwrap();
        writeln!(f, "Include {}", included.display()).unwrap();

        let confs = get_hosts_all(&main);
        assert!(confs.iter().any(|l| l == "Host alpha"));
        assert!(
            confs.iter().any(|l| l == "Host beta"),
            "expected Include to pull in Host beta, got {:?}",
            confs,
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
