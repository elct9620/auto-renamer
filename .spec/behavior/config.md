# Config

The configuration says which folders are watched, which routes take their files, and the targets those routes move them to. It comes from a file the operator owns, and everything wrong with it is refused when it is read, so a running watcher never meets a configuration it cannot follow.

## Includes

- `tests/config.rs`

## `CFG-001` A watch reads its source and routes

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video`, a target `library` at `/library`, and a watch `series` for `/downloads` with a route of `video` into `library` |
| When | the configuration is read |
| Then | the watch has that source and one route of `video` into `/library` |
## `CFG-002` The default gives every watch the routes it lists

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has a route of `video` and a watch `series` that has none |
| When | the configuration is read |
| Then | the watch has the route of `video` |
## `CFG-003` A watch overrides the default

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has a route of `video` and a watch with a route of `photo` |
| When | the configuration is read |
| Then | the watch has only the route of `photo` |
## `CFG-004` Variables of the default and the watch are merged by name

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has `show` and `year` and a watch with `show` |
| When | the configuration is read |
| Then | the watch has the `show` of the watch and the `year` of the default |

## `CFG-005` A quiet period is five minutes unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video`, a target `library` at `/library`, and a watch `series` for `/downloads` with a route of `video` into `library` |
| When | the configuration is read |
| Then | the quiet period is five minutes |

## `CFG-006` A maximum wait is thirty minutes unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video`, a target `library` at `/library`, and a watch `series` for `/downloads` with a route of `video` into `library` |
| When | the configuration is read |
| Then | the maximum wait is thirty minutes |

## `CFG-007` A batch is limited to a thousand files unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video`, a target `library` at `/library`, and a watch `series` for `/downloads` with a route of `video` into `library` |
| When | the configuration is read |
| Then | the limit is 1000 files |

## `CFG-008` A dry run is off unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video`, a target `library` at `/library`, and a watch `series` for `/downloads` with a route of `video` into `library` |
| When | the configuration is read |
| Then | the watch is not a dry run |

## `CFG-009` A duration is written in seconds, minutes or hours

| Step | Statement |
| --- | --- |
| Given | a watch with the quiet period `90s` and the maximum wait `2h` |
| When | the configuration is read |
| Then | the quiet period is 90 seconds and the maximum wait is 2 hours |

## `CFG-010` A duration without a unit is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the quiet period `5` |
| When | the configuration is read |
| Then | the configuration is refused, naming `quiet` |

## `CFG-011` A maximum wait shorter than the quiet period is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the quiet period `10m` and the maximum wait `5m` |
| When | the configuration is read |
| Then | the configuration is refused, naming `max_wait` |

## `CFG-012` A source that is not an absolute path is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the source `downloads` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-013` A watch without a source is refused

| Step | Statement |
| --- | --- |
| Given | a watch that has no source |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-014` A target inside a source is refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` and a target at `/downloads/library` |
| When | the configuration is read |
| Then | the configuration is refused, naming `path` |
## `CFG-015` A target that is a source is refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` and a target at `/downloads` |
| When | the configuration is read |
| Then | the configuration is refused, naming `path` |
## `CFG-016` Two watches whose sources overlap are refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` and a watch for `/downloads/movies` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-017` A source inside a target is refused

| Step | Statement |
| --- | --- |
| Given | a target at `/library` and a watch for `/library/incoming` |
| When | the configuration is read |
| Then | the configuration is refused, naming `path` |
## `CFG-018` A route naming a pipeline that is not defined is refused

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `missing` |
| When | the configuration is read |
| Then | the configuration is refused, naming `routes` |
## `CFG-019` A mistake in a pipeline is refused with its name

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` whose stage list holds an unknown stage |
| When | the configuration is read |
| Then | the configuration is refused, naming the pipeline `video` |

## `CFG-020` An unknown key is refused by name

| Step | Statement |
| --- | --- |
| Given | a watch with the key `colour` |
| When | the configuration is read |
| Then | the configuration is refused, naming `colour` |

## `CFG-021` A route without a move renames in place

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `video` that names no target |
| When | the configuration is read |
| Then | the route has no target |
## `CFG-022` A file limit must be a positive whole number

| Step | Statement |
| --- | --- |
| Given | a watch with the file limit 0 |
| When | the configuration is read |
| Then | the configuration is refused, naming `max_files` |

## `CFG-023` A document that is not TOML is refused

| Step | Statement |
| --- | --- |
| Given | a document that is not TOML |
| When | the configuration is read |
| Then | the configuration is refused |

## `CFG-024` A variable is text or a whole number

| Step | Statement |
| --- | --- |
| Given | a watch whose variable `show` is a list |
| When | the configuration is read |
| Then | the configuration is refused, naming `show` |

## `CFG-025` A dry run is switched on by the watch

| Step | Statement |
| --- | --- |
| Given | a watch with `dry_run` on |
| When | the configuration is read |
| Then | the watch is a dry run |

## `CFG-027` The default cannot hold a source or a target

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has a `source` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |


## `CFG-028` A file limit above the ceiling is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the file limit 100001 |
| When | the configuration is read |
| Then | the configuration is refused, naming `max_files` |

## `CFG-029` The ceiling itself is accepted

| Step | Statement |
| --- | --- |
| Given | a watch with the file limit 100000 |
| When | the configuration is read |
| Then | the watch has the limit 100000 |

## `CFG-031` A source or target written with `.` or `..` is refused

| Step | Statement |
| --- | --- |
| Given | a watch whose source is `/downloads/../library` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-032` Sources that overlap once their real paths are known are refused

| Step | Statement |
| --- | --- |
| Given | two watches with the sources `/link` and `/downloads`, where `/link` really is `/downloads/sub` |
| When | the paths are checked with their real locations |
| Then | the check is refused, naming `source` |

## `CFG-033` A configuration file inside a source once real paths are known is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the source `/link`, where `/link` really is `/etc/auto-renamer`, and the configuration file `/etc/auto-renamer/config.toml` |
| When | the paths are checked with their real locations |
| Then | the check is refused, naming `source` |

## `CFG-034` A route moving to a target that is not declared is refused

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `video` into `missing` |
| When | the configuration is read |
| Then | the configuration is refused, naming `routes` |

## `CFG-035` A target without a path is refused

| Step | Statement |
| --- | --- |
| Given | a target `library` with no `path` |
| When | the configuration is read |
| Then | the configuration is refused, naming `path` |

## `CFG-036` A route reads where its rejected files go

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `video` into `library` whose `rejected` moves into `conflict`, with no pipeline |
| When | the configuration is read |
| Then | the route's rejected route moves into `/conflict` and has no pipeline |

## `CFG-037` A rejected route has no rejected route of its own

| Step | Statement |
| --- | --- |
| Given | a watch with a route whose `rejected` holds a `rejected` of its own |
| When | the configuration is read |
| Then | the configuration is refused, naming `rejected` |

## `CFG-038` A watch reads the fields that make a group

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has `group = ["show", "episode"]` and a watch that says none |
| When | the configuration is read |
| Then | the watch groups files by `show` and `episode` |

## `CFG-039` A route can name a built-in pipeline without defining it

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `series-video` and no pipeline of that name |
| When | the configuration is read |
| Then | the route plans with the built-in `series-video` |

## `CFG-040` A pipeline defined with a built-in name replaces it

| Step | Statement |
| --- | --- |
| Given | a configuration defining `series-video` as a filter for `mkv` only, and a watch with a route of it |
| When | the configuration is read |
| Then | the route plans with the pipeline the configuration defines |

## `CFG-041` A target inside a source once real paths are known is refused

| Step | Statement |
| --- | --- |
| Given | a watch with the source `/downloads` and a target at `/link`, where `/link` really is `/downloads/library` |
| When | the paths are checked with their real locations |
| Then | the check is refused, naming `path` |

## `CFG-042` A watch takes the batch settings the default gives

| Step | Statement |
| --- | --- |
| Given | a default with `quiet`, `max_wait`, `max_files`, `dry_run` and `unit`, and a watch setting none of them |
| When | the configuration is read |
| Then | the watch has each value the default gives |
