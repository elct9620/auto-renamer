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

## `EFF-003` A dry run previews every effect and changes nothing

| Step | Statement |
| --- | --- |
| Given | the same pipeline and a dry run |
| When | the effects of the pipeline are applied |
| Then | the move is previewed, nothing has moved, and `Rel` is still there |

## `EFF-004` A pipeline without effects does nothing

| Step | Statement |
| --- | --- |
| Given | a pipeline with no effect stage and a file planned to `y.mkv` |
| When | the effects of the pipeline are applied |
| Then | nothing is done and the file has not moved |
