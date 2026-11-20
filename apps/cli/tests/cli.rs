//! Runs the compiled `cli` binary as a real process.

// Helpers here are test code too: a panic is how a test fails.
#![allow(clippy::unwrap_used)]

use std::process::{Command, Output};

/// Runs the `cli` binary with the given arguments.
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cli"))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn check_email_accepts_and_normalizes() {
    let output = run(&["check-email", "  Jane@Example.COM "]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), "valid email: jane@example.com\n");
}

#[test]
fn check_email_rejects_invalid_input_as_usage_error() {
    let output = run(&["check-email", "jane@"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("email must look like name@example.com"));
}

#[test]
fn check_file_reports_missing_file() {
    let output = run(&["check-file", "this/file/does/not/exist.txt"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("failed to read this/file/does/not/exist.txt"));
}
