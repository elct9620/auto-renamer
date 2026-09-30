# Next

A next stage fills a field with the number after the highest one already in the target. It reads the file names of the folder the plan is now in, keeps only those written the way its template writes them, and counts the number in the place of the field it fills. Numbers it has already handed out in the same batch count too, so records of one batch get consecutive numbers.

## Includes

- `tests/next.rs`

## `NXT-001` A field that is already set is left alone

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` and a record whose `episode` is the number 7 |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `NXT-002` The highest number in the target is followed

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder holding `Show s01e01.mkv` and `Show s01e02.mkv` |
| When | the stage is applied |
| Then | `episode` is 3 |

## `NXT-003` The first number is 1

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder that is empty |
| When | the stage is applied |
| Then | `episode` is 1 |

## `NXT-004` Names not written that way do not count

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder holding `Show s01e05.mkv` and `Other s01e09.mkv` and `notes.txt` |
| When | the stage is applied |
| Then | `episode` is 6 |

## `NXT-005` The extension is not compared

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder holding `Show s01e01.mkv` and `Show s01e04.mp4` |
| When | the stage is applied |
| Then | `episode` is 5 |

## `NXT-006` A known field stands for its value

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder holding `Show s01e02.mkv` and `Show s02e09.mkv` |
| When | the stage is applied |
| Then | `episode` is 3 |

## `NXT-007` A number padded wider than the template is read back

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, and a target folder holding `Show s01e007.mkv` |
| When | the stage is applied |
| Then | `episode` is 8 |

## `NXT-008` A field the template needs and the record lacks is refused

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `season` 1 and no `show`, and a target folder that is empty |
| When | the stage is applied |
| Then | the record is refused, naming the stage `next` |

## `NXT-009` Only the folder of the plan is read

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}` and a record for `Series/Show/x.mkv` with `show` Show and `season` 1, a target folder `Series/Show` that is empty, and a target folder `Series/Other` holding `Show s01e09.mkv` |
| When | the stage is applied |
| Then | `episode` is 1 |

## `NXT-010` Records of one batch get consecutive numbers

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}`, two records for `Series/Show/a.mkv` and `Series/Show/b.mkv` with `show` Show and `season` 1, and a target folder that is empty |
| When | the stage is applied to both records in turn |
| Then | the episodes are 1 and 2 |

## `NXT-012` Folders are counted apart

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show} s{season:02}e{episode:02}`, a record for `Series/A/x.mkv` and a record for `Series/B/y.mkv` both with `show` Show and `season` 1, and empty target folders |
| When | the stage is applied to both records in turn |
| Then | the episodes are both 1 |

## `NXT-013` An optional part whose field is missing is left out of what is counted

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show}[ {tag}] {episode}`, a record for `Series/Show/x.mkv` with `show` Show and no `tag`, and a target folder holding `Show 5.mkv` and `Show a 9.mkv` |
| When | the stage is applied |
| Then | `episode` is 6 |


## `NXT-014` An optional part whose field is present must appear in what is counted

| Step | Statement |
| --- | --- |
| Given | a next stage filling `episode` like `{show}[ {tag}] {episode}`, a record for `Series/Show/x.mkv` with `show` Show and `tag` a, and a target folder holding `Show 5.mkv` and `Show a 9.mkv` |
| When | the stage is applied |
| Then | `episode` is 10 |

## `NXT-015` A batch asks the target for a folder once

| Step | Statement |
| --- | --- |
| Given | a batch of files of two shows, all numbered into one folder of the target |
| When | the batch is numbered |
| Then | the target is asked for the files of that folder once |

## `NXT-016` Numbering stays quick against a folder of many files

| Step | Statement |
| --- | --- |
| Given | a target folder of 100000 files written the way the template writes them |
| When | a batch of 1000 files is numbered into it |
| Then | the batch is numbered within five seconds |
