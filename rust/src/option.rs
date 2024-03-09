use std::process::Command;
use std::fs::OpenOptions;
use std::io::Write;

use inquire::{Select, InquireError, Text};

use crate::utils;
use utils::{open_config, get_hosts, hosts_sort, home_dir};

fn get_cfg_edit() -> Vec<String>{
    if cfg!(target_os = "windows") {
        return vec!["notepad".to_string(), "code".to_string()];
    } else if cfg!(target_os = "macos") {
        return vec!["TextEdit".to_string(), "subl".to_string(), "atom".to_string(), "nano".to_string(), "vim".to_string(), "emacs".to_string(), "code".to_string()];
    } else if cfg!(target_os = "linux") {
        return vec!["nvim".to_string(), "emacs".to_string(), "nano".to_string(), "vim".to_string(), "subl".to_string(), "gedit".to_string(), "code".to_string()];
    } else {
        return vec!["nvim".to_string(), "emacs".to_string(), "nano".to_string(), "vim".to_string(), "subl".to_string(), "gedit".to_string(), "code".to_string()];
    }
}

fn edit() {
    let editor = get_cfg_edit();
    let selection = Select::new("Choose an editor", editor).prompt();

    match selection {
        Ok(selection) => {
            let editor: String;
            if selection == "TextEdit" {
                editor = "open -a TextEdit".to_string();
            } else {
                editor = selection;
            }
            println!("{}", home_dir());
            println!("{}", editor);
            let status = Command::new(editor)
                .arg(home_dir())
                .status();
                
            match status {
                Ok(status) => {
                    if status.success() {
                        println!("😙");
                    } else {
                        println!("oops, something went wrong🤣!");
                    }
                }
                Err(e) => println!("failed to execute process: {}", e),
            }
        }
        Err(_) => println!("You didn't select anything"),
        
    }

}

fn append_to_config(host: &str, hostname: &str, user: &str, port: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(home_dir())
        .unwrap();

    writeln!(file, "\n")?;
    writeln!(file, "Host {}", host)?;
    writeln!(file, "    HostName {}", hostname)?;
    writeln!(file, "    User {}", user)?;
    writeln!(file, "    Port {}", port)?;
    writeln!(file, "")?;

    Ok(())
}

fn add_host() {
    let error_deal = |_| {
        println!("oops, something went wrong!");
        std::process::exit(1);
    };
    let host = Text::new("Enter a domain name or IP address for SSH access:")
        .with_help_message("example: example.com or 111.111.11.111(public IP address)")
        .prompt()
        .unwrap_or_else(error_deal);

    let user = Text::new("Enter the username for SSH access:")
        .prompt()
        .unwrap_or_else(error_deal);

    let port = Text::new("Enter the port for SSH access:")
        .with_help_message("Default is 22")
        .with_default("22")
        .prompt()
        .unwrap_or_else(error_deal);

    let hostname = Text::new("Enter the hostname for SSH access:")
        .with_help_message("Default is the domain name or IP address")
        .with_default(&host.clone())
        .prompt()
        .unwrap_or_else(error_deal);

    if host.is_empty() || user.is_empty() || port.is_empty() || hostname.is_empty() {
        println!("You can't proceed without filling all the fields");
        std::process::exit(1);
    }

    let status = append_to_config(&host, &hostname, &user, &port);

    match status {
        Ok(_) => println!("Host added successfully"),
        Err(_) => println!("oops, something went wrong!"),
    }

}


fn connect() {
    let file = open_config();
    let confs = get_hosts(file);
    let hosts = hosts_sort(confs);

    let selection = Select::new("Choose a host", hosts).prompt();
    match selection {
        Ok(selection) => {
            let status = Command::new("ssh")
                .arg(selection)
                .status()
                .expect("failed to execute process");

            match status.success() {
                true => println!("😙"),
                false => println!("oops, something went wrong🤣!"),
            }
        }
        Err(_) => println!("You didn't select anything"),
    }
}

pub fn menu() {
    let options: Vec<&str> = vec!["Connect", "Add a new host", "Edit config", "Exit"];

    let ans: Result<&str, InquireError> = Select::new("Menu", options).prompt();

    match ans {
        Ok(choice) => {
            match choice {
                "Connect" => connect(),
                "Add a new host" => add_host(),
                "Edit config" => edit(),
                "Exit" => println!("😪"),
                _ => println!("Invalid choice"),
            }
        }
        Err(_) => println!("There was an error, please try again"),
    }
}