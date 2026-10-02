mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use auto_renamer::{Judged, Pipeline, Verdict, plan_batch};
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
    let listed = vec![(
        pipeline.to_string(),
        pipelines
            .get(pipeline)
            .unwrap_or_else(|| panic!("the design has no `{pipeline}` pipeline"))
            .clone(),
    )];
    let vars = show
        .map(|show| BTreeMap::from([("show".to_string(), text(show))]))
        .unwrap_or_default();
    let record = record_on(path, month.0, month.1, month.2).with_vars(vars);

    // One file is a batch of one.
    let judged = plan_batch(&listed, vec![record], files);
    match &judged[..] {
        [
            Judged {
                verdict: Verdict::Planned(planned),
                ..
            },
        ] => planned.plan().to_path_buf(),
        other => panic!("expected the file to be planned, got {other:?}"),
    }
}

fn planned(pipeline: &str, show: Option<&str>, path: &str, month: (i32, u32, u32)) -> PathBuf {
    planned_against(&Files::none(), pipeline, show, path, month)
}

fn series(show: &str, path: &str) -> PathBuf {
    planned("series-video", Some(show), path, (2026, 9, 27))
}

fn series_against(files: &Files, show: &str, path: &str) -> PathBuf {
    planned_against(files, "series-video", Some(show), path, (2026, 9, 27))
}

// @behavior EX-006
#[test]
fn should_plan_a_season_from_the_folder_and_season_marks_in_the_name() {
    let plan = series(
        "Zeta-Show",
        "Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv",
    );

    assert_eq!(
        plan,
        PathBuf::from("Zeta-Show/Season 03/Zeta-Show s03e10.mkv")
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

// @behavior EX-012
#[test]
fn should_not_take_the_first_episode_for_the_default_season() {
    let plan = series("Alpha", "Alpha/[Team] Alpha [01].mkv");

    assert_eq!(plan, PathBuf::from("Alpha/Season 01/Alpha s01e01.mkv"));
}

// @behavior EX-013
#[test]
fn should_fall_back_to_the_next_number_when_the_number_cannot_be_told() {
    let plan = series(
        "Eta Show",
        "Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4",
    );

    assert_eq!(
        plan,
        PathBuf::from("Eta Show/Season 17/Eta Show s17e01.mp4")
    );
}

// @behavior EX-015
#[test]
fn should_follow_the_folder_the_file_is_lifted_into() {
    let files = Files::of(&[
        ("Show/Season 01", &["Show s01e05.mkv"]),
        ("Show/Season 02", &["Show s02e09.mkv"]),
    ]);

    let plan = series_against(&files, "Show", "Show/Season 01/[Rel]/new.mkv");

    assert_eq!(plan, PathBuf::from("Show/Season 01/Show s01e06.mkv"));
}

/// The built-in series pipelines of the design planned together over the files of a batch.
fn batch_against(files: &Files, show: &str, paths: &[&str]) -> Vec<(String, String)> {
    let pipelines = design_pipelines();
    let listed = vec![
        (
            "series-video".to_string(),
            pipelines["series-video"].clone(),
        ),
        (
            "series-subtitle".to_string(),
            pipelines["series-subtitle"].clone(),
        ),
    ];
    let vars = BTreeMap::from([("show".to_string(), text(show))]);
    let records = paths
        .iter()
        .map(|path| record_on(path, 2026, 9, 27).with_vars(vars.clone()))
        .collect();

    plan_batch(&listed, records, files)
        .into_iter()
        .map(|judged| {
            let answer = match judged.verdict {
                Verdict::Planned(record) => record.plan().display().to_string(),
                Verdict::Unclaimed => "unclaimed".to_string(),
                other => format!("{other:?}"),
            };
            (judged.origin.display().to_string(), answer)
        })
        .collect()
}

fn plans_of(answers: &[(String, String)], origin: &str) -> String {
    answers
        .iter()
        .find(|(found, _)| found == origin)
        .unwrap_or_else(|| panic!("no answer for {origin}"))
        .1
        .clone()
}

// @behavior EX-016
#[test]
fn should_plan_two_episodes_and_their_subtitles_in_one_batch() {
    let folder = "Show/Season 01";
    let paths = [
        "Show/Season 01/Show 27.mkv",
        "Show/Season 01/Show 27.cht.ass",
        "Show/Season 01/Show 27.chs.ass",
        "Show/Season 01/Show 28.mkv",
        "Show/Season 01/Show 28.cht.ass",
    ];

    let answers = batch_against(&Files::none(), "Show", &paths);

    assert_eq!(
        plans_of(&answers, paths[0]),
        format!("{folder}/Show s01e27.mkv")
    );
    assert_eq!(
        plans_of(&answers, paths[1]),
        format!("{folder}/Show s01e27.zh.01.ass")
    );
    assert_eq!(
        plans_of(&answers, paths[2]),
        format!("{folder}/Show s01e27.zh.02.ass")
    );
    assert_eq!(
        plans_of(&answers, paths[3]),
        format!("{folder}/Show s01e28.mkv")
    );
    assert_eq!(
        plans_of(&answers, paths[4]),
        format!("{folder}/Show s01e28.zh.ass")
    );
}

// @behavior EX-018
#[test]
fn should_plan_subtitles_in_a_folder_of_the_release_with_their_video() {
    let paths = [
        "Show/Season 01/[Rel 05]/Show 05.mkv",
        "Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass",
    ];

    let answers = batch_against(&Files::none(), "Show", &paths);

    assert_eq!(
        plans_of(&answers, paths[0]),
        "Show/Season 01/Show s01e05.mkv"
    );
    assert_eq!(
        plans_of(&answers, paths[1]),
        "Show/Season 01/Show s01e05.zh.ass"
    );
}

// @behavior EX-019
#[test]
fn should_continue_the_target_in_name_order_for_files_without_a_number() {
    let files = Files::of(&[("Show/Season 01", &["Show s01e01.mkv", "Show s01e02.mkv"])]);
    let paths = [
        "Show/Show new a.mkv",
        "Show/Show new b.mkv",
        "Show/Show 07.mkv",
    ];

    let answers = batch_against(&files, "Show", &paths);

    assert_eq!(
        plans_of(&answers, paths[2]),
        "Show/Season 01/Show s01e07.mkv"
    );
    assert_eq!(
        plans_of(&answers, paths[0]),
        "Show/Season 01/Show s01e03.mkv"
    );
    assert_eq!(
        plans_of(&answers, paths[1]),
        "Show/Season 01/Show s01e04.mkv"
    );
}

// @behavior EX-020
#[test]
fn should_plan_a_folder_per_episode_into_its_season_folder() {
    let paths = [
        "Alpha/[Team] Alpha - 01 [1080p]/Alpha 01.mkv",
        "Alpha/[Team] Alpha - 01 [1080p]/Alpha 01.cht.ass",
    ];

    let answers = batch_against(&Files::none(), "Alpha", &paths);

    assert_eq!(
        plans_of(&answers, paths[0]),
        "Alpha/Season 01/Alpha s01e01.mkv"
    );
    assert_eq!(
        plans_of(&answers, paths[1]),
        "Alpha/Season 01/Alpha s01e01.zh.ass"
    );
}

/// The `pipeline` table of the first block of the design declaring a pipeline of this name.
fn design_block_declaring(name: &str) -> toml::Table {
    let design = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/design.md"))
        .expect("the design should be readable");
    design
        .split("```toml")
        .skip(1)
        .filter_map(|block| block.split("```").next()?.parse::<toml::Table>().ok())
        .find_map(|document| {
            let declared = document.get("pipeline")?.as_table()?;
            declared.contains_key(name).then(|| declared.clone())
        })
        .unwrap_or_else(|| panic!("the design declares no `{name}`"))
}

// @behavior EX-021
#[test]
fn should_build_in_the_pipelines_the_design_gives() {
    let built_in: toml::Table = auto_renamer::BUILT_IN_PIPELINES
        .parse::<toml::Table>()
        .expect("the built-in pipelines are TOML")["pipeline"]
        .as_table()
        .expect("they declare pipelines")
        .clone();

    assert_eq!(built_in, design_block_declaring("series-video"));
}

/// The plan of one file through the built-in `series-video`, as a watch with these variables reads it.
fn built_in_plan(vars: &str, path: &str) -> PathBuf {
    let config = auto_renamer::Config::parse(&format!(
        "[watch.w]\nsource = \"/downloads\"\nroutes = [{{ pipeline = \"series-video\" }}]\n{vars}\n"
    ))
    .expect("the configuration should be accepted");
    let watch = &config.watches()[0];
    let listed = vec![(
        "series-video".to_string(),
        watch.pipeline("series-video").clone(),
    )];
    let record = record_on(path, 2026, 9, 27).with_vars(watch.vars.clone());

    match &plan_batch(&listed, vec![record], &Files::none())[..] {
        [
            Judged {
                verdict: Verdict::Planned(planned),
                ..
            },
        ] => planned.plan().to_path_buf(),
        other => panic!("expected the file to be planned, got {other:?}"),
    }
}

// @behavior EX-022
#[test]
fn should_take_the_show_from_its_folder_in_the_built_in_pipelines() {
    let plan = built_in_plan("", "Alpha/[Team] Alpha - 01 [1080p].mkv");

    assert_eq!(plan, PathBuf::from("Alpha/Season 01/Alpha s01e01.mkv"));
}

// @behavior EX-023
#[test]
fn should_let_the_show_the_variables_give_win_over_the_folder() {
    let plan = built_in_plan(
        "vars = { show = \"Gemma\" }",
        "Alpha/Season 01/[Beta Team] Gemma 01 [1080p].mkv",
    );

    assert_eq!(plan, PathBuf::from("Alpha/Season 01/Gemma s01e01.mkv"));
}
