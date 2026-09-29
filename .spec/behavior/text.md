# Text

Stages that set fixed values and rewrite text fields, one small change each.

## Includes

- `tests/text.rs`

## `SET-001` Set replaces a field

| Step | Statement |
| --- | --- |
| Given | a set stage giving `season` the number 1 and a record whose `season` is 5 |
| When | the stage is applied |
| Then | `season` is 1 |

## `SET-002` Default fills a field that is unset

| Step | Statement |
| --- | --- |
| Given | a default stage giving `season` the number 1 and a record with no `season` |
| When | the stage is applied |
| Then | `season` is 1 |

## `SET-003` Default keeps a field that is set

| Step | Statement |
| --- | --- |
| Given | a default stage giving `season` the number 1 and a record whose `season` is 5 |
| When | the stage is applied |
| Then | `season` is 5 |

## `REP-001` A replace swaps the text asked for

| Step | Statement |
| --- | --- |
| Given | a replace stage changing `_` into a space and a record for `a_b.mkv` |
| When | the stage is applied |
| Then | `name` is `a b` |

## `REP-002` A replace swaps every occurrence

| Step | Statement |
| --- | --- |
| Given | a replace stage changing `_` into a space and a record for `a_b_c.mkv` |
| When | the stage is applied |
| Then | `name` is `a b c` |

## `REP-003` A replace on a field that does not exist is refused

| Step | Statement |
| --- | --- |
| Given | a replace stage on the field `missing` and a record for `a.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `replace` |

## `REP-004` A replace on a number is refused

| Step | Statement |
| --- | --- |
| Given | a replace stage on the field `episode` and a record whose `episode` is the number 3 |
| When | the stage is applied |
| Then | the record is refused, naming the stage `replace` |

## `CAS-001` Lower case

| Step | Statement |
| --- | --- |
| Given | a case stage to lower and a record for `ALPHA Beta.mkv` |
| When | the stage is applied |
| Then | `name` is `alpha beta` |

## `CAS-002` Upper case

| Step | Statement |
| --- | --- |
| Given | a case stage to upper and a record for `alpha Beta.mkv` |
| When | the stage is applied |
| Then | `name` is `ALPHA BETA` |

## `CAS-003` Title case capitalizes each word

| Step | Statement |
| --- | --- |
| Given | a case stage to title and a record for `aLPHA bETA.mkv` |
| When | the stage is applied |
| Then | `name` is `Alpha Beta` |

## `CAS-004` A case change on a field that does not exist is refused

| Step | Statement |
| --- | --- |
| Given | a case stage to lower on the field `missing` and a record for `a.mkv` |
| When | the stage is applied |
| Then | the record is refused, naming the stage `case` |

## `STR-001` Square brackets and what they hold are removed

| Step | Statement |
| --- | --- |
| Given | a strip stage and a record for `[Team] Show [1080p].mkv` |
| When | the stage is applied |
| Then | `name` is `Show` |

## `STR-002` Parentheses are kept unless asked for

| Step | Statement |
| --- | --- |
| Given | a strip stage and a record for `Show (2026).mkv` |
| When | the stage is applied |
| Then | `name` is `Show (2026)` |

## `STR-003` Other bracket groups are removed when asked for

| Step | Statement |
| --- | --- |
| Given | a strip stage for parentheses and a record for `Show (2026).mkv` |
| When | the stage is applied |
| Then | `name` is `Show` |

## `STR-004` Leftover spaces are collapsed and trimmed

| Step | Statement |
| --- | --- |
| Given | a strip stage and a record for `[A]  Show   [B]  Two.mkv` |
| When | the stage is applied |
| Then | `name` is `Show Two` |

## `STR-005` A group that is never closed stays

| Step | Statement |
| --- | --- |
| Given | a strip stage and a record for `Show [1080p.mkv` |
| When | the stage is applied |
| Then | `name` is `Show [1080p` |

## `STR-006` Nested groups go together

| Step | Statement |
| --- | --- |
| Given | a strip stage and a record for `Show [a [b] c].mkv` |
| When | the stage is applied |
| Then | `name` is `Show` |
