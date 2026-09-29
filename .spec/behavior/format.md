# Format

A format stage rewrites the file name of the plan out of a template and leaves the folders and the extension alone. Its result has to be one usable file name.

## Includes

- `tests/format.rs`

## `FMT-001` The file name is rewritten and the folder kept

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show} s01e{episode:02}` and a record for `Series/Alpha/x.mkv` with `show` Alpha and `episode` 7 |
| When | the stage is applied |
| Then | the plan is `Series/Alpha/Alpha s01e07.mkv` |

## `FMT-002` A file without an extension gets no dot

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show}` and a record for `x` with `show` Alpha |
| When | the stage is applied |
| Then | the plan is `Alpha` |

## `FMT-003` A template that cannot be rendered is refused

| Step | Statement |
| --- | --- |
| Given | a format stage for `{missing}` and a record for `x.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `format` |

## `FMT-004` A name holding a slash is refused

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show}` and a record whose `show` is `a/b` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `format` |

## `FMT-005` An empty name is refused

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show}` and a record whose `show` is empty |
| When | the stage is applied |
| Then | the record is refused, naming the stage `format` |

## `FMT-006` A name of dots only is refused

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show}` and a record whose `show` is `..` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `format` |

## `FMT-007` The name field follows the format

| Step | Statement |
| --- | --- |
| Given | a format stage for `{show}` and a record for `x.mkv` with `show` Alpha |
| When | the stage is applied |
| Then | the field `name` is `Alpha` |
