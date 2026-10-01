# Batch

A batch is planned as a whole: files are put in name order, each is claimed by the first pipeline whose leading filters accept it, and the stages of every pipeline run over all of its files one stage at a time. One file being refused never stops the others.

## Includes

- `tests/batch.rs`

## `BAT-001` Files are planned in the order of their names

| Step | Statement |
| --- | --- |
| Given | files `b.mkv`, `c.mkv` and `a.mkv` given in that order and a pipeline that keeps every file |
| When | the batch is planned |
| Then | the answers come in the order `a.mkv`, `b.mkv`, `c.mkv` |

## `BAT-002` A file goes to the first pipeline whose filter accepts it

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` for `mkv` and a pipeline `subtitle` for `ass`, and the files `a.mkv` and `a.ass` |
| When | the batch is planned |
| Then | `a.mkv` is planned by `video` and `a.ass` by `subtitle` |

## `BAT-003` A file that every pipeline turns away is unclaimed

| Step | Statement |
| --- | --- |
| Given | a pipeline for `mkv` and the file `a.nfo` |
| When | the batch is planned |
| Then | `a.nfo` is unclaimed |

## `BAT-004` A pipeline with no leading filter claims what is left

| Step | Statement |
| --- | --- |
| Given | a pipeline for `mkv` followed by a pipeline that starts with a format stage, and the files `a.mkv` and `a.nfo` |
| When | the batch is planned |
| Then | `a.nfo` is claimed by the second pipeline |

## `BAT-005` A filter after the first stage excludes the file for good

| Step | Statement |
| --- | --- |
| Given | a pipeline that formats and then filters for `mkv`, followed by a pipeline that keeps every file, and the file `a.nfo` |
| When | the batch is planned |
| Then | `a.nfo` is excluded and not offered to the second pipeline |

## `BAT-006` A refused file leaves the others planned

| Step | Statement |
| --- | --- |
| Given | a pipeline that formats with the field `episode` and the files `a.mkv` with `episode` and `b.mkv` without |
| When | the batch is planned |
| Then | `a.mkv` is planned and `b.mkv` is refused |

## `BAT-007` Stages run over the whole batch in the order of the names

| Step | Statement |
| --- | --- |
| Given | a pipeline whose stage fills `episode` with the next number, an empty target, and the files `b.mkv` and `a.mkv` |
| When | the batch is planned |
| Then | `a.mkv` gets 1 and `b.mkv` gets 2 |

## `BAT-008` A pipeline stops before the first effect stage

| Step | Statement |
| --- | --- |
| Given | a pipeline that formats and then moves, and the file `a.mkv` |
| When | the batch is planned |
| Then | `a.mkv` is planned as the formatted name |

## `BAT-009` Numbers handed out are kept across the pipelines of a batch

| Step | Statement |
| --- | --- |
| Given | two pipelines each filling `episode` with the next number, an empty target, and the files `x.mkv` for the first and `y.mp4` for the second |
| When | the batch is planned |
| Then | `x.mkv` gets 1 and `y.mp4` gets 2 |

## `BAT-011` One file is a batch of one, each stage running on the result of the last

| Step | Statement |
| --- | --- |
| Given | a pipeline that reads the episode and then formats the name, and the file `Show - 12.mkv` alone |
| When | the batch is planned |
| Then | `Show - 12.mkv` is planned as `e12.mkv` |

## `BAT-012` Planning tells each step of every file

| Step | Statement |
| --- | --- |
| Given | a pipeline of a filter, a number and a format, and a file it claims |
| When | the batch is planned and its steps are asked for |
| Then | the file is told once claimed, then after the number with its episode, then after the format with its name |

## `BAT-013` A stopped file is told no further than the stage that stopped it

| Step | Statement |
| --- | --- |
| Given | a pipeline whose format refuses a file missing a field |
| When | the batch is planned and its steps are asked for |
| Then | the last step told for the file is the format, holding the refusal |
