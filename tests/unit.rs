use std::path::{Path, PathBuf};

use auto_renamer::Config;

/// The unit of a file under the unit setting written in a watch.
fn unit_of(setting: &str, file: &str) -> PathBuf {
    let text = format!("[watch.w]\nsource = \"/s\"\nunit = {setting}\n");
    let config = Config::parse(&text).expect("the configuration should be accepted");
    config.watches()[0].unit.of(Path::new(file))
}

// @behavior UNT-001
#[test]
fn should_take_the_folder_of_the_file_by_default() {
    assert_eq!(
        unit_of("\"directory\"", "Movies/A/x.mkv"),
        Path::new("Movies/A")
    );
}

// @behavior UNT-002
#[test]
fn should_take_the_source_itself_as_one_unit() {
    assert_eq!(unit_of("\"source\"", "Movies/A/x.mkv"), Path::new(""));
}

// @behavior UNT-003
#[test]
fn should_take_a_folder_matching_a_root_pattern_as_a_unit() {
    assert_eq!(
        unit_of(r#"{ root = ["Movies/*"] }"#, "Movies/A/x.mkv"),
        Path::new("Movies/A")
    );
}

// @behavior UNT-004
#[test]
fn should_give_the_files_below_a_matching_folder_to_it() {
    assert_eq!(
        unit_of(r#"{ root = ["Movies/*"] }"#, "Movies/A/Subs/x.ass"),
        Path::new("Movies/A")
    );
}

// @behavior UNT-005
#[test]
fn should_fall_back_to_the_folder_when_no_root_matches() {
    assert_eq!(
        unit_of(r#"{ root = ["Movies/*"] }"#, "Music/B/x.mp3"),
        Path::new("Music/B")
    );
}

// @behavior UNT-006
#[test]
fn should_not_let_a_star_reach_across_a_slash() {
    assert_eq!(
        unit_of(r#"{ root = ["*"] }"#, "Movies/A/x.mkv"),
        Path::new("Movies")
    );
}

// @behavior UNT-007
#[test]
fn should_take_the_shallowest_matching_folder() {
    assert_eq!(
        unit_of(
            r#"{ root = ["Movies/*", "Movies/*/*"] }"#,
            "Movies/A/B/x.mkv"
        ),
        Path::new("Movies/A")
    );
}

// @behavior UNT-008
#[test]
fn should_fall_back_to_the_root_for_a_file_at_the_source_root() {
    assert_eq!(
        unit_of(r#"{ root = ["Movies/*"] }"#, "x.mkv"),
        Path::new("")
    );
}

#[test]
fn should_refuse_a_root_pattern_that_is_not_a_pattern() {
    let text = "[watch.w]\nsource = \"/s\"\nunit = { root = [\"[\"] }\n";

    assert!(Config::parse(text).is_err());
}
