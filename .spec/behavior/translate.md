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

## `TRN-013` A write that was closed rewrites the file

| Step | Statement |
| --- | --- |
| Given | a notification that `/c/config.toml` was closed after writing |
| When | it is asked whether it rewrites `/c/config.toml` |
| Then | it does |

## `TRN-014` Another file moved over the file rewrites it

| Step | Statement |
| --- | --- |
| Given | a notification that a file was moved to `/c/config.toml` |
| When | it is asked whether it rewrites `/c/config.toml` |
| Then | it does |

## `TRN-015` A read does not rewrite the file

| Step | Statement |
| --- | --- |
| Given | notifications that `/c/config.toml` was opened, read, and closed after reading |
| When | each is asked whether it rewrites `/c/config.toml` |
| Then | none does |

## `TRN-016` A notification about another file does not rewrite it

| Step | Statement |
| --- | --- |
| Given | a notification that `/c/other.toml` was closed after writing |
| When | it is asked whether it rewrites `/c/config.toml` |
| Then | it does not |

## `TRN-017` A notification that something changed is passed on

| Step | Statement |
| --- | --- |
| Given | the handler given to the filesystem watcher |
| When | it is handed a notification that a file was closed after writing |
| Then | the notification reaches the runner |

## `TRN-018` A notification that nothing changed is dropped

| Step | Statement |
| --- | --- |
| Given | the handler given to the filesystem watcher |
| When | it is handed notifications that a file was opened, closed after reading, and had its attributes changed |
| Then | none of them reaches the runner |

## `TRN-019` An error is passed on

| Step | Statement |
| --- | --- |
| Given | the handler given to the filesystem watcher |
| When | it is handed an error of the filesystem watcher |
| Then | the error reaches the runner |

## `TRN-020` Whatever is translated or rewrites the configuration is passed on

| Step | Statement |
| --- | --- |
| Given | the handler given to the filesystem watcher and a notification of every kind a translation or a rewrite answers to |
| When | it is handed each of them |
| Then | each reaches the runner |
