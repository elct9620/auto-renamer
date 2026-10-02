# Effects

A route joins a pipeline to its effects: a move to its target, or in place, and then a cleanup. Planning never touches a file; once the whole batch is planned, the effects of a file run, move first. If the move fails, the cleanup does not run, and what already happened is reported with the failure.

## Includes

- `tests/effects.rs`

## `EFF-001` Effects run in the order they are written

| Step | Statement |
| --- | --- |
| Given | a route that moves and then cleans up, a file `Rel/x.mkv` planned to `y.mkv`, and a unit `Rel` |
| When | the effects of the route are applied |
| Then | the file is moved and then `Rel`, now empty, is removed |

## `EFF-002` A failed move stops the effects after it

| Step | Statement |
| --- | --- |
| Given | the same route and a target that already holds `y.mkv` |
| When | the effects of the route are applied |
| Then | the failure is reported, `Rel` is kept, and the move is not in what was done |

## `EFF-003` A dry run previews the move, runs no cleanup and changes nothing

| Step | Statement |
| --- | --- |
| Given | the same route and a dry run |
| When | the effects of the route are applied |
| Then | the move preview is all that was done, and nothing has changed |

## `EFF-008` A file takes the effects of the route that claimed it

| Step | Statement |
| --- | --- |
| Given | a first route for `mkv` into the target `a`, a second route into the target `b`, and an `mkv` file |
| When | the batch is processed |
| Then | the file is moved into `a` |