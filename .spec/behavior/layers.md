# Layers

A folder can carry its own `auto-renamer.toml` for the exceptions of what lies in it. It comes from downloaded content, so it may only change how files are named, never where they come from, where they go or how they are grouped, and what it may set is checked when it is read.

## Includes

- `tests/layers.rs`

## `LAY-001` A folder configuration overrides a variable by name

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and a folder configuration with `show` Beta |
| When | the folder configuration is applied |
| Then | `show` is Beta |

## `LAY-002` The nearest folder configuration wins

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and folder configurations from the farthest to the nearest setting `show` to Beta and to Gamma |
| When | the folder configurations are applied |
| Then | `show` is Gamma |

## `LAY-003` A pipeline is replaced whole by name

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and a folder configuration whose pipeline `video` is a filter for `mp4` |
| When | the folder configuration is applied |
| Then | the pipeline `video` is that filter alone |

## `LAY-004` Pipelines of other names are kept

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and a second pipeline `photo` listed by the watch, and a folder configuration for `video` |
| When | the folder configuration is applied |
| Then | the pipeline `photo` is unchanged |

## `LAY-005` A folder configuration can set the file limit

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and a folder configuration with the file limit 20 |
| When | the folder configuration is applied |
| Then | the limit is 20 |

## `LAY-006` A folder configuration cannot set the source

| Step | Statement |
| --- | --- |
| Given | a folder configuration with `source` |
| When | the folder configuration is read |
| Then | it is refused, naming `source` |

## `LAY-007` A folder configuration cannot set the routes

| Step | Statement |
| --- | --- |
| Given | a folder configuration with `routes` |
| When | it is read |
| Then | it is refused, naming `routes` |
## `LAY-008` A folder configuration cannot set the unit

| Step | Statement |
| --- | --- |
| Given | a folder configuration with `unit` |
| When | the folder configuration is read |
| Then | it is refused, naming `unit` |

## `LAY-009` A folder configuration cannot switch off a dry run

| Step | Statement |
| --- | --- |
| Given | a folder configuration with `dry_run` |
| When | the folder configuration is read |
| Then | it is refused, naming `dry_run` |

## `LAY-011` An unknown key is refused by name

| Step | Statement |
| --- | --- |
| Given | a folder configuration with the key `colour` |
| When | the folder configuration is read |
| Then | it is refused, naming `colour` |

## `LAY-012` A folder configuration that is too large is refused

| Step | Statement |
| --- | --- |
| Given | a folder configuration of more than 64 KiB |
| When | the folder configuration is read |
| Then | it is refused as too large |

## `LAY-013` A mistake in a folder pipeline is refused with its name

| Step | Statement |
| --- | --- |
| Given | a folder configuration whose pipeline `video` holds an unknown stage |
| When | the folder configuration is read |
| Then | it is refused, naming the pipeline `video` |

## `LAY-015` A pipeline a folder defines is not run unless the watch lists it

| Step | Statement |
| --- | --- |
| Given | a watch `series` with the variable `show` set to `Alpha` and the pipeline `video` for `mkv` and a folder configuration that defines a pipeline `extra` |
| When | the folder configuration is applied |
| Then | the watch still lists only the pipeline `video` |

## `LAY-016` A folder cannot set the file limit above the ceiling

| Step | Statement |
| --- | --- |
| Given | a folder configuration with the file limit 100001 |
| When | the folder configuration is read |
| Then | it is refused, naming `max_files` |

## `LAY-017` A folder configuration cannot declare a target

| Step | Statement |
| --- | --- |
| Given | a folder configuration with a `target` table |
| When | it is read |
| Then | it is refused, naming `target` |
