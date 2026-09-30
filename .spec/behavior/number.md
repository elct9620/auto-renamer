# Number

A number stage reads one number out of a field and writes it into another. A marker that says outright what the number is wins; otherwise the numbers left after the noise is set aside must be exactly one, or the field stays unset rather than guessed.

## Includes

- `tests/number.rs`

## `NUM-001` A season and episode marker gives the episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show S01E12.mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-002` An episode marker gives the episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show EP07.mkv` |
| When | the stage is applied |
| Then | `episode` is 7 |

## `NUM-003` A dash before a number gives the episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `[Team] Show - 12 [1080p].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-004` The word episode gives the episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show episode 5.mkv` |
| When | the stage is applied |
| Then | `episode` is 5 |

## `NUM-005` A marker wins over other numbers

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `[Team 7] Show - 12 [1080p].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-006` The only number left is the episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][WEB].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-007` A resolution is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][1080p][2160p].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-008` A codec is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][x264][h.265].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-009` A bit depth is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][HEVC-10bit].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-010` A version is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show 18v2.mkv` |
| When | the stage is applied |
| Then | `episode` is 18 |

## `NUM-011` A date is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][2026.09.26].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-012` A year is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12](2026).mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-013` An audio specification is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12] OPUS 2.0.mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-014` A season marker is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show S03 [10].mkv` |
| When | the stage is applied |
| Then | `episode` is 10 |

## `NUM-015` A hash is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [12][5E9D2F64].mkv` |
| When | the stage is applied |
| Then | `episode` is 12 |

## `NUM-016` A superscript number is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `[Team7³] Show [10].mkv` |
| When | the stage is applied |
| Then | `episode` is 10 |

## `NUM-017` Four digits can be an episode

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show [1354][2026.09.26].mkv` |
| When | the stage is applied |
| Then | `episode` is 1354 |

## `NUM-018` Several candidates leave the field unset

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `[Team-7][Show 17][03].mkv` |
| When | the stage is applied |
| Then | `episode` is unset |

## `NUM-019` The nth candidate is taken when asked for

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode`, excluding `season` and taking the 2nd candidate, and a record for `[Team-7][Show 17][03].mkv` with `season` 17 |
| When | the stage is applied |
| Then | `episode` is 3 |

## `NUM-020` A negative nth counts from the end

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode`, excluding `season` and taking the 2nd candidate from the end, and a record for `[Team-7][Show 17][03].mkv` with `season` 17 |
| When | the stage is applied |
| Then | `episode` is 7 |

## `NUM-021` An nth beyond the candidates leaves the field unset

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` taking the 2nd candidate and a record for `Show [12].mkv` |
| When | the stage is applied |
| Then | `episode` is unset |

## `NUM-022` A number equal to an excluded field is not a candidate

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and excluding `season`, and a record for `Show 17 [03].mkv` with `season` 17 |
| When | the stage is applied |
| Then | `episode` is 3 |

## `NUM-023` Excluding a field that is unset excludes nothing

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` excluding `season` and a record for `Show 01.mkv` |
| When | the stage is applied |
| Then | `episode` is 1 |

## `NUM-024` Asking for the nth candidate skips markers

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and taking the 2nd candidate, and a record for `Show - 12 [7].mkv` |
| When | the stage is applied |
| Then | `episode` is 7 |

## `NUM-025` A prefix takes the number after its word

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `season` from `path` after the word `Season` and a record for `Series/Alpha/Season 03/x.mkv` |
| When | the stage is applied |
| Then | `season` is 3 |

## `NUM-026` A prefix that is not in the text leaves the field unset

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `season` from `path` after the word `Season` and a record for `Series/Alpha/x.mkv` |
| When | the stage is applied |
| Then | `season` is unset |

## `NUM-027` A source field that does not exist is refused

| Step | Statement |
| --- | --- |
| Given | a number stage reading from `missing` and a record for `Show 12.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `number` |

## `NUM-028` A name without numbers leaves the field unset

| Step | Statement |
| --- | --- |
| Given | a number stage writing into `episode` and a record for `Show.mkv` |
| When | the stage is applied |
| Then | `episode` is unset |

## `NUM-030` A date field cannot be read

| Step | Statement |
| --- | --- |
| Given | a number stage reading from `mtime` and a record for `Show 12.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `number` |
