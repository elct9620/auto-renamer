# Scan

A scan lists the regular files below a folder with the time each was last changed. It is how files that were there before the watcher started, or that arrived as a whole folder, are found. It never follows a link.

## Includes

- `tests/scan.rs`

## `SCN-001` The regular files below a folder are found with their modification times

| Step | Statement |
| --- | --- |
| Given | a folder holding `Show/a.mkv` and `Show/Subs/a.ass` |
| When | the folder is scanned |
| Then | both files are found, named relative to the source, each with its modification time |

## `SCN-002` A link to a file is not listed

| Step | Statement |
| --- | --- |
| Given | a folder holding a symbolic link `x.mkv` |
| When | the folder is scanned |
| Then | the link is not found |

## `SCN-003` A linked folder is not entered

| Step | Statement |
| --- | --- |
| Given | a folder holding a symbolic link `Linked` to a folder that holds `a.mkv` |
| When | the folder is scanned |
| Then | `a.mkv` is not found |

## `SCN-004` A folder that is not there gives nothing

| Step | Statement |
| --- | --- |
| Given | a folder that does not exist |
| When | the folder is scanned |
| Then | nothing is found |

## `SCN-005` A scan can stop and go on

| Step | Statement |
| --- | --- |
| Given | a folder holding five files in folders below it |
| When | a scan of it looks at three entries at a time until it is finished |
| Then | every file is found once |

## `SCN-006` A folder a scan opens counts as an entry looked at

| Step | Statement |
| --- | --- |
| Given | a folder holding 100 empty folders |
| When | a scan of it looks at 110 entries |
| Then | the scan is not finished |

## `SCN-007` A scan tells of each folder before it reads it

| Step | Statement |
| --- | --- |
| Given | a folder holding `A/1.mkv` and `A/B/2.mkv` |
| When | the folder is scanned by someone who is told of each folder entered |
| Then | they are told of `A` before `A/1.mkv` is found and of `A/B` before `A/B/2.mkv` is found |
