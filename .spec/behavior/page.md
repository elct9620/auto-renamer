# Page

The playground page draws a configuration for people who do not read TOML: each watch is joined to the pipelines it uses, and each pipeline is its stages in order. The table the core reads is the only state; the drawing and every edit are made from it and back into it, so what is downloaded is what was drawn.

## Includes

- `playground/src/**/*.test.ts`

## `PGE-001` A pipeline is drawn as its stages in order

| Step | Statement |
| --- | --- |
| Given | a configuration whose pipeline `video` has the stages filter, format and move |
| When | it is drawn |
| Then | the pipeline leads to the filter, the filter to the format, and the format to the move |

## `PGE-002` A watch is joined to the pipelines it lists

| Step | Statement |
| --- | --- |
| Given | a configuration whose watch `series` lists the pipelines `video` and `subtitle` |
| When | it is drawn |
| Then | the watch leads to `video` first and to `subtitle` second |

## `PGE-003` A watch without its own list is joined to the default pipelines

| Step | Statement |
| --- | --- |
| Given | a configuration whose default lists the pipeline `video` and whose watch lists none |
| When | it is drawn |
| Then | the watch leads to `video` |

## `PGE-004` A stage added to a pipeline goes after the others

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter and move |
| When | a format stage is added to it |
| Then | its stages are filter, move and format |

## `PGE-005` A stage moved earlier is written earlier

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter, move and format |
| When | the format stage is moved one place earlier |
| Then | its stages are filter, format and move |

## `PGE-006` A removed stage leaves the others in order

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter, format and move |
| When | the format stage is removed |
| Then | its stages are filter and move |

## `PGE-007` A stage's parameters are written as its value

| Step | Statement |
| --- | --- |
| Given | a pipeline whose first stage is a filter on `mkv` |
| When | its parameters are set to a filter on `mp4` |
| Then | the first stage is a filter on `mp4` and the other stages are unchanged |

## `PGE-008` A stage without parameters is written as its name

| Step | Statement |
| --- | --- |
| Given | a pipeline |
| When | a move stage is added with no parameters |
| Then | the stage is written as the text `move` |

## `PGE-009` A configuration that is refused cannot be downloaded

| Step | Statement |
| --- | --- |
| Given | a configuration the core refuses |
| When | a download is asked for |
| Then | no file is offered and the reason is shown |

## `PGE-010` A stage dropped between two stages is written between them

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter and move |
| When | a format stage is dropped between them |
| Then | its stages are filter, format and move |

## `PGE-011` A watch joined to a pipeline lists it last

| Step | Statement |
| --- | --- |
| Given | a watch listing the pipeline `video` |
| When | it is joined to `subtitle` |
| Then | it lists `video` and `subtitle` |

## `PGE-012` A watch following the default lists its own pipelines once joined

| Step | Statement |
| --- | --- |
| Given | a default listing `video` and a watch listing none |
| When | the watch is joined to `subtitle` |
| Then | the watch lists `video` and `subtitle` |

## `PGE-013` A joint removed drops the pipeline from the watch

| Step | Statement |
| --- | --- |
| Given | a watch listing `video` and `subtitle` |
| When | its joint to `video` is removed |
| Then | it lists `subtitle` |

## `PGE-014` A renamed pipeline is renamed where it is listed

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` listed by the default and by a watch |
| When | it is renamed `episode` |
| Then | the default and the watch list `episode` and not `video` |

## `PGE-015` A removed pipeline is listed nowhere

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` listed by the default and by a watch |
| When | it is removed |
| Then | neither the default nor the watch lists `video` |

## `PGE-016` A new stage starts with the example the core describes

| Step | Statement |
| --- | --- |
| Given | the stages the core describes |
| When | a regex stage is added |
| Then | the stage is written with the regex example |

## `PGE-017` The virtual tree is shown as the watch's source and target

| Step | Statement |
| --- | --- |
| Given | a watch from `/downloads` to `/video` and files under both |
| When | the tree is drawn for the watch |
| Then | the source shows the files under `/downloads` and the target those under `/video` |

## `PGE-018` A file is shown inside its folders

| Step | Statement |
| --- | --- |
| Given | the file `/downloads/Alpha/x.mkv` |
| When | the source tree is drawn |
| Then | `x.mkv` is shown inside the folder `Alpha` |

## `PGE-019` A removed folder takes what it holds

| Step | Statement |
| --- | --- |
| Given | the files `/downloads/Alpha/x.mkv` and `/downloads/Beta/y.mkv` |
| When | the folder `/downloads/Alpha` is removed |
| Then | only `/downloads/Beta/y.mkv` is left |

## `PGE-020` A renamed folder carries what it holds

| Step | Statement |
| --- | --- |
| Given | the file `/downloads/Alpha/x.mkv` |
| When | the folder `/downloads/Alpha` is renamed `Beta` |
| Then | the file is `/downloads/Beta/x.mkv` |
