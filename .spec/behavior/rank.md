# Rank

A rank stage numbers the files of a group among themselves: files whose named fields agree are one group, and within a group they are put in an order the preference can bend. A file alone in its group is not numbered.

## Includes

- `tests/rank.rs`

## `RNK-001` The files of a group are numbered from 1 in name order

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` and the files `b.ass` and `a.ass`, both with `episode` 1 |
| When | the batch is planned |
| Then | `a.ass` is 1 and `b.ass` is 2 |

## `RNK-002` A name holding a preferred word comes first

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` preferring `cht`, and the files `a.ass` and `b.cht.ass`, both with `episode` 1 |
| When | the batch is planned |
| Then | `b.cht.ass` is 1 and `a.ass` is 2 |

## `RNK-003` An earlier preferred word comes before a later one

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` preferring `cht` then `chs`, and the files `a.chs.ass`, `b.cht.ass` and `c.ass`, all with `episode` 1 |
| When | the batch is planned |
| Then | `b.cht.ass` is 1, `a.chs.ass` is 2 and `c.ass` is 3 |

## `RNK-004` A file alone in its group is not numbered

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` and the file `a.ass` with `episode` 1 |
| When | the batch is planned |
| Then | `index` is unset |

## `RNK-005` Files whose fields differ are numbered apart

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` and the files `a.ass` with `episode` 1 and `b.ass` with `episode` 2 |
| When | the batch is planned |
| Then | neither has an `index` |

## `RNK-006` A field the group needs and a file lacks is refused

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` and the file `a.ass` without `episode` |
| When | the batch is planned |
| Then | `a.ass` is refused, naming the stage `rank` |

## `RNK-007` The preference ignores case

| Step | Statement |
| --- | --- |
| Given | a rank stage into `index` by `episode` preferring `cht`, and the files `a.ass` and `b.CHT.ass`, both with `episode` 1 |
| When | the batch is planned |
| Then | `b.CHT.ass` is 1 |

## `RNK-008` A record ranked on its own is a group of one and is not numbered

| Step | Statement |
| --- | --- |
| Given | the rank stage by `episode` and one record with `episode` 1 |
| When | the stage is applied to that record alone |
| Then | the record goes on without an `index` |
