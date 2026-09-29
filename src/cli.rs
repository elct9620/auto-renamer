//! The command line, which says where the configuration is and nothing more.

use std::path::PathBuf;

const DEFAULT_CONFIG: &str = "/etc/auto-renamer/config.toml";

/// The settings of a run that come from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub config: PathBuf,
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run(Options),
    Help,
    Version,
}

/// Reads the command line, refusing what it does not know with a message that names it.
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut config = PathBuf::from(DEFAULT_CONFIG);
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-V" | "--version" => return Ok(Command::Version),
            "--config" => {
                config = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or("`--config` needs a path")?;
            }
            _ => match arg.strip_prefix("--config=") {
                Some(path) if !path.is_empty() => config = PathBuf::from(path),
                _ => return Err(format!("`{arg}` is not something it can do")),
            },
        }
    }
    Ok(Command::Run(Options { config }))
}
