mod common;

use std::path::Path;

use common::{apply, assert_rejected_by, record, run, text, with};

fn plan_after(declaration: &str, path: &str) -> std::path::PathBuf {
    run(declaration, record(path)).plan().to_path_buf()
}

// @behavior LIFT-001
#[test]
fn should_lift_the_plan_that_many_folders() {
    assert_eq!(
        plan_after("{ lift = 2 }", "a/b/c/x.mkv"),
        Path::new("a/x.mkv")
    );
}

// @behavior LIFT-002
#[test]
fn should_refuse_lifting_above_the_source() {
    assert_rejected_by(apply("{ lift = 3 }", record("a/b/x.mkv")), "lift");
}

// @behavior LIFT-003
#[test]
fn should_lift_to_the_nearest_folder_that_matches() {
    let plan = plan_after(
        r#"{ lift = { to = "Season *" } }"#,
        "Series/Season 01/Rel/Subs/x.ass",
    );

    assert_eq!(plan, Path::new("Series/Season 01/x.ass"));
}

// @behavior LIFT-004
#[test]
fn should_leave_a_plan_already_in_a_matching_folder() {
    let plan = plan_after(
        r#"{ lift = { to = "Season *" } }"#,
        "Series/Season 01/x.mkv",
    );

    assert_eq!(plan, Path::new("Series/Season 01/x.mkv"));
}

// @behavior LIFT-005
#[test]
fn should_leave_a_plan_with_no_matching_folder() {
    let plan = plan_after(r#"{ lift = { to = "Season *" } }"#, "Series/Alpha/x.mkv");

    assert_eq!(plan, Path::new("Series/Alpha/x.mkv"));
}

// @behavior FLD-002
#[test]
fn should_refuse_a_folder_template_that_cannot_be_rendered() {
    assert_rejected_by(
        apply(r#"{ folder = "{missing}" }"#, record("Photos/x.jpg")),
        "folder",
    );
}

// @behavior FLD-003
#[test]
fn should_refuse_a_folder_named_with_dots_only() {
    let outcome = apply(
        r#"{ folder = "{show}" }"#,
        with(record("Photos/x.jpg"), "show", text("..")),
    );

    assert_rejected_by(outcome, "folder");
}

// @behavior FLD-004
#[test]
fn should_refuse_an_empty_folder_name() {
    let outcome = apply(
        r#"{ folder = "/{show}" }"#,
        with(record("Photos/x.jpg"), "show", text("Alpha")),
    );

    assert_rejected_by(outcome, "folder");
}
