# Effects

A pipeline ends in effect stages. Planning only notes what each asks for on the files that reach it; once the whole batch is planned, the effects of a file run in the order they are written. If one fails, none after it runs, and what already happened is reported with the failure.

## Includes

- `tests/effects.rs`

## `EFF-001` Effects run in the order they are written

| Step | Statement |
| --- | --- |
| Given | a pipeline that moves and then cleans up, a file `Rel/x.mkv` planned to `y.mkv`, and a unit `Rel` |
| When | the effects of the pipeline are applied |
| Then | the file is moved and then `Rel`, now empty, is removed |

## `EFF-002` A failed move stops the effects after it

| Step | Statement |
| --- | --- |
| Given | the same pipeline and a target that already holds `y.mkv` |
| When | the effects of the pipeline are applied |
| Then | the failure is reported, `Rel` is kept, and the move is not in what was done |

## `EFF-003` A dry run previews the move, runs no cleanup and changes nothing

| Step | Statement |
| --- | --- |
| Given | the same pipeline and a dry run |
| When | the effects of the pipeline are applied |
| Then | the move preview is all that was done, and nothing has changed |

## `EFF-004` A pipeline without effects does nothing

| Step | Statement |
| --- | --- |
| Given | a pipeline with no effect stage and a file planned to `y.mkv` |
| When | the effects of the pipeline are applied |
| Then | nothing is done and the file has not moved |

## `EFF-006` A planned file carries the effects of its pipeline in the order they are written

| Step | Statement |
| --- | --- |
| Given | a pipeline that cleans up and then moves, and a file it claims |
| When | the batch is planned |
| Then | the file is planned with a cleanup and then a move to be done |

## `EFF-008` A file carries only the effects of the pipeline that claimed it

| Step | Statement |
| --- | --- |
| Given | a first pipeline for `mkv` without an effect stage, a second pipeline of `move`, and an `mkv` file |
| When | the batch is planned |
| Then | the file is planned by the first pipeline with nothing to be done to it |
