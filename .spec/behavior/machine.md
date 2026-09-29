# Machine

The machine decides when the settled files of a unit are handed over as a batch. It is told what happened to files and what time it is, and answers with the batches that are ready, so it can be run and tested without a filesystem or a clock.

## Includes

- `tests/machine.rs`

## `WCH-001` A unit is handed over once it has been quiet for the window

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `a.mkv` and `b.mkv` of the folder `Show` settled at 0 minutes |
| When | the machine is asked at 5 minutes |
| Then | one batch of `Show/a.mkv` and `Show/b.mkv` is ready |

## `WCH-002` Nothing is handed over before the window has passed

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` settled at 0 minutes |
| When | the machine is asked at 4 minutes 59 seconds |
| Then | nothing is ready |

## `WCH-003` A file settling in the unit restarts the wait

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` settled at 0 minutes and `Show/b.mkv` at 3 minutes |
| When | the machine is asked at 5 minutes and at 8 minutes |
| Then | nothing is ready at 5 minutes and both files at 8 |

## `WCH-004` Units are handed over apart

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `A/a.mkv` settled at 0 minutes and `B/b.mkv` at 3 minutes |
| When | the machine is asked at 5 minutes |
| Then | only the batch of `A` is ready |

## `WCH-005` The maximum wait closes a batch that never goes quiet

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and a file settling in `Show` every 4 minutes from 0 |
| When | the machine is asked at 30 minutes |
| Then | the batch is ready with the files so far |

## `WCH-006` Files come out in the order of their paths

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/b.mkv`, `Show/c.mkv` and `Show/a.mkv` settled in that order |
| When | the machine is asked when they are due |
| Then | the files are `a.mkv`, `b.mkv`, `c.mkv` |

## `WCH-007` A folder configuration never enters a batch

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/auto-renamer.toml` and `Show/a.mkv` settled |
| When | the machine is asked when they are due |
| Then | the batch holds `Show/a.mkv` only |

## `WCH-008` A batch over the batch limit is still handed over, for the service to judge

| Step | Statement |
| --- | --- |
| Given | the same watch with a batch limit of 2, and three files settled in `Show` |
| When | the machine is asked when they are due |
| Then | one batch of the three files is handed over |

## `WCH-010` A file being written holds its unit open

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` settled at 0 minutes and `Show/b.mkv` written to at 4 minutes |
| When | the machine is asked at 20 minutes |
| Then | nothing is ready |

## `WCH-011` A file that is gone leaves the batch and releases the hold

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` settled and `Show/b.mkv` written to and then gone |
| When | the machine is asked when they are due |
| Then | the batch holds `Show/a.mkv` |

## `WCH-012` A settled file written to again leaves the batch until it settles again

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` settled at 0 minutes and written to at 1 minute, and settled again at 2 minutes |
| When | the machine is asked at 7 minutes |
| Then | the batch holds `Show/a.mkv` |

## `WCH-013` A file found at start that was changed long ago is settled at once

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` found at start, last changed 10 minutes ago |
| When | the machine is asked 5 minutes later |
| Then | the batch holds `Show/a.mkv` |

## `WCH-014` A file found at start that was changed lately is settled after a window with no writes

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` found at start, last changed 1 minute ago |
| When | the machine is asked 3 minutes later and 4 minutes later |
| Then | nothing is ready at first, and the batch holds `Show/a.mkv` after |

## `WCH-015` A file found at start that is written again waits for its close

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` found at start changed 1 minute ago and written to 2 minutes later |
| When | the machine is asked 20 minutes later |
| Then | nothing is ready |

## `WCH-016` A hold that never resolves is dropped after the maximum wait

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, `Show/a.mkv` written to at 0 minutes and never settled, and `Show/b.mkv` settled at 1 minute |
| When | the machine is asked at 31 minutes |
| Then | the batch holds `Show/b.mkv` |

## `WCH-017` The next deadline says when to look again

| Step | Statement |
| --- | --- |
| Given | a machine with `Show/a.mkv` settled at 0 minutes and a window of 5 minutes |
| When | the next deadline is asked for |
| Then | it is at 5 minutes, and there is none when nothing is pending |

## `WCH-018` The unit of a file follows the unit of the watch

| Step | Statement |
| --- | --- |
| Given | a watch whose units are the folders below `Movies/*`, and `Movies/A/a.mkv` and `Movies/A/Subs/a.ass` settled |
| When | the machine is asked when they are due |
| Then | one batch for the unit `Movies/A` holds both |

## `WCH-019` A modification time in the future is taken as now

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` found at start with a modification time a year ahead |
| When | the machine is asked 5 minutes later |
| Then | the batch holds `Show/a.mkv` |


## `WCH-020` An extreme modification time does not stop the machine

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` found at start with the latest modification time a filesystem can hold |
| When | the machine is asked 5 minutes later |
| Then | the batch holds `Show/a.mkv` |


## `WCH-021` The deadline includes the end of a hold

| Step | Statement |
| --- | --- |
| Given | a watch with a batch window of 5 minutes, a maximum wait of 30 minutes, and each folder as a unit, and `Show/a.mkv` written to at 0 minutes |
| When | the next deadline is asked for |
| Then | it is at the maximum wait, 30 minutes |

## `WCH-022` A unit past the ceiling stops collecting files

| Step | Statement |
| --- | --- |
| Given | a watch with each folder as a unit and a batch limit of 100000, and 100005 files settled in `Show` |
| When | the batches that are ready are asked for |
| Then | the batch handed over names no more than 100001 of the files |
