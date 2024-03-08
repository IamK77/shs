use std::process::Command;

use inquire::{Select, InquireError};

use crate::utils;
use utils::{open_config, get_hosts, hosts_sort};

pub fn connect() {
    let file = open_config();
    let confs = get_hosts(file);
    let hosts = hosts_sort(confs);

    let selection = Select::new("Choose a host", hosts).prompt();
    match selection {
        Ok(selection) => {
            // let ssh = format!("ssh {}", selection);
            // println!("{}", ssh);
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
    let options: Vec<&str> = vec!["Connect", "Add a new host", "Exit"];

    let ans: Result<&str, InquireError> = Select::new("Menu", options).prompt();

    match ans {
        Ok(choice) => {
            match choice {
                "Connect" => connect(),
                "Add a new host" => println!("wait a update"),
                "Exit" => println!("😪"),
                _ => println!("Invalid choice"),
            }
        }
        Err(_) => println!("There was an error, please try again"),
    }
}