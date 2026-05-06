mod error;
mod hiiro;
mod option;
mod utils;

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use inquire::InquireError;

use error::{Result, ShsError};
use hiiro::hello_hiiro;
use option::{add_host, connect, gen_key, list_hosts, menu, push_key};

/// Interactive SSH host helper that drives ~/.ssh/config and ~/.ssh/precommand.
///
/// Run with no arguments for the interactive menu, or use a subcommand for
/// scriptable, non-interactive operation. As a shorthand, `shs <host>`
/// connects to that host directly.
#[derive(Parser)]
#[command(name = "shs", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Connect to a host (interactive picker if no <host> is given).
    Connect {
        /// Host alias from ssh_config.
        host: Option<String>,
    },
    /// List host aliases parsed from ~/.ssh/config (and Include'd files).
    Ls,
    /// Add a new host entry to ~/.ssh/config (interactive).
    Add,
    /// Push a public key from ~/.ssh to a host's authorized_keys (interactive).
    PushKey,
    /// Generate a new 4096-bit RSA key via ssh-keygen.
    GenKey {
        /// Email comment for the generated key.
        #[arg(short, long)]
        email: Option<String>,
    },
    /// Catches `shs <host>` and connects directly.
    #[command(external_subcommand)]
    External(Vec<String>),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.command.is_none() {
        hello_hiiro();
    }

    let result: Result<()> = match cli.command {
        None => menu(),
        Some(Command::Connect { host }) => connect(host.as_deref()),
        Some(Command::Ls) => list_hosts(),
        Some(Command::Add) => add_host(),
        Some(Command::PushKey) => push_key(),
        Some(Command::GenKey { email }) => gen_key(email),
        Some(Command::External(args)) => {
            let host = args.into_iter().next().unwrap_or_default();
            connect(Some(&host))
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(ShsError::Inquire(
            InquireError::OperationCanceled | InquireError::OperationInterrupted,
        )) => {
            println!("Cancelled");
            ExitCode::from(130)
        }
        Err(e) => {
            utils::print_error(&format!("{}", e));
            ExitCode::FAILURE
        }
    }
}
