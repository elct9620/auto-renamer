mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use auto_renamer::{Context, Outcome, Pipeline};
use common::{Files, record_on, text};

/// The pipelines the design gives as examples, read from the design itself so they cannot drift from it.
fn design_pipelines() -> BTreeMap<String, Pipeline> {
    let design = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/design.md"))
        .expect("the design should be readable");

    let mut pipelines = BTreeMap::new();
    for block in design.split("```toml").skip(1) {
        let source = block.split("```").next().unwrap_or_default();
        let Ok(document) = source.parse::<toml::Table>() else {
            continue;
        };
        let Some(declared) = document.get("pipeline").and_then(toml::Value::as_table) else {
            continue;
        };
        for (name, table) in declared {
            let stages = table
                .get("stages")
                .and_then(toml::Value::as_array)
                .expect("a pipeline lists its stages");
            pipelines.insert(
                name.clone(),
                Pipeline::declare(stages).expect("the design's pipeline should be accepted"),
            );
        }
    }
    pipelines
}

fn planned_against(
    files: &Files,
    pipeline: &str,
    show: Option<&str>,
    path: &str,
    month: (i32, u32, u32),
) -> PathBuf {
    let pipelines = design_pipelines();
    let pipeline = pipelines
        .get(pipeline)
        .unwrap_or_else(|| panic!("the design has no `{pipeline}` pipeline"));
    let vars = show
        .map(|show| BTreeMap::from([("show".to_string(), text(show))]))
        .unwrap_or_default();
    let record = record_on(path, month.0, month.1, month.2).with_vars(vars);

    match pipeline.plan(record, &mut Context::new(files)) {
        Outcome::Continue(planned) => planned.plan().to_path_buf(),
        other => panic!("expected the file to be planned, got {other:?}"),
    }
}

fn planned(pipeline: &str, show: Option<&str>, path: &str, month: (i32, u32, u32)) -> PathBuf {
    planned_against(&Files::none(), pipeline, show, path, month)
}

fn series(show: &str, path: &str) -> PathBuf {
    planned("video", Some(show), path, (2026, 9, 27))
}

fn series_against(files: &Files, show: &str, path: &str) -> PathBuf {
    planned_against(files, "video", Some(show), path, (2026, 9, 27))
}

// @behavior EX-001
#[test]
fn should_plan_a_number_after_a_dash() {
    let plan = series(
        "Alpha",
        "Series/Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv",
    );

    assert_eq!(plan, PathBuf::from("Series/Alpha/Alpha s01e12.mkv"));
}

// @behavior EX-002
#[test]
fn should_plan_a_number_in_brackets_among_other_brackets() {
    let plan = series(
        "Beta Show",
        "Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4",
    );

    assert_eq!(plan, PathBuf::from("Series/Beta Show/Beta Show s01e12.mp4"));
}

// @behavior EX-003
#[test]
fn should_plan_a_number_beside_a_version_a_hash_and_a_year() {
    let plan = series(
        "Gamma Show",
        "Series/Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv",
    );

    assert_eq!(
        plan,
        PathBuf::from("Series/Gamma Show/Gamma Show s01e18.mkv")
    );
}

// @behavior EX-004
#[test]
fn should_plan_a_number_before_a_resolution_and_a_codec() {
    let plan = series(
        "Delta-Show Kai",
        "Series/Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv",
    );

    assert_eq!(
        plan,
        PathBuf::from("Series/Delta-Show Kai/Delta-Show Kai s01e13.mkv")
    );
}

// @behavior EX-005
#[test]
fn should_plan_a_four_digit_number_beside_a_date() {
    let plan = series(
        "Epsilon",
        "Series/Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4",
    );

    assert_eq!(plan, PathBuf::from("Series/Epsilon/Epsilon s01e1354.mp4"));
}

// @behavior EX-006
#[test]
fn should_plan_a_season_from_the_folder_and_season_marks_in_the_name() {
    let plan = series(
        "Zeta-Show",
        "Series/Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv",
    );

    assert_eq!(
        plan,
        PathBuf::from("Series/Zeta-Show/Season 03/Zeta-Show s03e10.mkv")
    );
}

// @behavior EX-007
#[test]
fn should_remove_a_tag_from_a_movie() {
    let plan = planned(
        "movie",
        None,
        "Movies/XXX/[Group] XXX [1080p].mkv",
        (2026, 9, 27),
    );

    assert_eq!(plan, PathBuf::from("Movies/XXX/XXX.mkv"));
}

// @behavior EX-008
#[test]
fn should_keep_a_year_in_parentheses_in_a_movie_name() {
    let plan = planned(
        "movie",
        None,
        "Movies/Movie (2026)/[Group] Movie (2026).mkv",
        (2026, 9, 27),
    );

    assert_eq!(plan, PathBuf::from("Movies/Movie (2026)/Movie (2026).mkv"));
}

// @behavior EX-009
#[test]
fn should_rewrite_a_track_number_and_title() {
    let plan = planned(
        "music",
        None,
        "Music/Artist/Album/03 - Title.mp3",
        (2026, 9, 27),
    );

    assert_eq!(plan, PathBuf::from("Music/Artist/Album/03 Title.mp3"));
}

// @behavior EX-010
#[test]
fn should_put_a_photo_in_the_folder_of_its_month() {
    let plan = planned("photo", None, "Photos/IMG_0001.jpg", (2026, 9, 27));

    assert_eq!(plan, PathBuf::from("Photos/2026/09/IMG_0001.jpg"));
}

// @behavior EX-011
#[test]
fn should_put_a_photo_of_another_month_in_that_month() {
    let plan = planned("photo", None, "Photos/IMG_0002.jpg", (2026, 10, 1));

    assert_eq!(plan, PathBuf::from("Photos/2026/10/IMG_0002.jpg"));
}

// @behavior EX-012
#[test]
fn should_not_take_the_first_episode_for_the_default_season() {
    let plan = series("Alpha", "Series/Alpha/[Team] Alpha [01].mkv");

    assert_eq!(plan, PathBuf::from("Series/Alpha/Alpha s01e01.mkv"));
}

// @behavior EX-013
#[test]
fn should_fall_back_to_the_next_number_when_the_number_cannot_be_told() {
    let plan = series(
        "Eta Show",
        "Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4",
    );

    assert_eq!(
        plan,
        PathBuf::from("Series/Eta Show/Season 17/Eta Show s17e01.mp4")
    );
}

// @behavior EX-014
#[test]
fn should_follow_the_target_for_a_file_without_a_number() {
    let files = Files::of(&[("Series/Show", &["Show s01e01.mkv", "Show s01e02.mkv"])]);

    let plan = series_against(&files, "Show", "Series/Show/Show new a.mkv");

    assert_eq!(plan, PathBuf::from("Series/Show/Show s01e03.mkv"));
}

// @behavior EX-015
#[test]
fn should_follow_the_folder_the_file_is_lifted_into() {
    let files = Files::of(&[
        ("Series/Show/Season 01", &["Show s01e05.mkv"]),
        ("Series/Show/Season 02", &["Show s02e09.mkv"]),
    ]);

    let plan = series_against(&files, "Show", "Series/Show/Season 01/[Rel]/new.mkv");

    assert_eq!(plan, PathBuf::from("Series/Show/Season 01/Show s01e06.mkv"));
}
