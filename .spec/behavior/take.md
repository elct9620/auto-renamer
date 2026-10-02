# Take

A take stage copies fields from the file a name belongs with, found among what earlier pipelines planned: the file whose name is the longest beginning of this one. It never guesses between two.

## Includes

- `tests/take.rs`

## `TAK-001` Fields are taken from the file whose name begins this one

| Step | Statement |
| --- | --- |
| Given | pipeline `video` planning `Show 27.mkv` with `episode` 27, a pipeline `subtitle` taking `episode`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` has `episode` 27 |

## `TAK-002` The longest beginning wins

| Step | Statement |
| --- | --- |
| Given | pipeline `video` planning `Show.mkv` with `episode` 1 and `Show 27.mkv` with `episode` 27, a pipeline `subtitle` taking `episode`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` has `episode` 27 |

## `TAK-003` A file with nothing to take from is refused

| Step | Statement |
| --- | --- |
| Given | a pipeline `subtitle` taking `episode` and the file `Show 27.cht.ass` alone in the batch |
| When | the batch is planned |
| Then | `Show 27.cht.ass` is refused, naming the stage `take` |

## `TAK-004` Two equally long beginnings are refused

| Step | Statement |
| --- | --- |
| Given | pipeline `video` planning `Show 27.mkv` and `Show 27.mp4`, a pipeline `subtitle` taking `episode`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` is refused, naming the stage `take` |

## `TAK-005` A file that took from a refused file is refused

| Step | Statement |
| --- | --- |
| Given | pipeline `video` refusing `Show 27.mkv`, a pipeline `subtitle` taking `episode`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` is refused, naming the stage `take` |

## `TAK-006` Only the pipeline asked for is taken from

| Step | Statement |
| --- | --- |
| Given | pipelines `video` and `extra` both planning a file that begins the name, and a take from `extra` for `Show 27.cht.ass` |
| When | the batch is planned |
| Then | the fields are those of the file planned by `extra` |

## `TAK-007` A field the other file lacks is refused

| Step | Statement |
| --- | --- |
| Given | pipeline `video` planning `Show 27.mkv` without `episode`, a pipeline `subtitle` taking `episode`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` is refused, naming the stage `take` |

## `TAK-008` A pipeline is not taken from before it has run

| Step | Statement |
| --- | --- |
| Given | a pipeline `subtitle` taking `episode` followed by a pipeline `video` planning `Show 27.mkv`, and the file `Show 27.cht.ass` |
| When | the batch is planned |
| Then | `Show 27.cht.ass` is refused, naming the stage `take` |


## `TAK-009` A file named as the video it follows takes its fields

| Step | Statement |
| --- | --- |
| Given | pipeline `video` planning `Show 27.mkv` with `episode` 27, a pipeline `subtitle` taking `episode`, and the file `Show 27.ass` |
| When | the batch is planned |
| Then | `Show 27.ass` has `episode` 27 |
