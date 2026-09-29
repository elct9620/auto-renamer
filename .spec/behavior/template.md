# Template

A template writes a name out of a record's fields. It is checked when it is read, and again against the fields of each record it is rendered for.

## Includes

- `tests/template.rs`

## `TPL-001` A field is written as its text

| Step | Statement |
| --- | --- |
| Given | the template `{show} done` and a record whose `show` is `Alpha` |
| When | the template is rendered |
| Then | the result is `Alpha done` |

## `TPL-002` A number is padded to the width asked for

| Step | Statement |
| --- | --- |
| Given | the template `e{episode:02}` and a record whose `episode` is the number 7 |
| When | the template is rendered |
| Then | the result is `e07` |

## `TPL-003` A number wider than the padding is kept whole

| Step | Statement |
| --- | --- |
| Given | the template `e{episode:02}` and a record whose `episode` is the number 1354 |
| When | the template is rendered |
| Then | the result is `e1354` |

## `TPL-004` A date is written in the format asked for

| Step | Statement |
| --- | --- |
| Given | the template `{mtime:%Y-%m}` and a record modified on 2026-09-27 |
| When | the template is rendered |
| Then | the result is `2026-09` |

## `TPL-005` An optional part is written when its fields are present

| Step | Statement |
| --- | --- |
| Given | the template `zh[.{index:02}]` and a record whose `index` is the number 1 |
| When | the template is rendered |
| Then | the result is `zh.01` |

## `TPL-006` An optional part is left out when a field is missing

| Step | Statement |
| --- | --- |
| Given | the template `zh[.{index:02}]` and a record with no `index` |
| When | the template is rendered |
| Then | the result is `zh` |

## `TPL-007` A missing required field refuses the render

| Step | Statement |
| --- | --- |
| Given | the template `{show} s{season:02}` and a record with no `season` |
| When | the template is rendered |
| Then | the render is refused, naming the field `season` |

## `TPL-008` Doubled braces are literal braces

| Step | Statement |
| --- | --- |
| Given | the template `{{name}}` |
| When | the template is rendered |
| Then | the result is `{name}` |

## `TPL-009` Doubled brackets are literal brackets

| Step | Statement |
| --- | --- |
| Given | the template `[[tag]]` |
| When | the template is rendered |
| Then | the result is `[tag]` |

## `TPL-010` A field left open is refused when the template is read

| Step | Statement |
| --- | --- |
| Given | the template `{show` |
| When | the template is read |
| Then | the template is refused |

## `TPL-011` An optional part left open is refused when the template is read

| Step | Statement |
| --- | --- |
| Given | the template `zh[.{index}` |
| When | the template is read |
| Then | the template is refused |

## `TPL-012` Optional parts do not nest

| Step | Statement |
| --- | --- |
| Given | the template `a[b[c]]` |
| When | the template is read |
| Then | the template is refused |

## `TPL-013` A date format the calendar library does not know is refused

| Step | Statement |
| --- | --- |
| Given | the template `{mtime:%Q}` |
| When | the template is read |
| Then | the template is refused |

## `TPL-014` A padding on text is refused

| Step | Statement |
| --- | --- |
| Given | the template `{show:02}` and a record whose `show` is the text `Alpha` |
| When | the template is rendered |
| Then | the render is refused, naming the field `show` |

## `TPL-015` A field with no name is refused when the template is read

| Step | Statement |
| --- | --- |
| Given | the template `a{}b` |
| When | the template is read |
| Then | the template is refused |

## `TPL-016` A date without a format is refused

| Step | Statement |
| --- | --- |
| Given | the template `{mtime}` and a record with a modification time |
| When | the template is rendered |
| Then | the render is refused, naming the field `mtime` |

## `TPL-017` A padding wider than any number is refused when the template is read

| Step | Statement |
| --- | --- |
| Given | the template `{episode:021}` |
| When | the template is read |
| Then | the template is refused |

## `TPL-018` A date format on a number is refused

| Step | Statement |
| --- | --- |
| Given | the template `{episode:%Y}` and a record whose `episode` is the number 7 |
| When | the template is rendered |
| Then | the render is refused, naming the field `episode` |

## `TPL-019` A padding on a date is refused

| Step | Statement |
| --- | --- |
| Given | the template `{mtime:02}` and a record with a modification time |
| When | the template is rendered |
| Then | the render is refused, naming the field `mtime` |

