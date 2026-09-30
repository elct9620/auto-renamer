# Regex

A regex stage extracts text into fields, or rewrites a field. It is the way out when the built-in stages cannot say what is wanted, so it runs on a regular expression that cannot take unbounded time.

## Includes

- `tests/regex_stage.rs`

## `RGX-001` The first group is written into the field asked for

| Step | Statement |
| --- | --- |
| Given | a regex stage writing the pattern `Show (\w+)` into `word` and a record for `Show Alpha.mkv` |
| When | the stage is applied |
| Then | `word` is `Alpha` |

## `RGX-002` Without a group the whole match is written

| Step | Statement |
| --- | --- |
| Given | a regex stage writing the pattern `A[a-z]+` into `word` and a record for `Show Alpha.mkv` |
| When | the stage is applied |
| Then | `word` is `Alpha` |

## `RGX-003` Digits alone become a number

| Step | Statement |
| --- | --- |
| Given | a regex stage writing the pattern `([0-9]+)` into `episode` and a record for `Show 007.mkv` |
| When | the stage is applied |
| Then | `episode` is the number 7 |

## `RGX-004` Named groups become fields

| Step | Statement |
| --- | --- |
| Given | a regex stage with the pattern `(?<track>[0-9]+) - (?<title>.+)` and a record for `03 - Song.mp3` |
| When | the stage is applied |
| Then | `track` is 3 and `title` is `Song` |

## `RGX-005` A rewrite replaces every match

| Step | Statement |
| --- | --- |
| Given | a regex stage rewriting `name` with the pattern `[._]` replaced by a space and a record for `a.b_c.mkv` |
| When | the stage is applied |
| Then | `name` is `a b c` |

## `RGX-007` No match leaves the record unchanged

| Step | Statement |
| --- | --- |
| Given | a regex stage writing the pattern `Z+` into `word` and a record for `Show.mkv` |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `RGX-008` A source field that does not exist is refused

| Step | Statement |
| --- | --- |
| Given | a regex stage reading from `missing` and a record for `Show.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `regex` |

## `RGX-009` Another field can be read

| Step | Statement |
| --- | --- |
| Given | a regex stage writing the pattern `Season ([0-9]+)` into `season` from `path` and a record for `Show/Season 4/x.mkv` |
| When | the stage is applied |
| Then | `season` is 4 |

## `RGX-010` A date field cannot be read

| Step | Statement |
| --- | --- |
| Given | a regex stage reading from `mtime` and a record for `Show.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `regex` |

