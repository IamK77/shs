mod error;
mod hiiro;
mod option;
mod utils;

use std::process::ExitCode;

use inquire::InquireError;

use error::ShsError;
use hiiro::hello_hiiro;
use option::menu;

fn main() -> ExitCode {
    hello_hiiro();
    match menu() {
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
