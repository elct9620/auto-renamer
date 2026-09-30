# Service

Processing a batch is what a ready batch of settled files turns into: the files are read, the folder configurations that apply are found, the batch is planned through the pipelines, and the effects are run on what was planned. Each file ends up moved, left where it was, or refused, and the answer says which and why.

## Includes

- `tests/service.rs`

## `SVC-001` The files of a batch are planned and moved to the target

| Step | Statement |
| --- | --- |
| Given | a watch with a pipeline that formats the name as `{show}` and moves, the variable `show` Alpha, and the file `Show/x.mkv` in the source |
| When | the batch is processed |
| Then | the target holds `Show/Alpha.mkv` and the source no longer holds `Show/x.mkv` |

## `SVC-002` A folder configuration applies to the files under it

| Step | Statement |
| --- | --- |
| Given | the same watch and a folder configuration in `Show` setting `show` to Beta |
| When | the batch is processed |
| Then | the target holds `Show/Beta.mkv` |

## `SVC-003` A nearer folder configuration beats a farther one

| Step | Statement |
| --- | --- |
| Given | the same watch, a folder configuration in the source setting `show` to Beta, and one in `Show` setting it to Gamma |
| When | the batch is processed |
| Then | the target holds `Show/Gamma.mkv` |

## `SVC-004` A file no pipeline claims stays where it is

| Step | Statement |
| --- | --- |
| Given | the same watch and the file `Show/notes.nfo` in the source |
| When | the batch is processed |
| Then | the file is reported as unclaimed and is still in the source |

## `SVC-005` A refused file stays and says why

| Step | Statement |
| --- | --- |
| Given | the same watch with the variable `show` missing, and the file `Show/x.mkv` |
| When | the batch is processed |
| Then | the file is reported as refused with the reason and is still in the source |

## `SVC-006` A dry run moves nothing and reports where each file would go

| Step | Statement |
| --- | --- |
| Given | the same watch as a dry run and the file `Show/x.mkv` |
| When | the batch is processed |
| Then | the file is reported as previewed and is still in the source |

## `SVC-007` A batch over the limit stays as it is

| Step | Statement |
| --- | --- |
| Given | the same watch with a batch limit of 1 and two files in `Show` |
| When | the batch is processed |
| Then | both files are reported as left because the batch is too large, and both are still in the source |

## `SVC-008` A folder configuration can raise the limit for its own batch

| Step | Statement |
| --- | --- |
| Given | the same watch with a batch limit of 1, a folder configuration in `Show` setting the limit to 5, and two files in `Show` |
| When | the batch is processed |
| Then | both files are moved |

## `SVC-009` A file that vanished before the batch was processed is skipped

| Step | Statement |
| --- | --- |
| Given | the same watch and a file `Show/x.mkv` that is not in the source |
| When | the batch is processed |
| Then | the file is reported as skipped as missing |

## `SVC-011` The next number follows what the target already holds

| Step | Statement |
| --- | --- |
| Given | a watch whose pipeline fills `episode` with the next number like `{show} s01e{episode:02}` and formats the name so, a target holding `Alpha s01e04.mkv` in `Show`, and the file `Show/x.mkv` |
| When | the batch is processed |
| Then | the target holds `Show/Alpha s01e05.mkv` |

## `SVC-012` A folder configuration that cannot be read is ignored and reported

| Step | Statement |
| --- | --- |
| Given | the same watch and a folder configuration in `Show` that is not valid |
| When | the batch is processed |
| Then | the watch is applied as it is, and the folder configuration is reported |

## `SVC-013` A file renamed in place again and again is left after a limit

| Step | Statement |
| --- | --- |
| Given | a watch without a target whose pipeline puts a prefix before the name, and a file processed again each time it was renamed |
| When | it has been renamed five times in a row and is processed once more |
| Then | it is refused, saying it was renamed too many times in a row, and it stays as it is |

## `SVC-014` A file that is left as it is forgets its renames

| Step | Statement |
| --- | --- |
| Given | the same watch, a file renamed four times in a row, and a pipeline that then leaves it as it is |
| When | it is processed with that pipeline and then renamed by the prefix pipeline five more times |
| Then | every one of those renames is done |

## `SVC-015` A file moved before a later effect failed is reported as moved, with the failure

| Step | Statement |
| --- | --- |
| Given | the same watch whose pipeline moves the file and then has an effect that fails, and a file in `Show` |
| When | the batch is processed |
| Then | the file is reported as moved to the target together with why the later effect failed |

## `SVC-016` A pipeline without an effect stage reports where each file would go

| Step | Statement |
| --- | --- |
| Given | the same watch whose pipeline only formats the name, and the file `Show/x.mkv` |
| When | the batch is processed |
| Then | the file is reported as previewed at its planned path under the target |

## `SVC-017` A dry run reports a file whose plan is where it already is

| Step | Statement |
| --- | --- |
| Given | a dry run of a watch without a target whose pipeline keeps the name, and the file `x.mkv` |
| When | the batch is processed |
| Then | the file is reported as previewed at the path it already has |

