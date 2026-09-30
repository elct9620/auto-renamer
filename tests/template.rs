use std::path::Path;

use auto_renamer::{Record, RenderError, Template, TemplateError, Value};
use chrono::{TimeZone, Utc};

fn record() -> Record {
    let mtime = Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap();
    Record::new(Path::new("Alpha - 12.mkv"), mtime).expect("the path should be text")
}

fn with(name: &str, value: Value) -> Record {
    let mut record = record();
    record.set_field(name, value);
    record
}

fn render(source: &str, record: &Record) -> Result<String, RenderError> {
    Template::parse(source)
        .expect("template should be readable")
        .render(record)
}

fn parse_error(source: &str) -> TemplateError {
    Template::parse(source).expect_err("template should be refused")
}

// @behavior TPL-001
#[test]
fn should_write_a_field_as_its_text() {
    let record = with("show", Value::Text("Alpha".to_string()));

    assert_eq!(render("{show} done", &record), Ok("Alpha done".to_string()));
}

// @behavior TPL-002
#[test]
fn should_pad_a_number_to_the_width_asked_for() {
    let record = with("episode", Value::Number(7));

    assert_eq!(render("e{episode:02}", &record), Ok("e07".to_string()));
}

// @behavior TPL-003
#[test]
fn should_keep_a_number_wider_than_the_padding() {
    let record = with("episode", Value::Number(1354));

    assert_eq!(render("e{episode:02}", &record), Ok("e1354".to_string()));
}

// @behavior TPL-004
#[test]
fn should_write_a_date_in_the_format_asked_for() {
    assert_eq!(
        render("{mtime:%Y-%m}", &record()),
        Ok("2026-09".to_string())
    );
}

// @behavior TPL-005
#[test]
fn should_write_an_optional_part_when_its_fields_are_present() {
    let record = with("index", Value::Number(1));

    assert_eq!(render("zh[.{index:02}]", &record), Ok("zh.01".to_string()));
}

// @behavior TPL-006
#[test]
fn should_leave_an_optional_part_out_when_a_field_is_missing() {
    assert_eq!(render("zh[.{index:02}]", &record()), Ok("zh".to_string()));
}

// @behavior TPL-007
#[test]
fn should_refuse_a_missing_required_field_by_name() {
    let refused = render("{name} s{season:02}", &record());

    assert_eq!(
        refused,
        Err(RenderError::MissingField("season".to_string()))
    );
}

// @behavior TPL-008
#[test]
fn should_write_doubled_braces_as_literal_braces() {
    assert_eq!(render("{{name}}", &record()), Ok("{name}".to_string()));
}

// @behavior TPL-009
#[test]
fn should_write_doubled_brackets_as_literal_brackets() {
    assert_eq!(render("[[tag]]", &record()), Ok("[tag]".to_string()));
}

// @behavior TPL-012
#[test]
fn should_refuse_nested_optional_parts() {
    assert_eq!(parse_error("a[b[c]]"), TemplateError::NestedOptional);
}

// @behavior TPL-014
#[test]
fn should_refuse_a_padding_on_text() {
    let record = with("show", Value::Text("Alpha".to_string()));

    let refused = render("{show:02}", &record);

    assert_eq!(
        refused,
        Err(RenderError::SpecMismatch {
            field: "show".to_string(),
            spec: "02".to_string()
        })
    );
}

// @behavior TPL-016
#[test]
fn should_refuse_a_date_without_a_format() {
    assert_eq!(
        render("{mtime}", &record()),
        Err(RenderError::NeedsFormat("mtime".to_string()))
    );
}

// @behavior TPL-017
#[test]
fn should_refuse_a_padding_wider_than_any_number() {
    assert_eq!(
        parse_error("{episode:021}"),
        TemplateError::InvalidSpec("021".to_string())
    );
}

// @behavior TPL-018
#[test]
fn should_refuse_a_date_format_on_a_number() {
    let record = with("episode", Value::Number(7));

    let refused = render("{episode:%Y}", &record);

    assert_eq!(
        refused,
        Err(RenderError::SpecMismatch {
            field: "episode".to_string(),
            spec: "%Y".to_string()
        })
    );
}

// @behavior TPL-019
#[test]
fn should_refuse_a_padding_on_a_date() {
    let refused = render("{mtime:02}", &record());

    assert_eq!(
        refused,
        Err(RenderError::SpecMismatch {
            field: "mtime".to_string(),
            spec: "02".to_string()
        })
    );
}
