# Runner

The runner is the watcher itself: it watches the sources, hands ready batches to be processed, and follows the configuration file as it changes. It runs until it is told to stop, and a broken configuration never stops it.

## Includes

- `tests/runner.rs`
- `tests/signal.rs`
- `tests/logging.rs`

## `RUN-001` A file dropped in the source ends up in the target

| Step | Statement |
| --- | --- |
| Given | a running watcher with a short batch window and a pipeline that moves |
| When | a file is written into the source |
| Then | the file appears in the target |

## `RUN-002` Files already there at start are processed

| Step | Statement |
| --- | --- |
| Given | a file already in the source |
| When | the watcher starts |
| Then | the file appears in the target |

## `RUN-003` A change to the configuration takes effect

| Step | Statement |
| --- | --- |
| Given | a running watcher whose pipeline keeps the name |
| When | the configuration is changed to rename the file to `renamed` and a file is written |
| Then | the file appears in the target as `renamed` |

## `RUN-004` A configuration that is not valid keeps the running one

| Step | Statement |
| --- | --- |
| Given | a running watcher |
| When | the configuration is changed to something that is not valid and a file is written |
| Then | the file is still processed by the earlier configuration |

## `RUN-005` A linked folder is not followed

| Step | Statement |
| --- | --- |
| Given | a running watcher and a symbolic link in the source to a folder outside it |
| When | a file is written into the folder outside the source |
| Then | the file stays where it was, the target stays empty and the program says nothing of the folder |

## `RUN-006` A termination signal stops the watcher cleanly

| Step | Statement |
| --- | --- |
| Given | the program running with a valid configuration |
| When | it is sent a termination signal |
| Then | it exits successfully |

## `RUN-007` An interrupt stops the watcher cleanly

| Step | Statement |
| --- | --- |
| Given | the program running with a valid configuration |
| When | it is sent an interrupt |
| Then | it exits successfully |

## `RUN-008` A folder moved into the source is processed with what it holds

| Step | Statement |
| --- | --- |
| Given | a running watcher and a folder holding a file outside the source |
| When | the folder is moved into the source |
| Then | the file appears in the target |

## `RUN-009` A configuration replaced by another file takes effect

| Step | Statement |
| --- | --- |
| Given | a running watcher whose pipeline keeps the name |
| When | another file is moved over the configuration file, renaming to `replaced`, and a file is written |
| Then | the file appears in the target as `replaced` |

## `RUN-010` A dry run reports where a file would go

| Step | Statement |
| --- | --- |
| Given | the program running as a dry run |
| When | a file is written into the source |
| Then | its error output says where the file would go, and the file stays |

## `RUN-011` A refused file is reported with why

| Step | Statement |
| --- | --- |
| Given | the program running with a pipeline that needs a variable that is not set |
| When | a file is written into the source |
| Then | its error output says the file was refused, and the file stays |

## `RUN-012` A file no pipeline claims is reported

| Step | Statement |
| --- | --- |
| Given | the program running with a pipeline for `mkv` only |
| When | a `nfo` file is written into the source |
| Then | its error output says the file was left |

## `RUN-013` A configuration is read again once for one change

| Step | Statement |
| --- | --- |
| Given | the program running and its configuration changed once |
| When | some seconds pass with nothing else changing |
| Then | its error output says the configuration was read again exactly once |

## `RUN-014` A hangup signal reads the configuration again

| Step | Statement |
| --- | --- |
| Given | the program running whose pipeline keeps the name |
| When | the configuration is changed without a notification and the program is sent a hangup signal, and a file is written |
| Then | the file appears in the target as `renamed` |
