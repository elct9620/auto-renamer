# Move

A move puts a file at its plan under the target, and it is the only thing that touches the files of the source. It never overwrites, never follows a link, and never leaves the source or the target: whatever it cannot do safely it refuses and leaves everything as it was.

## Includes

- `tests/move.rs`
- `src/effects/relocate.rs`

## `MV-001` A file is moved to its plan under the target, folders made as needed

| Step | Statement |
| --- | --- |
| Given | a file `Series/x.mkv` in the source and the plan `Series/Alpha/y.mkv` |
| When | the file is moved |
| Then | the target holds `Series/Alpha/y.mkv` |

## `MV-002` The source no longer has the file, and the content is kept

| Step | Statement |
| --- | --- |
| Given | a file `x.mkv` holding some bytes and the plan `y.mkv` |
| When | the file is moved |
| Then | the source has no `x.mkv` and the target has `y.mkv` with the same bytes |

## `MV-003` Without a separate target the file is renamed in place

| Step | Statement |
| --- | --- |
| Given | a source that is also the target, a file `x.mkv` and the plan `y.mkv` |
| When | the file is moved |
| Then | `y.mkv` is in the source and `x.mkv` is not |

## `MV-004` A file already at its plan is left as it is

| Step | Statement |
| --- | --- |
| Given | a target that is the source, a file `x.mkv` and the plan `x.mkv` |
| When | the file is moved |
| Then | nothing is done and the file is unchanged |

## `MV-005` A file at the plan already is not overwritten

| Step | Statement |
| --- | --- |
| Given | a file `x.mkv` and the plan `y.mkv` where the target already holds a different `y.mkv` |
| When | the file is moved |
| Then | the move is refused as a conflict and both files are as they were |

## `MV-006` A conflict can be settled by a suffix

| Step | Statement |
| --- | --- |
| Given | a file `x.mkv`, the plan `y.mkv` where the target already holds `y.mkv`, and a suffix `_v2` asked for |
| When | the file is moved |
| Then | the target holds `y_v2.mkv` and the earlier `y.mkv` is as it was |

## `MV-007` A suffix that also conflicts is refused

| Step | Statement |
| --- | --- |
| Given | a file `x.mkv`, the plan `y.mkv`, a suffix `_v2` asked for, and a target holding `y.mkv` and `y_v2.mkv` |
| When | the file is moved |
| Then | the move is refused as a conflict and nothing is overwritten |

## `MV-008` The suffix goes before the extension

| Step | Statement |
| --- | --- |
| Given | a file `x`, the plan `y` where the target already holds `y`, and a suffix `_v2` asked for |
| When | the file is moved |
| Then | the target holds `y_v2` |

## `MV-009` A plan that leaves the target is refused

| Step | Statement |
| --- | --- |
| Given | a file `x.mkv` and the plan `../y.mkv` |
| When | the file is moved |
| Then | the move is refused as unsafe and nothing is moved |

## `MV-011` A link is not moved

| Step | Statement |
| --- | --- |
| Given | a symbolic link `x.mkv` in the source and the plan `y.mkv` |
| When | the file is moved |
| Then | the file is skipped as a link, and the link is untouched |

## `MV-012` A file below a linked folder is not moved

| Step | Statement |
| --- | --- |
| Given | a folder `Linked` that is a symbolic link to another folder holding `x.mkv`, and the plan `y.mkv` |
| When | the file is moved |
| Then | the file is skipped as a link, and the other folder is untouched |

## `MV-013` A folder is not moved

| Step | Statement |
| --- | --- |
| Given | a folder `Season` in the source and the plan `Other` |
| When | the file is moved |
| Then | the folder is skipped as not a file, and it is untouched |

## `MV-015` A file that is not there is refused

| Step | Statement |
| --- | --- |
| Given | the plan `y.mkv` for a file `x.mkv` that does not exist |
| When | the file is moved |
| Then | the move is refused because the file is missing |

## `MV-016` A move between filesystems goes through a temporary name

| Step | Statement |
| --- | --- |
| Given | a file on one filesystem, a target on another, and the plan `y.mkv` |
| When | the file is moved |
| Then | `y.mkv` holds the same bytes, the source no longer has the file, and no temporary file is left |

## `MV-018` A file that appears at the plan after it was checked is not replaced

| Step | Statement |
| --- | --- |
| Given | a file placed at a path where another file has appeared since the path was checked |
| When | the file is put there |
| Then | it is refused as already existing and the other file is as it was |

## `MV-020` A temporary file left by an interrupted move does not block the next one

| Step | Statement |
| --- | --- |
| Given | a file on one filesystem, a target on another that holds a hidden `.y.mkv.part` left by an earlier move, and the plan `y.mkv` |
| When | the file is moved |
| Then | the target holds the same bytes as `y.mkv` |

## `MV-021` A move within a filesystem is seen as a file moved in

| Step | Statement |
| --- | --- |
| Given | a file in a watched folder and a plan that renames it within the same folder |
| When | the file is moved and the notifications are translated |
| Then | the new name is settled and is never taken for a file being written |

## `MV-026` A temporary name already taken is not handed to `mv`

| Step | Statement |
| --- | --- |
| Given | a file and a temporary name that something else already holds |
| When | the file is handed to `mv` under that name |
| Then | the move is refused, the file is still there, and what held the temporary name is as it was |

## `MV-027` A plan taken at the last step keeps the file under its temporary name

| Step | Statement |
| --- | --- |
| Given | a file, and a plan `y.mkv` that appears after the file was checked but before it is put in place |
| When | the file is moved through a temporary name |
| Then | the move is refused naming the temporary file, which holds the file's bytes, and `y.mkv` is as it was |

## `MV-028` A move `mv` cannot finish leaves the file in the source and no temporary file

| Step | Statement |
| --- | --- |
| Given | a file larger than the room left on the filesystem of the target, and the plan `y.mkv` |
| When | the file is moved |
| Then | the move is refused, the file is still in the source, and the target holds nothing |
