use std::collections::BTreeMap;
use std::path::Path;

use auto_renamer::{Record, Value};
use chrono::{DateTime, TimeZone, Utc};

fn mtime() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap()
}

fn record(path: &str) -> Record {
    Record::new(Path::new(path), mtime())
}

fn text(value: &str) -> Value {
    Value::Text(value.to_string())
}

// @behavior REC-001
#[test]
fn should_name_the_main_file_name_without_its_extension() {
    let record = record("Series/Alpha/Alpha - 12.mkv");

    assert_eq!(record.field("name"), Some(&text("Alpha - 12")));
}

// @behavior REC-002
#[test]
fn should_expose_the_extension() {
    let record = record("Series/Alpha/Alpha - 12.mkv");

    assert_eq!(record.field("ext"), Some(&text("mkv")));
}

// @behavior REC-003
#[test]
fn should_start_the_extension_at_the_last_dot() {
    let record = record("Show 27.cht.ass");

    assert_eq!(record.field("name"), Some(&text("Show 27.cht")));
}

// @behavior REC-004
#[test]
fn should_leave_the_extension_empty_when_the_name_has_no_dot() {
    let record = record("README");

    assert_eq!(record.field("ext"), Some(&text("")));
}

// @behavior REC-005
#[test]
fn should_not_treat_a_leading_dot_as_an_extension() {
    let record = record(".hidden");

    assert_eq!(record.field("name"), Some(&text(".hidden")));
}

// @behavior REC-006
#[test]
fn should_expose_the_parent_folder_name() {
    let record = record("Series/Alpha/Season 03/Alpha - 12.mkv");

    assert_eq!(record.field("dir"), Some(&text("Season 03")));
}

// @behavior REC-007
#[test]
fn should_expose_the_folder_path() {
    let record = record("Series/Alpha/Season 03/Alpha - 12.mkv");

    assert_eq!(record.field("path"), Some(&text("Series/Alpha/Season 03")));
}

// @behavior REC-008
#[test]
fn should_leave_the_folder_fields_empty_at_the_source_root() {
    let record = record("loose.mkv");

    assert_eq!(record.field("dir"), Some(&text("")));
    assert_eq!(record.field("path"), Some(&text("")));
}

// @behavior REC-009
#[test]
fn should_expose_the_modification_time_as_a_date() {
    let record = record("Series/Alpha/Alpha - 12.mkv");

    assert_eq!(record.field("mtime"), Some(&Value::Date(mtime())));
}

// @behavior REC-010
#[test]
fn should_plan_the_relative_path_at_first() {
    let record = record("Series/Alpha/Alpha - 12.mkv");

    assert_eq!(record.plan(), Path::new("Series/Alpha/Alpha - 12.mkv"));
}

// @behavior REC-011
#[test]
fn should_add_variables_as_fields() {
    let vars = BTreeMap::from([("show".to_string(), text("Alpha"))]);

    let record = record("Alpha - 12.mkv").with_vars(vars);

    assert_eq!(record.field("show"), Some(&text("Alpha")));
}

// @behavior REC-012
#[test]
fn should_keep_a_built_in_field_when_a_variable_has_its_name() {
    let vars = BTreeMap::from([("name".to_string(), text("other"))]);

    let record = record("Alpha - 12.mkv").with_vars(vars);

    assert_eq!(record.field("name"), Some(&text("Alpha - 12")));
}

// @behavior REC-013
#[test]
fn should_rewrite_the_file_name_of_the_plan_when_the_name_is_written() {
    let mut record = record("Series/Alpha/x.mkv");

    record.set_field("name", text("y"));

    assert_eq!(record.plan(), Path::new("Series/Alpha/y.mkv"));
}

// @behavior REC-014
#[test]
fn should_rewrite_the_file_name_of_the_plan_when_the_extension_is_written() {
    let mut record = record("Series/Alpha/x.mkv");

    record.set_field("ext", text("mp4"));

    assert_eq!(record.plan(), Path::new("Series/Alpha/x.mp4"));
}

// @behavior REC-015
#[test]
fn should_leave_the_plan_alone_when_the_name_cannot_be_a_file_name() {
    let mut record = record("Series/Alpha/x.mkv");

    record.set_field("name", text("a/b"));

    assert_eq!(record.plan(), Path::new("Series/Alpha/x.mkv"));
}
