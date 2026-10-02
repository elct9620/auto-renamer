# Config

The configuration says which folders are watched, where their files go, and by which pipelines. It comes from a file the operator owns, and everything wrong with it is refused when it is read, so a running watcher never meets a configuration it cannot follow.

## Includes

- `tests/config.rs`

## `CFG-001` A watch reads its source, target and pipelines

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video` and a watch `series` for `/downloads` into `/library` listing `video` |
| When | the configuration is read |
| Then | the watch has that source and target and the pipeline `video` |

## `CFG-002` The default gives every watch the pipelines it lists

| Step | Statement |
| --- | --- |
| Given | a configuration whose default lists `video` and a watch `series` that lists none |
| When | the configuration is read |
| Then | the watch has the pipeline `video` |

## `CFG-003` A watch overrides the default

| Step | Statement |
| --- | --- |
| Given | a configuration whose default lists `video` and a watch that lists `photo` |
| When | the configuration is read |
| Then | the watch has only the pipeline `photo` |

## `CFG-004` Variables of the default and the watch are merged by name

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has `show` and `year` and a watch with `show` |
| When | the configuration is read |
| Then | the watch has the `show` of the watch and the `year` of the default |

## `CFG-005` A quiet period is five minutes unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video` and a watch `series` for `/downloads` into `/library` listing `video` |
| When | the configuration is read |
| Then | the quiet period is five minutes |

## `CFG-006` A maximum wait is thirty minutes unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video` and a watch `series` for `/downloads` into `/library` listing `video` |
| When | the configuration is read |
| Then | the maximum wait is thirty minutes |

## `CFG-007` A batch is limited to a thousand files unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video` and a watch `series` for `/downloads` into `/library` listing `video` |
| When | the configuration is read |
| Then | the limit is 1000 files |

## `CFG-008` A dry run is off unless set

| Step | Statement |
| --- | --- |
| Given | a configuration with a pipeline `video` and a watch `series` for `/downloads` into `/library` listing `video` |
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

## `CFG-014` A target inside its own source is refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` with the target `/downloads/library` |
| When | the configuration is read |
| Then | the configuration is refused, naming `target` |

## `CFG-015` A target that is its own source is refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` with the target `/downloads` |
| When | the configuration is read |
| Then | the configuration is refused, naming `target` |

## `CFG-016` Two watches whose sources overlap are refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` and a watch for `/downloads/movies` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-017` A target inside the source of another watch is refused

| Step | Statement |
| --- | --- |
| Given | a watch for `/downloads` into `/library` and a watch for `/library/incoming` into `/other` |
| When | the configuration is read |
| Then | the configuration is refused, naming `source` |

## `CFG-018` A watch naming a pipeline that is not defined is refused

| Step | Statement |
| --- | --- |
| Given | a watch that lists `missing` |
| When | the configuration is read |
| Then | the configuration is refused, naming `pipelines` |

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

## `CFG-021` A watch without a target renames in place

| Step | Statement |
| --- | --- |
| Given | a watch that has a source and no target |
| When | the configuration is read |
| Then | the watch has no target |

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

## `CFG-030` A pipeline without an effect stage is warned about

| Step | Statement |
| --- | --- |
| Given | a watch listing a pipeline whose stages only rewrite the plan |
| When | the configuration is read |
| Then | the warnings name the watch and the pipeline |

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
