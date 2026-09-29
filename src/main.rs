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
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    // As process 1 of a container, a signal without a handler is ignored, so `docker stop` would wait to kill it.
    for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
        if let Err(error) = signal_hook::flag::register(signal, std::sync::Arc::clone(&stop)) {
            eprintln!("could not handle signal {signal}: {error}");
            return ExitCode::from(2);
        }
    }
    let reload = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    if let Err(error) =
        signal_hook::flag::register(signal_hook::consts::SIGHUP, std::sync::Arc::clone(&reload))
    {
        eprintln!("could not handle the hangup signal: {error}");
        return ExitCode::from(2);
    }
    match auto_renamer::runner::run(options, &stop, &reload) {
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
