use std::process::ExitCode;

use domain::model::Email;

fn main() -> ExitCode {
    let Some(input) = std::env::args().nth(1) else {
        eprintln!("usage: cli <email>");
        return ExitCode::FAILURE;
    };

    match Email::parse(&input) {
        Ok(email) => {
            println!("valid email: {email}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("invalid email: {err}");
            ExitCode::FAILURE
        }
    }
}
