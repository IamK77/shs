use std::process::{self, exit, Command};
use std::fs::OpenOptions;
use std::io::Write;

use inquire::{Select, InquireError, Text};
use which;

use crate::utils;
use utils::{open_config, 
    get_hosts_all, 
    hosts_sort, 
    home_dir, 
    get_cmd_json, 
    print_success, 
    print_error,
    genrsa,
    AppError
};

fn select_host(prompt: &str) -> Result<String, AppError> {
    let hosts = get_hosts();
    if hosts.is_empty() {
        return Err(AppError::ConfigError("No hosts available".into()));
    }
    
    let selection = Select::new(prompt, hosts).prompt()?;
    Ok(selection)
}

fn add_precommand() {
    let selection = match select_host("Choose a host") {
        Ok(selection) => selection,
        Err(AppError::ConfigError(msg)) => {
            print_error(&msg);
            exit(1);
        }
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };

    let command = Text::new("Enter a command to execute before connecting to the host:")
        .with_help_message("If the command is too long or include ESC, please add it through Edit precommand")
        .prompt()
        .unwrap_or_else(|_| {
            println!("You can't proceed without filling all the fields");
            exit(1);
        });

    let mut precommand = get_cmd_json("precommand").unwrap_or_else(|e| {
        println!("Error reading precommand file: {}", e);
        exit(1);
    });
    
    if precommand[&selection].is_null() {
        precommand[&selection] = serde_json::json!(vec![&command]);
    } else {
        if let Some(arr) = precommand[&selection].as_array_mut() {
            arr.push(serde_json::json!(command));
        }
    }
    
    let data = serde_json::to_string_pretty(&precommand).unwrap_or_else(|e| {
        println!("Error serializing JSON: {}", e);
        exit(1);
    });
    
    let mut precommand_path = home_dir();
    precommand_path.push("precommand");
    
    std::fs::write(&precommand_path, data).unwrap_or_else(|e| {
        println!("Error writing file: {}", e);
        exit(1);
    });
    
    print_success("Command added successfully");
}

fn execute_precommand() {
    let precommand = get_cmd_json("precommand").unwrap_or_else(|e| {
        println!("Error reading precommand file: {}", e);
        exit(1);
    });
    
    if let Some(obj) = precommand.as_object() {
        if obj.is_empty() {
            println!("No precommand found");
            exit(1);
        }
    }
    
    let selection = match select_host("Choose a host") {
        Ok(selection) => selection,
        Err(AppError::ConfigError(msg)) => {
            print_error(&msg);
            exit(1);
        }
        Err(e) => {
            println!("Error: {}", e);
            exit(1);
        }
    };

    if precommand[&selection].is_null() {
        print_error(&format!("No precommand found for {}", &selection));
        exit(1);
    }

    let commands: Vec<String> = precommand
        .get(&selection)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str()
        .unwrap()
        .to_string())
        .collect();

    let command = Select::new("Choose a command", commands).prompt();
    let command = match command {
        Ok(command) => command,
        Err(_) => {
            println!("You didn't select anything");
            exit(1);
        },
    };

    print_error(&format!("Now execute command: ssh {} {}", &selection, &command));

    let status = Command::new("ssh")
        .arg(&selection)
        .arg(&command)
        .status()
        .unwrap_or_else(|e| {
            println!("Failed to execute SSH command: {}", e);
            exit(1);
        });

    match status.success() {
        true => println!("😙"),
        false => println!("\x1b[31moops, something went wrong🤣!\x1b[31m"),
    }
}

fn find_editor() -> Result<String, AppError> {
    let preferred_editors = if cfg!(target_os = "windows") {
        vec!["code", "notepad"]
    } else {
        vec!["code", "nvim", "vim", "nano", "emacs", "subl", "gedit"]
    };

    for editor in preferred_editors {
        if which::which(editor).is_ok() {
            return Ok(editor.to_string());
        }
    }

    Err(AppError::ConfigError("No suitable editor found".into()))
}

fn edit(path: String) {
    match find_editor() {
        Ok(editor) => {
            println!("Opening {}...", editor);
            let status = if editor == "TextEdit" {
                Command::new("open")
                    .args(["-a", "TextEdit", &path])
                    .status()
            } else {
                Command::new(&editor)
                    .arg(&path)
                    .status()
            };
            
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
        Err(e) => {
            println!("Error finding editor: {}", e);
            println!("Please install a text editor like VSCode, Vim, or Nano");
        }
    }
}

fn append_to_config(host: &str, hostname: &str, user: &str, port: &str) -> std::io::Result<()> {
    let mut config_path = home_dir();
    config_path.push("config");
    
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(config_path)
        .unwrap_or_else(|_| {
            println!("Unable to open file");
            exit(1);
        });

    match file.metadata() {
        Ok(_) => {
            writeln!(file, "\n")?;
            writeln!(file, "Host {}", host)?;
            writeln!(file, "HostName {}", hostname)?;
            writeln!(file, "User {}", user)?;
            writeln!(file, "Port {}", port)?;
        }
        Err(_) => {
            eprintln!("Unable to get metadata");
        }
    }
    Ok(())
}



fn validate_hostname(hostname: &str) -> Result<(), AppError> {
    if hostname.is_empty() {
        return Err(AppError::ValidationError("Hostname cannot be empty".into()));
    }
    
    if hostname.len() > 253 {
        return Err(AppError::ValidationError("Hostname too long".into()));
    }
    
    if hostname.contains(' ') {
        return Err(AppError::ValidationError("Hostname cannot contain spaces".into()));
    }
    
    Ok(())
}

fn validate_port(port: &str) -> Result<u16, AppError> {
    let port_num: u16 = port.parse().map_err(|_| {
        AppError::ValidationError("Port must be a number between 1 and 65535".into())
    })?;
    
    if port_num == 0 {
        return Err(AppError::ValidationError("Port cannot be 0".into()));
    }
    
    Ok(port_num)
}

fn add_host() {
    let error_deal = |which| {
        move |e: inquire::InquireError| {
            println!("oops, {} something went wrong: {}", which, e);
            std::process::exit(1);
        }
    };
    
    let host = Text::new("Enter a domain name or IP address for SSH access:")
        .with_help_message("Default is the domain name or IP address")
        .prompt()
        .unwrap_or_else(error_deal("host"));

    let user = Text::new("Enter the username for SSH access:")
        .with_default("root")
        .prompt()
        .unwrap_or_else(error_deal("user"));

    let port_input = Text::new("Enter the port for SSH access:")
        .with_help_message("Default is 22")
        .with_default("22")
        .prompt()
        .unwrap_or_else(error_deal("port"));

    let hostname = Text::new("Enter the hostname for SSH access:")
        .with_help_message("example: example.com or 111.111.11.111(public IP address)")
        .with_default(&host.clone())
        .prompt()
        .unwrap_or_else(error_deal("hostname"));

    if host.is_empty() || user.is_empty() || port_input.is_empty() || hostname.is_empty() {
        println!("You can't proceed without filling all the fields");
        std::process::exit(1);
    }

    if let Err(e) = validate_hostname(&host) {
        println!("Invalid host: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = validate_hostname(&hostname) {
        println!("Invalid hostname: {}", e);
        std::process::exit(1);
    }

    let port = match validate_port(&port_input) {
        Ok(port) => port.to_string(),
        Err(e) => {
            println!("Invalid port: {}", e);
            std::process::exit(1);
        }
    };

    // push_s_key(&user, &hostname, &port, "id_rsa");

    let status = append_to_config(&host, &hostname, &user, &port);

    match status {
        Ok(_) => {
            println!("Host added successfully");
            println!("Execute the follow commend to push secret key to the server\n \x1b[31mtype %USERPROFILE%\\.ssh\\id_rsa.pub | ssh {}@{} \"mkdir -p ~/.ssh && cat >> ~/.ssh/authorized_keys\"\x1b[31m", user, hostname);
        },
        Err(e) => {
            println!("oops, something went wrong🤣!");
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    }

}

fn get_hosts() -> Vec<String> {
    let file = open_config();
    let confs = get_hosts_all(file);
    let hosts = hosts_sort(confs);

    hosts
}


fn connect() {
    let hosts = get_hosts();

    let selection = Select::new("Choose a host", hosts.clone()).prompt();
    match selection {
        Ok(selection) => {
            let status = Command::new("ssh")
                .arg(selection)
                .status()
                .expect("failed to execute process");

            match status.success() {
                true => println!("😙"),
                false => println!("\x1b[31moops, something went wrong🤣!\x1b[31m"),
            }
        }
        Err(_) => {
            if hosts.is_empty() {
                println!("\x1b[31mYou don't have any hosts to connect to\x1b[31m");
            } else {
                println!("You didn't select anything");
            }
        },
    }
}

pub fn menu() {
    let options: Vec<&str> = vec!["Connect", "Execute precommand", "Add a new host", "Add a new precommand","Edit config", "Edit precommand", "Generate RSA key", "Exit"];

    let ans: Result<&str, InquireError> = Select::new("Menu", options).prompt();

    match ans {
        Ok(choice) => {
            match choice {
                "Connect" => connect(),
                "Execute precommand" => execute_precommand(),
                "Add a new host" => add_host(),
                "Add a new precommand" => add_precommand(),
                "Edit config" => {
                    let mut config_path = home_dir();
                    config_path.push("config");
                    edit(config_path.to_string_lossy().to_string())
                },
                "Edit precommand" => {
                    let mut precommand_path = home_dir();
                    precommand_path.push("precommand");
                    edit(precommand_path.to_string_lossy().to_string())
                },
                "Generate RSA key" => {
                    let email = Text::new("Enter your email:")
                        .prompt()
                        .unwrap_or_else(|_| {
                            println!("oops, something went wrong🤣!");
                            process::exit(0);
                        });
                    genrsa(&email);
                },
                "Exit" => println!("😪"),
                _ => println!("Invalid choice"),
            }
        }
        Err(_) => println!("There was an error, please try again"),
    }
}