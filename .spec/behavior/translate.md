# Translate

The filesystem reports what happened in its own words, and only some of it matters. A translation says what happened to which file of the source, and asks for a folder to be scanned when a whole folder arrived at once and its files were never reported one by one.

## Includes

- `tests/translate.rs`

## `TRN-001` A write that was closed settles the file

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that `/s/Show/a.mkv` was closed after writing |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is settled |

## `TRN-002` A file moved in settles

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the file `/s/Show/a.mkv` was moved to there |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is settled |

## `TRN-003` A folder moved in is scanned

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the folder `/s/Show` was moved to there |
| When | the notification is translated |
| Then | the folder `/s/Show` is to be scanned |

## `TRN-004` A created file is being written

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the file `/s/Show/a.mkv` was created |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is being written |

## `TRN-005` A created folder is scanned

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the folder `/s/Show` was created |
| When | the notification is translated |
| Then | the folder `/s/Show` is to be scanned |

## `TRN-006` A change of data is a write in progress

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the data of the file `/s/Show/a.mkv` changed |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is being written |

## `TRN-007` A file moved out is gone

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that `/s/Show/a.mkv` was moved from there |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is gone |

## `TRN-008` A removed file is gone

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that `/s/Show/a.mkv` was removed |
| When | the notification is translated |
| Then | the file `Show/a.mkv` is gone |

## `TRN-009` A rename inside the source is gone at the old name and settled at the new

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that `/s/a.mkv` was renamed to `/s/b.mkv` |
| When | the notification is translated |
| Then | `a.mkv` is gone and `b.mkv` is settled |

## `TRN-010` Paths outside the source are ignored

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification about `/elsewhere/a.mkv` |
| When | the notification is translated |
| Then | nothing is reported |

## `TRN-011` A change to a folder is not a write of a file

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that the data of the folder `/s/Show` changed |
| When | the notification is translated |
| Then | nothing is reported |

## `TRN-012` Notifications of other kinds are ignored

| Step | Statement |
| --- | --- |
| Given | the source `/s` and a notification that `/s/Show/a.mkv` was opened for reading |
| When | the notification is translated |
| Then | nothing is reported |
