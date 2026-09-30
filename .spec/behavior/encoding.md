# Encoding

File names on Linux are bytes and need not be valid UTF-8, while fields and templates are text. Text made from such a path would stand for a different file, so no record is made of it: the file enters no pipeline and stays in the source with the reason.

## Includes

- `tests/encoding.rs`

## `ENC-001` A path that is not valid UTF-8 makes no record

| Step | Statement |
| --- | --- |
| Given | a file whose name is not valid UTF-8 |
| When | a record is created for it |
| Then | no record is made, and the refusal says the name is not valid UTF-8 |

## `ENC-002` A file whose path is not valid UTF-8 is refused and stays

| Step | Statement |
| --- | --- |
| Given | a source holding a file whose name is not valid UTF-8 and a watch whose pipeline is `move` |
| When | the batch is processed |
| Then | the file is refused for its name and is still in the source |

## `ENC-003` A conflict suffix is not made from a name that is not valid UTF-8

| Step | Statement |
| --- | --- |
| Given | a target already holding the file a move would make, whose name is not valid UTF-8, and `on_conflict` set to suffix |
| When | the file is moved |
| Then | the move is refused as a conflict and the file stays |
