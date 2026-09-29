# Filter

A filter decides whether a file goes on down its pipeline. A file it lets through is unchanged; a file it turns away is excluded, which is not a refusal.

## Includes

- `tests/filter.rs`

## `FLT-001` A listed extension passes

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `mkv` and a record for `Alpha.mkv` |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `FLT-002` An unlisted extension is excluded

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `mkv` and a record for `Alpha.nfo` |
| When | the stage is applied |
| Then | the record is excluded |

## `FLT-003` Extensions are compared without regard to case

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `mkv` and a record for `Alpha.MKV` |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `FLT-004` A name pattern is matched against the whole file name

| Step | Statement |
| --- | --- |
| Given | a filter for the name pattern `Alpha*.mkv` and a record for `Alpha 12.mkv` |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `FLT-005` Inverting turns a match into an exclusion

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `nfo` with inversion and a record for `Alpha.nfo` |
| When | the stage is applied |
| Then | the record is excluded |

## `FLT-006` Inverting lets everything else through

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `nfo` with inversion and a record for `Alpha.mkv` |
| When | the stage is applied |
| Then | the record goes on unchanged |

## `FLT-007` Extension and pattern must both match

| Step | Statement |
| --- | --- |
| Given | a filter for the extension `mkv` and the name pattern `Beta*` and a record for `Alpha.mkv` |
| When | the stage is applied |
| Then | the record is excluded |
