#![cfg(target_os = "linux")]

mod common;

use common::{Program, Sandbox};

// @behavior RUN-006
#[test]
fn should_stop_cleanly_on_a_termination_signal() {
    let sandbox = Sandbox::new();
    let mut program = Program::start(&sandbox, r#"["move"]"#, "");

    program.signal("TERM");

    assert!(program.exits_successfully());
}

// @behavior RUN-007
#[test]
fn should_stop_cleanly_on_an_interrupt() {
    let sandbox = Sandbox::new();
    let mut program = Program::start(&sandbox, r#"["move"]"#, "");

    program.signal("INT");

    assert!(program.exits_successfully());
}
