# Unit

A unit is the set of files that may relate to one another. A watch says how a file is assigned to one, and the answer is a folder of the source, named relative to it.

## Includes

- `tests/unit.rs`

## `UNT-001` By default the unit is the folder of the file

| Step | Statement |
| --- | --- |
| Given | the unit `directory` and the file `Movies/A/x.mkv` |
| When | the unit of the file is asked for |
| Then | it is `Movies/A` |

## `UNT-002` The source can be one unit

| Step | Statement |
| --- | --- |
| Given | the unit `source` and the file `Movies/A/x.mkv` |
| When | the unit of the file is asked for |
| Then | it is the source itself, the empty path |

## `UNT-003` A folder matching a root pattern is a unit

| Step | Statement |
| --- | --- |
| Given | the roots `Movies/*` and the file `Movies/A/x.mkv` |
| When | the unit of the file is asked for |
| Then | it is `Movies/A` |

## `UNT-004` Files below a matching folder belong to it

| Step | Statement |
| --- | --- |
| Given | the roots `Movies/*` and the file `Movies/A/Subs/x.ass` |
| When | the unit of the file is asked for |
| Then | it is `Movies/A` |

## `UNT-005` A file matching no root falls back to its folder

| Step | Statement |
| --- | --- |
| Given | the roots `Movies/*` and the file `Music/B/x.mp3` |
| When | the unit of the file is asked for |
| Then | it is `Music/B` |

## `UNT-006` A star does not reach across a slash

| Step | Statement |
| --- | --- |
| Given | the roots `*` and the file `Movies/A/x.mkv` |
| When | the unit of the file is asked for |
| Then | it is `Movies` |

## `UNT-007` The shallowest matching folder wins

| Step | Statement |
| --- | --- |
| Given | the roots `Movies/*` and `Movies/*/*` and the file `Movies/A/B/x.mkv` |
| When | the unit of the file is asked for |
| Then | it is `Movies/A` |

## `UNT-008` A file at the source root falls back to the root

| Step | Statement |
| --- | --- |
| Given | the roots `Movies/*` and the file `x.mkv` |
| When | the unit of the file is asked for |
| Then | it is the source itself, the empty path |
