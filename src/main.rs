use std::process::ExitCode;

use auto_renamer::{Command, parse_args};

const HELP: &str = "Usage: auto-renamer [--config <path>]\n\n  --config <path>  the configuration file (default /etc/auto-renamer/config.toml)\n  -h, --help       show this help\n  -V, --version    show the version";

fn main() -> ExitCode {
    match parse_args(std::env::args().skip(1)) {
        Ok(Command::Help) => {
            println!("{HELP}");
            ExitCode::SUCCESS
        }
        Ok(Command::Version) => {
            println!("auto-renamer {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Command::Run(options)) => start(&options),
        Err(message) => {
            eprintln!("{message}\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

#[cfg(target_os = "linux")]
fn start(options: &auto_renamer::Options) -> ExitCode {
    let stop = std::sync::atomic::AtomicBool::new(false);
    match auto_renamer::runner::run(options, &stop) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn start(_options: &auto_renamer::Options) -> ExitCode {
    eprintln!("auto-renamer only supports Linux");
    ExitCode::from(1)
}
