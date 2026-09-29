# Record

A record is one file as it moves through a pipeline: a plan path together with named fields, all known from the path and the modification time before any stage runs.

## Includes

- `tests/record.rs`

## `REC-001` The main file name is a field

| Step | Statement |
| --- | --- |
| Given | a file `Series/Alpha/Alpha - 12.mkv` |
| When | a record is created for it |
| Then | the field `name` is `Alpha - 12` |

## `REC-002` The extension is a field

| Step | Statement |
| --- | --- |
| Given | a file `Series/Alpha/Alpha - 12.mkv` |
| When | a record is created for it |
| Then | the field `ext` is `mkv` |

## `REC-003` Only the last dot starts the extension

| Step | Statement |
| --- | --- |
| Given | a file `Show 27.cht.ass` |
| When | a record is created for it |
| Then | the field `name` is `Show 27.cht` |

## `REC-004` A name without an extension keeps every character

| Step | Statement |
| --- | --- |
| Given | a file `README` with no dot in its name |
| When | a record is created for it |
| Then | the field `ext` is empty |

## `REC-005` A leading dot is not an extension

| Step | Statement |
| --- | --- |
| Given | a file `.hidden` |
| When | a record is created for it |
| Then | the field `name` is `.hidden` |

## `REC-006` The parent folder name is a field

| Step | Statement |
| --- | --- |
| Given | a file `Series/Alpha/Season 03/Alpha - 12.mkv` |
| When | a record is created for it |
| Then | the field `dir` is `Season 03` |

## `REC-007` The folder path is a field

| Step | Statement |
| --- | --- |
| Given | a file `Series/Alpha/Season 03/Alpha - 12.mkv` |
| When | a record is created for it |
| Then | the field `path` is `Series/Alpha/Season 03` |

## `REC-008` A file at the source root has no folder

| Step | Statement |
| --- | --- |
| Given | a file `loose.mkv` directly under the source |
| When | a record is created for it |
| Then | the fields `dir` and `path` are empty |

## `REC-009` The modification time is a date field

| Step | Statement |
| --- | --- |
| Given | a file modified at 2026-09-27 12:00:00 UTC |
| When | a record is created for it |
| Then | the field `mtime` is that date |

## `REC-010` The plan starts as the relative path

| Step | Statement |
| --- | --- |
| Given | a file `Series/Alpha/Alpha - 12.mkv` |
| When | a record is created for it |
| Then | the plan is `Series/Alpha/Alpha - 12.mkv` |

## `REC-011` Variables become fields

| Step | Statement |
| --- | --- |
| Given | a record and the variable `show` set to `Alpha` |
| When | the variables are added to the record |
| Then | the field `show` is `Alpha` |

## `REC-012` A variable never replaces a built-in field

| Step | Statement |
| --- | --- |
| Given | a record for `Alpha - 12.mkv` and a variable `name` set to `other` |
| When | the variables are added to the record |
| Then | the field `name` is still `Alpha - 12` |

## `REC-013` Writing the name rewrites the file name of the plan

| Step | Statement |
| --- | --- |
| Given | a record for `Series/Alpha/x.mkv` |
| When | the field `name` is written as `y` |
| Then | the plan is `Series/Alpha/y.mkv` |


## `REC-014` Writing the extension rewrites the file name of the plan

| Step | Statement |
| --- | --- |
| Given | a record for `Series/Alpha/x.mkv` |
| When | the field `ext` is written as `mp4` |
| Then | the plan is `Series/Alpha/x.mp4` |


## `REC-015` A name that cannot be a file name leaves the plan alone

| Step | Statement |
| --- | --- |
| Given | a record for `Series/Alpha/x.mkv` |
| When | the field `name` is written as `a/b` |
| Then | the plan is still `Series/Alpha/x.mkv` |

## `REC-016` A record remembers the path it was made from

| Step | Statement |
| --- | --- |
| Given | a record for `Series/Alpha/x.mkv` whose file name has since been rewritten |
| When | the origin is asked for |
| Then | it is `Series/Alpha/x.mkv` |
