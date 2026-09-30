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

## `RUN-015` A file renamed in place does not hold the folder

| Step | Statement |
| --- | --- |
| Given | a running watcher without a target, a long maximum wait, and a file renamed in place |
| When | another file is written into the same folder |
| Then | it is renamed too, well before the maximum wait |

## `RUN-016` A pipeline that names its own result again is stopped

| Step | Statement |
| --- | --- |
| Given | a running watcher without a target whose pipeline puts a prefix before the name |
| When | a file is written into the source |
| Then | the file is renamed a few times and then left, with the error output saying it was renamed too many times in a row |

## `RUN-017` A configuration file inside a source is refused

| Step | Statement |
| --- | --- |
| Given | a configuration file kept inside the source of its own watch |
| When | the watcher is started |
| Then | it stops with a configuration error naming the source |

## `RUN-018` A configuration file reached through a link to a source is refused

| Step | Statement |
| --- | --- |
| Given | a watch whose source is a link to the folder holding the configuration file |
| When | the watcher is started |
| Then | it stops with a configuration error naming the source |

## `RUN-019` A file written into a folder that has just appeared is not missed

| Step | Statement |
| --- | --- |
| Given | a running watcher and a folder holding many folders outside the source |
| When | the folder is moved into the source and a file is written into each folder under it at once |
| Then | every file appears in the target |

## `RUN-020` A folder that cannot be watched is named

| Step | Statement |
| --- | --- |
| Given | the program running and a folder it is not permitted to read |
| When | the folder is moved into the source |
| Then | its error output says the folder is not watched |

## `RUN-021` A folder renamed inside the source is processed under its new name

| Step | Statement |
| --- | --- |
| Given | a running watcher and a folder made in the source while it runs, holding a file |
| When | the folder is renamed inside the source before its batch is handed over |
| Then | the file appears in the target under the new name of the folder |

## `RUN-022` A batch is not handed over before the scan that finds its files is finished

| Step | Statement |
| --- | --- |
| Given | a source holding 600 files of one unit, last changed before the batch window, and a pipeline that numbers the files of a batch |
| When | the watcher starts |
| Then | all 600 files appear in the target, numbered as one batch |

## `RUN-023` A stop is answered between batches

| Step | Statement |
| --- | --- |
| Given | a watcher with 3000 batches ready at once |
| When | it is told to stop as the first file reaches the target |
| Then | it stops with files still in the source |

## `RUN-024` A file written into a folder that was there at start is processed

| Step | Statement |
| --- | --- |
| Given | a source holding the folder `Show/Season 01` when the watcher starts |
| When | a file is written into `Show/Season 01` |
| Then | the file appears in the target |

## `RUN-025` A folder that cannot be watched at start stops the watcher

| Step | Statement |
| --- | --- |
| Given | a source holding a folder the program is not permitted to read |
| When | the program is started |
| Then | it stops with an error naming the folder |

## `RUN-026` Lost notifications are made up for by scanning again

| Step | Statement |
| --- | --- |
| Given | the program moving a batch of 50000 files, which leaves no room in its queue |
| When | files are written into the source while it does |
| Then | its error output says notifications were lost, and the files appear in the target |

## `RUN-027` A watcher with nothing to do uses next to no CPU

| Step | Statement |
| --- | --- |
| Given | the program running after it moved a file, with nothing left waiting |
| When | three seconds pass |
| Then | it has used less than a tenth of a second of CPU in them |
