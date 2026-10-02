# Playground

The playground checks, reads and writes a configuration, and simulates a watch on a virtual tree: the files the user put in its source are grouped into units as the watcher would, each unit is processed as one batch by the same processing the CLI runs, and the virtual files move by the verdicts.

## Includes

- `crates/wasm/tests/*.rs`

## `PLG-001` A simulation moves a virtual file by the pipeline

| Step | Statement |
| --- | --- |
| Given | a configuration whose watch formats the name as `{show}` and moves, the variable `show` Alpha, and a virtual source holding `Show/x.mkv` |
| When | the watch is simulated |
| Then | the virtual target holds `Show/Alpha.mkv` and the virtual source no longer holds `Show/x.mkv` |

## `PLG-002` A folder configuration in the virtual tree applies

| Step | Statement |
| --- | --- |
| Given | the same configuration and a virtual source holding `Show/x.mkv` and `Show/auto-renamer.toml` setting `show` to Beta |
| When | the watch is simulated |
| Then | the virtual target holds `Show/Beta.mkv` |

## `PLG-003` A folder configuration is not processed as a file

| Step | Statement |
| --- | --- |
| Given | the same configuration and a virtual source holding `Show/x.mkv` and `Show/auto-renamer.toml` |
| When | the watch is simulated |
| Then | nothing is said of `Show/auto-renamer.toml` and it stays in the virtual source |

## `PLG-004` Each unit is its own batch

| Step | Statement |
| --- | --- |
| Given | the same configuration taking each folder as a unit with `max_files` 1, and a virtual source holding `A/x.mkv` and `B/y.mkv` |
| When | the watch is simulated |
| Then | both are moved, since neither batch holds more than one file |

## `PLG-005` A taken name is refused

| Step | Statement |
| --- | --- |
| Given | the same configuration as the first, a virtual source holding `Show/x.mkv`, and a virtual target already holding `Show/Alpha.mkv` |
| When | the watch is simulated |
| Then | `Show/x.mkv` is reported as refused and stays in the virtual source |

## `PLG-006` A folder the move left empty is cleaned up

| Step | Statement |
| --- | --- |
| Given | a configuration whose watch takes each folder as a unit and whose route moves and cleans up, and a virtual source holding `Show/Season/x.mkv` |
| When | the watch is simulated |
| Then | the virtual source no longer holds the folder `Show/Season` |

## `PLG-007` A dry run moves nothing and says where a file would go

| Step | Statement |
| --- | --- |
| Given | the same configuration as the first as a dry run, and a virtual source holding `Show/x.mkv` |
| When | the watch is simulated |
| Then | `Show/x.mkv` is reported as previewed at the target's `Show/Alpha.mkv` and stays in the virtual source |

## `PLG-008` A configuration that is not valid is refused with why

| Step | Statement |
| --- | --- |
| Given | a global configuration whose pipeline names a stage that does not exist |
| When | it is checked |
| Then | it is refused with a reason naming the stage |

## `PLG-009` A folder configuration is checked as one

| Step | Statement |
| --- | --- |
| Given | a folder configuration setting a variable |
| When | it is checked as a folder configuration |
| Then | it is accepted |

## `PLG-010` A configuration read and written again parses the same

| Step | Statement |
| --- | --- |
| Given | a global configuration with a default, a pipeline and a watch |
| When | it is read and written again |
| Then | the text written is accepted and declares the same watch, pipeline and variables |

## `PLG-011` Writing a configuration drops its comments

| Step | Statement |
| --- | --- |
| Given | a global configuration with a comment |
| When | it is read and written again |
| Then | the text written holds no comment |

## `PLG-012` A virtual file's modification time is its time field

| Step | Statement |
| --- | --- |
| Given | a configuration whose watch formats the name as `{mtime:%Y}` and moves, and a virtual source holding `Show/x.mkv` modified in 2024 |
| When | the watch is simulated |
| Then | the virtual target holds `Show/2024.mkv` |

## `PLG-013` A stage the page is offered starts out accepted

| Step | Statement |
| --- | --- |
| Given | each stage the page is offered |
| When | a global configuration whose pipeline holds its example is checked |
| Then | it is accepted |

## `PLG-014` A simulated file carries every step of its planning

| Step | Statement |
| --- | --- |
| Given | a pipeline of a number and a format, and a file it claims |
| When | the watch is simulated |
| Then | the file's outcome lists the claim, the number with its episode, the format with its name, and the move that claims its plan, in that order |

## `PLG-015` A file a replaced pipeline planned is marked as such

| Step | Statement |
| --- | --- |
| Given | a folder configuration replacing the pipeline of its folder |
| When | the watch is simulated |
| Then | the outcome of a file in that folder is marked as planned by a replaced pipeline, and one elsewhere is not |

## `PLG-016` A built-in pipeline written out plans as the built-in does

| Step | Statement |
| --- | --- |
| Given | a watch routing `series-video`, and the same watch with the pipeline the page is offered for `series-video` written into the configuration |
| When | each is simulated over `Alpha/Alpha - 12.mkv` |
| Then | both move the file to the same place |

## `PLG-017` A refused file's steps end at the stage that refused it

| Step | Statement |
| --- | --- |
| Given | a pipeline of a number, a format naming a field no stage gives, and a format after it, and a file it claims |
| When | the watch is simulated |
| Then | the file's steps are the claim, the number and the format that refused it, which says why |
