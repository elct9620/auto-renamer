# Effects

A pipeline ends in effect stages, which run in the order they are written on the file that was planned. If one fails, none after it runs, and what already happened is reported with the failure.

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

## `EFF-005` An effect stage applied while planning passes the record on unchanged

| Step | Statement |
| --- | --- |
| Given | the `move` stage and a record for `x.mkv` |
| When | the stage is applied to the record |
| Then | the record goes on as it was |
