use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

use inquire::validator::Validation;
use inquire::{CustomUserError, Select, Text};

use crate::error::{Result, ShsError};
use crate::utils::{
    genrsa, get_cmd_json, get_hosts_all, home_dir, hosts_sort, open_config,
    print_error, print_success,
};

fn add_precommand() -> Result<()> {
    let hosts = get_hosts()?;
    if hosts.is_empty() {
        return Err(ShsError::Config(
            "You don't have any hosts to connect to".into(),
        ));
    }
    let selection = Select::new("Choose a host", hosts).prompt()?;

    let command = Text::new("Enter a command to execute before connecting to the host:")
        .with_help_message(
            "If the command is too long or includes ESC, please add it through Edit precommand",
        )
        .prompt()?;

    let mut precommand = get_cmd_json("precommand")?;
    if precommand[&selection].is_null() {
        precommand[&selection] = serde_json::json!(vec![&command]);
    } else if let Some(arr) = precommand[&selection].as_array_mut() {
        arr.push(serde_json::json!(command));
    }
    let data = serde_json::to_string_pretty(&precommand)?;
    let path = home_dir()?.join("precommand");
    std::fs::write(&path, data)?;
    print_success("Command added successfully");
    Ok(())
}

fn execute_precommand() -> Result<()> {
    let precommand = get_cmd_json("precommand")?;
    if precommand.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        return Err(ShsError::Config("No precommand found".into()));
    }

    let hosts = get_hosts()?;
    if hosts.is_empty() {
        return Err(ShsError::Config(
            "You don't have any hosts to connect to".into(),
        ));
    }
    let selection = Select::new("Choose a host", hosts).prompt()?;

    if precommand[&selection].is_null() {
        return Err(ShsError::Config(format!(
            "No precommand found for {}",
            selection,
        )));
    }

    let commands: Vec<String> = precommand[&selection]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let command = Select::new("Choose a command", commands).prompt()?;

    print_error(&format!("Now execute command: ssh {} {}", &selection, &command));

    let status = Command::new("ssh").arg(&selection).arg(&command).status()?;

    if status.success() {
        println!("😙");
    } else {
        println!("\x1b[31moops, something went wrong🤣!\x1b[0m");
    }
    Ok(())
}

fn get_cfg_edit() -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec!["notepad".into(), "code".into()]
    } else if cfg!(target_os = "macos") {
        vec![
            "TextEdit".into(),
            "subl".into(),
            "atom".into(),
            "nano".into(),
            "vim".into(),
            "emacs".into(),
            "code".into(),
        ]
    } else {
        vec![
            "nvim".into(),
            "emacs".into(),
            "nano".into(),
            "vim".into(),
            "subl".into(),
            "gedit".into(),
            "code".into(),
        ]
    }
}

fn edit(path: PathBuf) -> Result<()> {
    let editors = get_cfg_edit();
    let selection = Select::new("Choose an editor", editors).prompt()?;
    let editor = if selection == "TextEdit" {
        "open -a TextEdit".to_string()
    } else {
        selection
    };
    println!("Opening {}...", editor);
    let status = Command::new(editor).arg(&path).status()?;
    if status.success() {
        println!("😙");
    } else {
        println!("oops, something went wrong🤣!");
    }
    Ok(())
}

fn append_to_config(host: &str, hostname: &str, user: &str, port: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(home_dir()?.join("config"))?;

    writeln!(file)?;
    writeln!(file, "Host {}", host)?;
    writeln!(file, "HostName {}", hostname)?;
    writeln!(file, "User {}", user)?;
    writeln!(file, "Port {}", port)?;
    Ok(())
}

fn validate_no_whitespace(input: &str) -> std::result::Result<Validation, CustomUserError> {
    if input.is_empty() {
        return Ok(Validation::Invalid("must not be empty".into()));
    }
    if input.chars().any(char::is_whitespace) {
        return Ok(Validation::Invalid("must not contain whitespace".into()));
    }
    if input.contains('#') {
        return Ok(Validation::Invalid(
            "'#' starts a comment in ssh_config".into(),
        ));
    }
    Ok(Validation::Valid)
}

fn validate_port(input: &str) -> std::result::Result<Validation, CustomUserError> {
    match input.parse::<u16>() {
        Ok(n) if n > 0 => Ok(Validation::Valid),
        _ => Ok(Validation::Invalid(
            "port must be a number between 1 and 65535".into(),
        )),
    }
}

fn add_host() -> Result<()> {
    let host = Text::new("Enter an alias for SSH access:")
        .with_help_message("ssh <alias> will use this entry; cannot contain whitespace or '#'")
        .with_validator(validate_no_whitespace)
        .prompt()?;

    let user = Text::new("Enter the username for SSH access:")
        .with_default("root")
        .with_validator(validate_no_whitespace)
        .prompt()?;

    let port = Text::new("Enter the port for SSH access:")
        .with_help_message("Default is 22")
        .with_default("22")
        .with_validator(validate_port)
        .prompt()?;

    let hostname = Text::new("Enter the hostname for SSH access:")
        .with_help_message("example: example.com or 111.111.11.111(public IP address)")
        .with_default(&host.clone())
        .with_validator(validate_no_whitespace)
        .prompt()?;

    append_to_config(&host, &hostname, &user, &port)?;
    println!("Host added successfully");
    let push_cmd = if cfg!(target_os = "windows") {
        format!(
            "type %USERPROFILE%\\.ssh\\id_rsa.pub | ssh {}@{} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"",
            user, hostname,
        )
    } else {
        format!(
            "cat ~/.ssh/id_rsa.pub | ssh {}@{} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"",
            user, hostname,
        )
    };
    println!(
        "Execute the follow command to push secret key to the server\n \x1b[31m{}\x1b[0m",
        push_cmd,
    );
    Ok(())
}

fn get_hosts() -> Result<Vec<String>> {
    let file = open_config()?;
    Ok(hosts_sort(get_hosts_all(file)))
}

fn connect() -> Result<()> {
    let hosts = get_hosts()?;
    if hosts.is_empty() {
        return Err(ShsError::Config(
            "You don't have any hosts to connect to".into(),
        ));
    }
    let selection = Select::new("Choose a host", hosts).prompt()?;
    let status = Command::new("ssh").arg(&selection).status()?;
    if status.success() {
        println!("😙");
    } else {
        println!("\x1b[31moops, something went wrong🤣!\x1b[0m");
    }
    Ok(())
}

pub fn menu() -> Result<()> {
    let options: Vec<&str> = vec![
        "Connect",
        "Execute precommand",
        "Add a new host",
        "Add a new precommand",
        "Edit config",
        "Edit precommand",
        "Generate RSA key",
        "Exit",
    ];

    let choice = Select::new("Menu", options).prompt()?;
    match choice {
        "Connect" => connect(),
        "Execute precommand" => execute_precommand(),
        "Add a new host" => add_host(),
        "Add a new precommand" => add_precommand(),
        "Edit config" => edit(home_dir()?.join("config")),
        "Edit precommand" => edit(home_dir()?.join("precommand")),
        "Generate RSA key" => {
            let email = Text::new("Enter your email:").prompt()?;
            genrsa(&email)
        }
        "Exit" => {
            println!("😪");
            Ok(())
        }
        other => Err(ShsError::Config(format!("Invalid choice: {}", other))),
    }
}

