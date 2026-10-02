# Path

Stages that move a file to another folder of its plan, up or down, without touching its file name. The folders of a plan can only stay inside the source, so what would leave it is refused.

## Includes

- `tests/path.rs`

## `LIFT-001` A number of levels lifts the plan that many folders

| Step | Statement |
| --- | --- |
| Given | a lift of 2 levels and a record for `a/b/c/x.mkv` |
| When | the stage is applied |
| Then | the plan is `a/x.mkv` |

## `LIFT-002` Lifting above the source is refused

| Step | Statement |
| --- | --- |
| Given | a lift of 3 levels and a record for `a/b/x.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `lift` |

## `LIFT-003` A name pattern lifts to the nearest folder that matches

| Step | Statement |
| --- | --- |
| Given | a lift to `Season *` and a record for `Series/Season 01/Rel/Subs/x.ass` |
| When | the stage is applied |
| Then | the plan is `Series/Season 01/x.ass` |

## `LIFT-004` A plan already in a matching folder is unchanged

| Step | Statement |
| --- | --- |
| Given | a lift to `Season *` and a record for `Series/Season 01/x.mkv` |
| When | the stage is applied |
| Then | the plan is `Series/Season 01/x.mkv` |

## `LIFT-005` A plan with no matching folder is unchanged

| Step | Statement |
| --- | --- |
| Given | a lift to `Season *` and a record for `Series/Alpha/x.mkv` |
| When | the stage is applied |
| Then | the plan is `Series/Alpha/x.mkv` |

## `LIFT-006` Keeping a number of folders drops the folders below them

| Step | Statement |
| --- | --- |
| Given | a lift keeping 1 folder and a record for `Alpha/[Rel] 01/Subs/x.ass` |
| When | the stage is applied |
| Then | the plan is `Alpha/x.ass` |

## `LIFT-007` A plan with no more folders than kept is unchanged

| Step | Statement |
| --- | --- |
| Given | a lift keeping 1 folder and a record for `Alpha/x.mkv` |
| When | the stage is applied |
| Then | the plan is `Alpha/x.mkv` |

## `FLD-002` A template that cannot be rendered is refused

| Step | Statement |
| --- | --- |
| Given | a folder stage for `{missing}` and a record for `Photos/x.jpg` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `folder` |

## `FLD-003` A folder named with dots only is refused

| Step | Statement |
| --- | --- |
| Given | a folder stage for `{show}` and a record whose `show` is `..` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `folder` |

## `FLD-004` An empty folder name is refused

| Step | Statement |
| --- | --- |
| Given | a folder stage for `/{show}` and a record for `Photos/x.jpg` with `show` Alpha |
| When | the stage is applied |
| Then | the record is refused, naming the stage `folder` |
