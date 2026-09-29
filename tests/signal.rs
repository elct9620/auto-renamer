#![cfg(target_os = "linux")]

mod common;

use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use common::Sandbox;

/// The program running over a sandbox, killed if the test ends before it did.
struct Program {
    child: Child,
}

impl Program {
    fn start(sandbox: &Sandbox) -> Program {
        sandbox.make_dir("source");
        sandbox.make_dir("target");
        sandbox.write(
            "config.toml",
            &format!(
                "[pipeline.p]\nstages = [\"move\"]\n\n[watch.w]\nsource = \"{}\"\ntarget = \"{}\"\npipelines = [\"p\"]\n",
                sandbox.path("source").display(),
                sandbox.path("target").display(),
            ),
        );
        let child = Command::new(env!("CARGO_BIN_EXE_auto-renamer"))
            .arg("--config")
            .arg(sandbox.path("config.toml"))
            .stderr(Stdio::null())
            .spawn()
            .expect("the program should start");
        thread::sleep(Duration::from_millis(1500));
        Program { child }
    }

    fn signal(&self, name: &str) {
        let status = Command::new("kill")
            .arg(format!("-{name}"))
            .arg(self.child.id().to_string())
            .status()
            .expect("kill should run");
        assert!(status.success());
    }

    /// Whether the program exited successfully within five seconds.
    fn exits_successfully(&mut self) -> bool {
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if let Some(status) = self
                .child
                .try_wait()
                .expect("the program should be waited on")
            {
                return status.success();
            }
            thread::sleep(Duration::from_millis(100));
        }
        false
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// @behavior RUN-006
#[test]
fn should_stop_cleanly_on_a_termination_signal() {
    let sandbox = Sandbox::new();
    let mut program = Program::start(&sandbox);

    program.signal("TERM");

    assert!(program.exits_successfully());
}

// @behavior RUN-007
#[test]
fn should_stop_cleanly_on_an_interrupt() {
    let sandbox = Sandbox::new();
    let mut program = Program::start(&sandbox);

    program.signal("INT");

    assert!(program.exits_successfully());
}
