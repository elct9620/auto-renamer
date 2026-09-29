# Encoding

File names on Linux are bytes and need not be valid UTF-8, while fields and templates are text. A file whose path is not valid UTF-8 is never renamed through a lossy copy of its name: whatever needs its name refuses it, and whatever does not moves it by its own bytes.

## Includes

- `tests/encoding.rs`

## `ENC-001` A path that is not valid UTF-8 has no name fields

| Step | Statement |
| --- | --- |
| Given | a file whose name is not valid UTF-8 |
| When | a record is created for it |
| Then | the record is not readable and has no `name`, `ext`, `dir` or `path`, and still has `mtime` |

## `ENC-002` A stage that reads names refuses such a file

| Step | Statement |
| --- | --- |
| Given | a record for a file whose name is not valid UTF-8 and the stage `strip` |
| When | the stage is applied |
| Then | the record is refused by `strip`, saying the name is not valid UTF-8 |

## `ENC-003` A filter does not claim such a file

| Step | Statement |
| --- | --- |
| Given | a batch of a file whose name is not valid UTF-8 and a pipeline that starts with a filter for `mkv` |
| When | the batch is planned |
| Then | the file is not claimed by the pipeline |

## `ENC-004` A file no pipeline claims is refused for its name

| Step | Statement |
| --- | --- |
| Given | a batch of a file whose name is not valid UTF-8 and only a pipeline that starts with a filter |
| When | the batch is planned |
| Then | the file is refused by `name` rather than left unclaimed |

## `ENC-005` A folder stage keeps the bytes of the name

| Step | Statement |
| --- | --- |
| Given | a batch of a file whose name is not valid UTF-8 and a pipeline of `folder` by year and `move` |
| When | the batch is planned |
| Then | the plan is the year folder with the same name bytes |

## `ENC-006` A file is moved by its own bytes

| Step | Statement |
| --- | --- |
| Given | a source holding a file whose name is not valid UTF-8 and a watch whose pipeline is `move` |
| When | the batch is processed |
| Then | the target holds a file with the same name bytes and the same content |

## `ENC-007` A conflict suffix is not made from such a name

| Step | Statement |
| --- | --- |
| Given | a target already holding the file a move would make, whose name is not valid UTF-8, and `on_conflict` set to suffix |
| When | the file is moved |
| Then | the move is refused as a conflict and the file stays |
