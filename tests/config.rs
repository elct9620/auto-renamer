mod common;

use std::path::Path;
use std::time::Duration;

use auto_renamer::{Config, ConfigError, Unit, Value, Watch};
use common::names;

const PIPELINES: &str = r#"
[pipeline.video]
stages = [{ filter = { ext = ["mkv"] } }]

[pipeline.photo]
stages = []

[target.library]
path = "/library"
"#;

/// A configuration of the two pipelines above and a watch `series` with the extra lines added to it.
fn with_watch(extra: &str) -> String {
    format!(
        "{PIPELINES}\n[watch.series]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"video\", move = \"library\" }}]\n{extra}\n"
    )
}

fn read(text: &str) -> Vec<Watch> {
    Config::parse(text)
        .expect("the configuration should be accepted")
        .watches()
        .to_vec()
}

fn series(extra: &str) -> Watch {
    read(&with_watch(extra)).remove(0)
}

fn refused(text: &str) -> ConfigError {
    Config::parse(text).expect_err("the configuration should be refused")
}

fn pipeline_names(watch: &Watch) -> Vec<String> {
    watch
        .pipelines()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

// @behavior CFG-001
#[test]
fn should_read_the_source_and_routes_of_a_watch() {
    let watch = series("");

    assert_eq!(watch.source, Path::new("/downloads"));
    assert_eq!(watch.routes().len(), 1);
    assert_eq!(watch.routes()[0].pipeline, "video");
    assert_eq!(
        watch.routes()[0].target.as_deref(),
        Some(Path::new("/library"))
    );
}

// @behavior CFG-002
#[test]
fn should_give_every_watch_the_pipelines_the_default_lists() {
    let text = format!(
        "{PIPELINES}\n[default]\nroutes = [{{ pipeline = \"video\" }}]\n\n[watch.series]\nsource = \"/downloads\"\n"
    );

    assert_eq!(pipeline_names(&read(&text)[0]), ["video"]);
}

// @behavior CFG-003
#[test]
fn should_let_a_watch_override_the_default() {
    let text = format!(
        "{PIPELINES}\n[default]\nroutes = [{{ pipeline = \"video\" }}]\n\n[watch.series]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"photo\" }}]\n"
    );

    assert_eq!(pipeline_names(&read(&text)[0]), ["photo"]);
}

// @behavior CFG-004
#[test]
fn should_merge_the_variables_of_the_default_and_the_watch_by_name() {
    let text = format!(
        "{PIPELINES}\n[default]\nvars = {{ show = \"D\", year = 2026 }}\n\n[watch.series]\nsource = \"/downloads\"\nvars = {{ show = \"W\" }}\n"
    );

    let watch = read(&text).remove(0);

    assert_eq!(watch.vars["show"], Value::Text("W".to_string()));
    assert_eq!(watch.vars["year"], Value::Number(2026));
}

// @behavior CFG-005
#[test]
fn should_default_the_quiet_to_five_minutes() {
    assert_eq!(series("").quiet, Duration::from_secs(300));
}

// @behavior CFG-006
#[test]
fn should_default_the_maximum_wait_to_thirty_minutes() {
    assert_eq!(series("").max_wait, Duration::from_secs(1800));
}

// @behavior CFG-007
#[test]
fn should_default_to_a_file_limit_of_a_thousand() {
    assert_eq!(series("").max_files, 1000);
}

// @behavior CFG-008
#[test]
fn should_default_to_no_dry_run() {
    assert!(!series("").dry_run);
}

// @behavior CFG-009
#[test]
fn should_read_durations_in_seconds_minutes_and_hours() {
    let watch = series("quiet = \"90s\"\nmax_wait = \"2h\"");

    assert_eq!(watch.quiet, Duration::from_secs(90));
    assert_eq!(watch.max_wait, Duration::from_secs(7200));
}

// @behavior CFG-010
#[test]
fn should_refuse_a_duration_without_a_unit() {
    assert!(names(&refused(&with_watch("quiet = \"5\"")), "quiet"));
}

// @behavior CFG-011
#[test]
fn should_refuse_a_maximum_wait_shorter_than_the_quiet_period() {
    let error = refused(&with_watch("quiet = \"10m\"\nmax_wait = \"5m\""));

    assert!(names(&error, "max_wait"));
}

// @behavior CFG-012
#[test]
fn should_refuse_a_source_that_is_not_absolute() {
    let text = format!("{PIPELINES}\n[watch.series]\nsource = \"downloads\"\n");

    assert!(names(&refused(&text), "source"));
}

// @behavior CFG-013
#[test]
fn should_refuse_a_watch_without_a_source() {
    let text = format!("{PIPELINES}\n[watch.series]\ndry_run = true\n");

    assert!(names(&refused(&text), "source"));
}

// @behavior CFG-014
#[test]
fn should_refuse_a_target_inside_a_source() {
    let text = "[target.library]\npath = \"/downloads/library\"\n\n[watch.series]\nsource = \"/downloads\"\n";

    assert!(names(&refused(text), "path"));
}

// @behavior CFG-015
#[test]
fn should_refuse_a_target_that_is_a_source() {
    let text =
        "[target.library]\npath = \"/downloads\"\n\n[watch.series]\nsource = \"/downloads\"\n";

    assert!(names(&refused(text), "path"));
}

// @behavior CFG-016
#[test]
fn should_refuse_two_watches_whose_sources_overlap() {
    let text = format!(
        "{PIPELINES}\n[watch.a]\nsource = \"/downloads\"\n\n[watch.b]\nsource = \"/downloads/movies\"\n"
    );

    assert!(names(&refused(&text), "source"));
}

// @behavior CFG-017
#[test]
fn should_refuse_a_source_inside_a_target() {
    let text =
        "[target.library]\npath = \"/library\"\n\n[watch.b]\nsource = \"/library/incoming\"\n";

    assert!(names(&refused(text), "path"));
}

// @behavior CFG-018
#[test]
fn should_refuse_a_route_naming_a_pipeline_that_is_not_defined() {
    let text = format!(
        "{PIPELINES}\n[watch.series]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"missing\" }}]\n"
    );

    assert!(names(&refused(&text), "routes"));
}

// @behavior CFG-019
#[test]
fn should_refuse_a_mistake_in_a_pipeline_with_its_name() {
    let text =
        "[pipeline.video]\nstages = [\"shred\"]\n\n[watch.series]\nsource = \"/downloads\"\n";

    assert!(names(&refused(text), "video"));
}

// @behavior CFG-020
#[test]
fn should_refuse_an_unknown_key_by_name() {
    assert!(names(&refused(&with_watch("colour = \"red\"")), "colour"));
}

// @behavior CFG-021
#[test]
fn should_let_a_route_without_a_move_rename_in_place() {
    let text = format!(
        "{PIPELINES}\n[watch.series]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"video\" }}]\n"
    );

    assert_eq!(read(&text)[0].routes()[0].target, None);
}

// @behavior CFG-022
#[test]
fn should_refuse_a_file_limit_that_is_not_positive() {
    assert!(names(&refused(&with_watch("max_files = 0")), "max_files"));
}

// @behavior CFG-023
#[test]
fn should_refuse_a_document_that_is_not_toml() {
    assert!(matches!(
        refused("this is = = not toml"),
        ConfigError::Syntax(_)
    ));
}

// @behavior CFG-024
#[test]
fn should_refuse_a_variable_that_is_neither_text_nor_a_whole_number() {
    assert!(names(
        &refused(&with_watch("vars = { show = [\"a\"] }")),
        "show"
    ));
}

// @behavior CFG-025
#[test]
fn should_let_a_watch_switch_on_a_dry_run() {
    assert!(series("dry_run = true").dry_run);
}

#[test]
fn should_default_the_unit_to_the_folder_of_the_file() {
    assert!(matches!(series("").unit, Unit::Directory));
}

// @behavior CFG-027
#[test]
fn should_refuse_a_default_that_holds_a_source() {
    let text = format!("{PIPELINES}\n[default]\nsource = \"/downloads\"\n");

    assert!(names(&refused(&text), "source"));
}

// @behavior CFG-028
#[test]
fn should_refuse_a_file_limit_above_the_ceiling() {
    assert!(names(
        &refused(&with_watch("max_files = 100001")),
        "max_files"
    ));
}

// @behavior CFG-029
#[test]
fn should_accept_the_ceiling_as_the_file_limit() {
    assert_eq!(series("max_files = 100000").max_files, 100_000);
}

// @behavior CFG-031
#[test]
fn should_refuse_a_source_written_with_a_parent_folder() {
    let text = format!(
        "{PIPELINES}\n[watch.series]\nsource = \"/downloads/../library\"\nroutes = [{{ pipeline = \"video\" }}]\n"
    );

    assert!(names(&refused(&text), "source"));
}

/// The real location of a path, where `/link` is a link to `to`.
fn linked(to: &'static str) -> impl Fn(&Path) -> std::path::PathBuf {
    move |path: &Path| match path.strip_prefix("/link") {
        Ok(rest) => Path::new(to).join(rest),
        Err(_) => path.to_path_buf(),
    }
}

// @behavior CFG-032
#[test]
fn should_refuse_sources_that_overlap_once_their_real_paths_are_known() {
    let config = Config::parse(&format!(
        "{PIPELINES}\n[watch.a]\nsource = \"/link\"\nroutes = [{{ pipeline = \"video\" }}]\n\n[watch.b]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"video\" }}]\n"
    ))
    .expect("the configuration should be accepted as written");

    let checked = config.check_paths(
        Path::new("/etc/auto-renamer/config.toml"),
        linked("/downloads/sub"),
    );

    assert!(names(
        &checked.expect_err("the paths should be refused"),
        "source"
    ));
}

// @behavior CFG-033
#[test]
fn should_refuse_a_configuration_file_inside_a_source_once_real_paths_are_known() {
    let config = Config::parse(&format!(
        "{PIPELINES}\n[watch.a]\nsource = \"/link\"\nroutes = [{{ pipeline = \"video\" }}]\n"
    ))
    .expect("the configuration should be accepted as written");

    let checked = config.check_paths(
        Path::new("/etc/auto-renamer/config.toml"),
        linked("/etc/auto-renamer"),
    );

    assert!(names(
        &checked.expect_err("the paths should be refused"),
        "source"
    ));
}

// @behavior CFG-034
#[test]
fn should_refuse_a_route_moving_to_a_target_that_is_not_declared() {
    let text = format!(
        "{PIPELINES}\n[watch.series]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"video\", move = \"missing\" }}]\n"
    );

    assert!(names(&refused(&text), "routes"));
}

// @behavior CFG-035
#[test]
fn should_refuse_a_target_without_a_path() {
    let text = "[target.library]\n\n[watch.series]\nsource = \"/downloads\"\n";

    assert!(names(&refused(text), "path"));
}
