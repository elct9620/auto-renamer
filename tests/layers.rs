mod common;

use auto_renamer::{Config, ConfigError, FolderConfig, Value, Watch};
use common::names;

const BASE: &str = r#"
[pipeline.video]
stages = [{ filter = { ext = ["mkv"] } }, "move"]

[pipeline.photo]
stages = ["move"]

[watch.series]
source = "/downloads"
pipelines = ["video", "photo"]
vars = { show = "Alpha" }
max_files = 100
"#;

fn watch() -> Watch {
    Config::parse(BASE)
        .expect("the configuration should be accepted")
        .watches()[0]
        .clone()
}

fn folder(text: &str) -> FolderConfig {
    FolderConfig::parse(text).expect("the folder configuration should be accepted")
}

fn refused(text: &str) -> ConfigError {
    FolderConfig::parse(text).expect_err("the folder configuration should be refused")
}

fn show(watch: &Watch) -> Value {
    watch.vars["show"].clone()
}

fn stage_names(watch: &Watch, pipeline: &str) -> Vec<&'static str> {
    let listed = watch.pipelines();
    let (_, found) = listed
        .iter()
        .find(|(name, _)| name == pipeline)
        .expect("the pipeline should be listed");
    found.stages().iter().map(|stage| stage.name()).collect()
}

// @behavior LAY-001
#[test]
fn should_let_a_folder_override_a_variable_by_name() {
    let under = watch().under(&[folder("vars = { show = \"Beta\" }")]);

    assert_eq!(show(&under), Value::Text("Beta".to_string()));
}

// @behavior LAY-002
#[test]
fn should_let_the_nearest_folder_win() {
    let farthest = folder("vars = { show = \"Beta\" }");
    let nearest = folder("vars = { show = \"Gamma\" }");

    let under = watch().under(&[farthest, nearest]);

    assert_eq!(show(&under), Value::Text("Gamma".to_string()));
}

// @behavior LAY-003
#[test]
fn should_replace_a_pipeline_whole_by_name() {
    let under = watch().under(&[folder(
        "[pipeline.video]\nstages = [{ filter = { ext = [\"mp4\"] } }]",
    )]);

    assert_eq!(stage_names(&under, "video"), ["filter"]);
}

// @behavior LAY-004
#[test]
fn should_keep_the_pipelines_of_other_names() {
    let under = watch().under(&[folder("[pipeline.video]\nstages = [\"move\"]")]);

    assert_eq!(stage_names(&under, "photo"), ["move"]);
}

// @behavior LAY-005
#[test]
fn should_let_a_folder_set_the_batch_limit() {
    let under = watch().under(&[folder("max_files = 20")]);

    assert_eq!(under.max_files, 20);
}

// @behavior LAY-006
#[test]
fn should_not_let_a_folder_set_the_source() {
    assert!(names(&refused("source = \"/etc\""), "source"));
}

// @behavior LAY-007
#[test]
fn should_not_let_a_folder_set_the_target() {
    assert!(names(&refused("target = \"/etc\""), "target"));
}

// @behavior LAY-008
#[test]
fn should_not_let_a_folder_set_the_unit() {
    assert!(names(&refused("unit = \"source\""), "unit"));
}

// @behavior LAY-009
#[test]
fn should_not_let_a_folder_switch_off_a_dry_run() {
    assert!(names(&refused("dry_run = false"), "dry_run"));
}

// @behavior LAY-011
#[test]
fn should_not_let_a_folder_set_the_quiet() {
    assert!(names(&refused("quiet = \"1s\""), "quiet"));
}

// @behavior LAY-011
#[test]
fn should_refuse_an_unknown_key_by_name() {
    assert!(names(&refused("colour = \"red\""), "colour"));
}

// @behavior LAY-012
#[test]
fn should_refuse_a_folder_configuration_that_is_too_large() {
    let text = format!("# {}\n", "x".repeat(64 * 1024));

    assert_eq!(refused(&text), ConfigError::TooLarge);
}

// @behavior LAY-013
#[test]
fn should_refuse_a_mistake_in_a_folder_pipeline_with_its_name() {
    assert!(names(
        &refused("[pipeline.video]\nstages = [\"shred\"]"),
        "video"
    ));
}

// @behavior LAY-015
#[test]
fn should_not_run_a_pipeline_a_folder_defines_unless_the_watch_lists_it() {
    let under = watch().under(&[folder("[pipeline.extra]\nstages = [\"move\"]")]);

    let names: Vec<_> = under
        .pipelines()
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    assert_eq!(names, ["video", "photo"]);
}

// @behavior LAY-016
#[test]
fn should_refuse_a_folder_batch_limit_above_the_ceiling() {
    assert!(names(&refused("max_files = 100001"), "max_files"));
}
