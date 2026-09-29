use std::path::PathBuf;

use auto_renamer::{Command, Options, parse_args};

fn parse(args: &[&str]) -> Result<Command, String> {
    parse_args(args.iter().map(|arg| arg.to_string()))
}

fn run_with(config: &str) -> Result<Command, String> {
    Ok(Command::Run(Options {
        config: PathBuf::from(config),
    }))
}

// @behavior CLI-001
#[test]
fn should_read_the_configuration_from_a_default_place() {
    assert_eq!(parse(&[]), run_with("/etc/auto-renamer/config.toml"));
}

// @behavior CLI-002
#[test]
fn should_take_a_path_after_the_flag() {
    assert_eq!(parse(&["--config", "/tmp/a.toml"]), run_with("/tmp/a.toml"));
}

// @behavior CLI-003
#[test]
fn should_take_a_path_after_an_equals_sign() {
    assert_eq!(parse(&["--config=/tmp/a.toml"]), run_with("/tmp/a.toml"));
}

// @behavior CLI-004
#[test]
fn should_refuse_a_flag_without_a_path() {
    assert!(parse(&["--config"]).is_err());
}

// @behavior CLI-005
#[test]
fn should_refuse_an_unknown_argument_by_name() {
    let message = parse(&["--colour"]).unwrap_err();

    assert!(message.contains("--colour"), "{message}");
}

// @behavior CLI-006
#[test]
fn should_show_help_when_asked() {
    assert_eq!(parse(&["--help"]), Ok(Command::Help));
}

// @behavior CLI-007
#[test]
fn should_show_the_version_when_asked() {
    assert_eq!(parse(&["--version"]), Ok(Command::Version));
}
