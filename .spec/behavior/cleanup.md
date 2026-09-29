# Cleanup

After a file has moved out, the folders it leaves empty are removed, from the folder it was in upward. It only ever removes empty folders inside the unit, never the source itself, and stops at anything a person asked to keep.

## Includes

- `tests/cleanup.rs`

## `CLN-001` The folder a file left empty is removed

| Step | Statement |
| --- | --- |
| Given | a unit `Rel` holding only a folder `Rel` that is now empty and the file that was in it `Rel/x.mkv` |
| When | the folders are cleaned up |
| Then | `Rel` is removed |

## `CLN-002` A folder that still holds a file is kept

| Step | Statement |
| --- | --- |
| Given | a folder `Rel` from which `x.mkv` moved out and which still holds `y.mkv` |
| When | the folders are cleaned up |
| Then | `Rel` is kept |

## `CLN-003` Folders emptied one inside another are removed upward

| Step | Statement |
| --- | --- |
| Given | a unit `Season` holding `Rel/Subs`, both now empty, for the file that was in `Season/Rel/Subs/x.ass` |
| When | the folders are cleaned up |
| Then | `Season/Rel/Subs` and `Season/Rel` are removed, and `Season` too as it is the unit and empty |

## `CLN-004` A folder matching a keep pattern stops the cleanup

| Step | Statement |
| --- | --- |
| Given | a unit `Series` with the folders `Season 01/Rel` both empty, and keep `Season *` |
| When | the folders are cleaned up |
| Then | `Season 01/Rel` is removed and `Season 01` is kept |

## `CLN-005` The source itself is never removed

| Step | Statement |
| --- | --- |
| Given | a unit that is the whole source and a source whose only folder `Rel` has been emptied |
| When | the folders are cleaned up |
| Then | `Rel` is removed and the source is kept |

## `CLN-006` Folders above the unit are not touched

| Step | Statement |
| --- | --- |
| Given | a unit `Movies/A` and the folders `Movies/A` and `Movies` both empty |
| When | the folders are cleaned up |
| Then | `Movies/A` is removed and `Movies` is kept |

## `CLN-007` A folder holding a folder configuration is kept

| Step | Statement |
| --- | --- |
| Given | a folder `Rel` from which the last file moved out and which holds `auto-renamer.toml` |
| When | the folders are cleaned up |
| Then | `Rel` is kept |

## `CLN-008` A linked folder is not removed

| Step | Statement |
| --- | --- |
| Given | a folder `Rel` that is a symbolic link to an empty folder |
| When | the folders are cleaned up |
| Then | the link and the folder it points to are kept |

## `CLN-010` The answer lists the folders removed, innermost first

| Step | Statement |
| --- | --- |
| Given | a unit `Season` holding `Rel/Subs`, both empty |
| When | the folders are cleaned up |
| Then | the answer is `Season/Rel/Subs`, `Season/Rel`, `Season` |
