use std::ops::Range;
use std::sync::LazyLock;

use regex::Regex;

use super::{Number, Outcome, Prefix, field_text, write_field};
use crate::record::{Record, Value};

/// Markers that say outright which number is the episode, in the order they are tried.
static MARKERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)\bS[0-9]{1,2}E([0-9]{1,3})\b",
        r"(?i)\bEP?[\s._-]?([0-9]{1,3})\b",
        r"(?:^|[\s_-])-[\s_]*([0-9]{1,3})(?:[\s_]|$|\[|\()",
        r"(?i)\bepisode[\s._-]?([0-9]{1,3})\b",
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("the marker patterns are fixed"))
    .collect()
});

/// Noise that is a number as a whole.
static WHOLE_NOISE: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)\b[0-9]{3,4}x[0-9]{3,4}\b",
        r"\b[0-9]{1,2}\.[0-9]{1,2}\b",
        r"\b[0-9]{4}[.\-/][0-9]{1,2}[.\-/][0-9]{1,2}\b",
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("the noise patterns are fixed"))
    .collect()
});

/// Noise where only the first group is the number.
static GROUP_NOISE: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?i)([0-9]{1,2})bit\b",
        r"(?i)[a-z]{2,6}x([0-9]{1,2})\b",
        r"(?i)[0-9]v([0-9]{1,2})\b",
        r"(?i)\b([0-9]{1,2})(?:st|nd|rd|th)\b",
        r"(?i)\bseason[\s._-]*([0-9]{1,2})\b",
        r"(?i)\bS([0-9]{1,2})\b",
        r"第([0-9]{1,3})季",
        r"([0-9]{1,2})[:：]",
        r"(?i)\b(?:AV|VP|MP)([0-9]{1,2})\b",
        r"(?i)\bH\.?([0-9]{2,3})\b",
        r"(?i)\bx(26[0-9])\b",
        r"([0-9]+)[\x{00B2}\x{00B3}\x{00B9}\x{2070}-\x{2079}]",
    ]
    .iter()
    .map(|pattern| Regex::new(pattern).expect("the noise patterns are fixed"))
    .collect()
});

static DIGITS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[0-9]+").expect("the digits pattern is fixed"));

const RESOLUTIONS: [&str; 7] = ["480", "720", "1080", "2160", "4320", "360", "540"];

pub(super) fn apply(number: &Number, mut record: Record) -> Outcome {
    let text = match field_text(&record, &number.from) {
        Ok(text) => text.into_owned(),
        Err(reason) => return Outcome::rejected("number", reason),
    };
    let excluded = excluded_values(&record, &number.exclude);

    let found = match (&number.prefix, number.nth) {
        (Some(prefix), nth) => pick(after_prefix(prefix, &text, &excluded), nth),
        (None, None) => marker(&text).or_else(|| pick(scan(&text, &excluded), None)),
        (None, nth) => pick(scan(&text, &excluded), nth),
    };

    if let Some(found) = found
        && let Err(refused) = write_field("number", &mut record, &number.into, Value::Number(found))
    {
        return refused;
    }
    Outcome::Continue(record)
}

fn excluded_values(record: &Record, fields: &[String]) -> Vec<u64> {
    fields
        .iter()
        .filter_map(|field| match record.field(field) {
            Some(Value::Number(number)) => Some(*number),
            Some(Value::Text(text)) => text.parse().ok(),
            _ => None,
        })
        .collect()
}

fn marker(text: &str) -> Option<u64> {
    MARKERS
        .iter()
        .find_map(|marker| marker.captures(text))
        .and_then(|captures| captures[1].parse().ok())
}

fn after_prefix(prefix: &Prefix, text: &str, excluded: &[u64]) -> Vec<u64> {
    prefix
        .pattern
        .captures_iter(text)
        .filter_map(|captures| captures[1].parse().ok())
        .filter(|found| !excluded.contains(found))
        .collect()
}

/// Every number left once the noise is set aside, in the order it is written.
fn scan(text: &str, excluded: &[u64]) -> Vec<u64> {
    let noise = noise_spans(text);
    DIGITS
        .find_iter(text)
        .filter(|digits| !is_noise(text, digits.range(), &noise))
        .filter_map(|digits| digits.as_str().parse().ok())
        .filter(|found| !excluded.contains(found))
        .collect()
}

fn noise_spans(text: &str) -> Vec<Range<usize>> {
    let whole = WHOLE_NOISE
        .iter()
        .flat_map(|noise| noise.find_iter(text).map(|found| found.range()));
    let grouped = GROUP_NOISE.iter().flat_map(|noise| {
        noise
            .captures_iter(text)
            .filter_map(|captures| captures.get(1))
            .map(|group| group.range())
    });
    whole.chain(grouped).collect()
}

fn is_noise(text: &str, digits: Range<usize>, noise: &[Range<usize>]) -> bool {
    let run = &text[digits.clone()];
    let next_to_hex = |ch: Option<char>| ch.is_some_and(|ch| matches!(ch, 'a'..='f' | 'A'..='F'));
    let year = run.len() == 4
        && run
            .parse::<u32>()
            .is_ok_and(|year| (1900..=2100).contains(&year));

    RESOLUTIONS.contains(&run)
        || run.len() > 4
        || year
        || noise
            .iter()
            .any(|span| span.start <= digits.start && digits.end <= span.end)
        || next_to_hex(text[..digits.start].chars().next_back())
        || next_to_hex(text[digits.end..].chars().next())
}

/// One candidate is taken only when it is the only one, unless the wanted position is given:
/// counted from 1, or from the end when negative.
fn pick(candidates: Vec<u64>, nth: Option<i64>) -> Option<u64> {
    match nth {
        None if candidates.len() == 1 => candidates.first().copied(),
        None => None,
        Some(nth) if nth > 0 => candidates.get(usize::try_from(nth - 1).ok()?).copied(),
        Some(nth) => {
            let from_end = usize::try_from(nth.checked_neg()?).ok()?;
            candidates
                .get(candidates.len().checked_sub(from_end)?)
                .copied()
        }
    }
}
