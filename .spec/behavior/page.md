# Page

The playground page draws a configuration for people who do not read TOML: each watch is joined to the pipelines it uses, and each pipeline is its stages in order. The table the core reads is the baseline; the drawing and every edit are made from it and back into it, so what is downloaded is what was drawn. Where a node sits is only its layout, which is neither written nor kept.

## Includes

- `playground/src/**/*.test.ts`
- `playground/e2e/*.e2e.ts`

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

## `PGE-022` A parameter set in a stage's form is written into the stage

| Step | Statement |
| --- | --- |
| Given | a pipeline whose first stage is a filter on `mkv` |
| When | its `invert` parameter is set in the form |
| Then | the first stage is a filter on `mkv` that inverts |

## `PGE-023` A stage left without parameters is written as its name

| Step | Statement |
| --- | --- |
| Given | a move stage whose `on_conflict` is `suffix` |
| When | `on_conflict` is cleared in the form |
| Then | the stage is written as the text `move` |

## `PGE-021` A file a simulation moved is marked where it arrived

| Step | Statement |
| --- | --- |
| Given | a simulation that moved `/downloads/Alpha/x.mkv` to `/video/Alpha/Alpha.mkv` |
| When | the trees are drawn from it |
| Then | `/video/Alpha/Alpha.mkv` is marked as moved |

## `PGE-024` A browser preferring Chinese is shown the page in Traditional Chinese

| Step | Statement |
| --- | --- |
| Given | a browser whose preferred languages are `zh-CN` and `en` |
| When | the page chooses its language |
| Then | it is Traditional Chinese |

## `PGE-025` Any other browser is shown the page in English

| Step | Statement |
| --- | --- |
| Given | a browser whose preferred languages are `ja` and `fr` |
| When | the page chooses its language |
| Then | it is English |

## `PGE-026` Every text of the page is written in both languages

| Step | Statement |
| --- | --- |
| Given | the English and the Traditional Chinese texts of the page |
| When | they are compared |
| Then | each names the same texts |

## `PGE-027` A node moved on the canvas is drawn where it was left

| Step | Statement |
| --- | --- |
| Given | a drawing of a configuration with the watch `series` |
| When | the watch is moved by hand |
| Then | it is drawn where it was left |

## `PGE-028` Stages whose order changed are drawn in their order

| Step | Statement |
| --- | --- |
| Given | a pipeline whose format stage was moved by hand |
| When | its stages are put in another order |
| Then | each of its stages is drawn where its order places it |

## `PGE-029` Every setting the page offers is one the CLI reads

| Step | Statement |
| --- | --- |
| Given | a watch, the default or a folder configuration holding a setting its form offers, at the setting's example |
| When | the configuration is checked |
| Then | it is not refused |

## `PGE-030` Every stage and parameter the core declares is described in both languages

| Step | Statement |
| --- | --- |
| Given | the stages the core declares, with their parameters and single values |
| When | the page's texts in Traditional Chinese and English are read |
| Then | each stage, parameter and single value has a description in both |

## `PGE-031` A new path is offered the folders already in the tree

| Step | Statement |
| --- | --- |
| Given | a source holding `Alpha/Season 1/01.mkv` |
| When | the folders to offer for a new path under it are listed |
| Then | they are `Alpha/` and `Alpha/Season 1/` |

## `PGE-032` A node moved on the canvas leaves the configuration as it was

| Step | Statement |
| --- | --- |
| Given | the example configuration drawn on the canvas |
| When | the watch is dragged elsewhere |
| Then | the configuration text is unchanged |

## `PGE-033` A reset layout draws every node where its order places it

| Step | Statement |
| --- | --- |
| Given | a canvas whose watch was dragged elsewhere |
| When | the layout is reset |
| Then | the watch is drawn where it was first drawn |

## `PGE-034` Choosing an example brings in its configuration and tree

| Step | Statement |
| --- | --- |
| Given | the example the page opened with, with a stage removed |
| Given | a file added to the source |
| When | that example is chosen from the examples |
| Then | the configuration text and the tree are those the page opened with |

## `PGE-035` Adding in a folder starts the new path from that folder

| Step | Statement |
| --- | --- |
| Given | a source holding the folder `Alpha/Season 1` |
| When | adding in that folder is chosen |
| Then | the new path reads `Alpha/Season 1/` |

## `PGE-036` A parameter limited to some values offers them in a list

| Step | Statement |
| --- | --- |
| Given | the move stage selected |
| When | its `on_conflict` is opened |
| Then | the choices are `reject` and `suffix` |

## `PGE-037` A required parameter left empty is pointed out

| Step | Statement |
| --- | --- |
| Given | a next stage whose `into` is empty |
| When | its form is shown |
| Then | `into` is marked as required |

## `PGE-038` A folder configuration chosen in the tree is edited with the trees still shown

| Step | Statement |
| --- | --- |
| Given | a source holding `Alpha/auto-renamer.toml` |
| When | it is chosen in the tree |
| Then | the header says the folder configuration of `Alpha` is being edited, and the source and target trees are still shown |

## `PGE-039` The language chosen in the header is the one the page speaks

| Step | Statement |
| --- | --- |
| Given | the page shown in English |
| When | 繁體中文 is chosen in the header |
| Then | the page is shown in Traditional Chinese |

## `PGE-040` Settings the simulation ignores are marked as not simulated

| Step | Statement |
| --- | --- |
| Given | the watch `series` selected |
| When | its form is shown |
| Then | `batch_window` and `batch_max_wait` are marked as not simulated |

## `PGE-041` A watch lists pipelines picked from those defined

| Step | Statement |
| --- | --- |
| Given | a configuration defining the pipelines `video` and `new_pipeline` |
| Given | the watch `series` listing `video` |
| When | `new_pipeline` is picked in its form |
| Then | the watch lists `video` and `new_pipeline` |

## `PGE-042` A unit is chosen among its three forms

| Step | Statement |
| --- | --- |
| Given | the watch `series` selected |
| When | its `unit` is opened |
| Then | the choices are directory, source and root |

## `PGE-043` A yes-or-no setting is a switch

| Step | Statement |
| --- | --- |
| Given | the watch `series` selected |
| When | its form is shown |
| Then | `dry_run` is a switch |

## `PGE-044` A key is labelled in the page's language beside the CLI's key

| Step | Statement |
| --- | --- |
| Given | the page shown in Traditional Chinese |
| When | the watch `series` is selected |
| Then | its `dry_run` field is labelled 試運行 with `dry_run` beside it |

## `PGE-045` The canvas sits between the palette and the inspector, above the trees

| Step | Statement |
| --- | --- |
| Given | the global configuration shown |
| When | the page is laid out |
| Then | the palette, canvas and inspector run left to right above the source and target trees |

## `PGE-046` Every stage, parameter, setting and single value is named in both languages

| Step | Statement |
| --- | --- |
| Given | the stages, parameters and single values the core declares, and the settings the page offers |
| When | the page's texts in Traditional Chinese and English are read |
| Then | each has a name in both |

## `PGE-047` Every stage the core declares has an icon

| Step | Statement |
| --- | --- |
| Given | the stages the core declares |
| When | the page looks up the icon of each |
| Then | every stage has one |

## `PGE-048` A folder configuration edited in the form applies in the next simulation

| Step | Statement |
| --- | --- |
| Given | `Alpha/auto-renamer.toml` opened from the tree |
| Given | its variable `show` set to `Beta` in its form |
| When | the watch is triggered |
| Then | the target holds `Beta s01e01.mkv` |

## `PGE-049` Going back to the global configuration edits it again

| Step | Statement |
| --- | --- |
| Given | `Alpha/auto-renamer.toml` opened from the tree |
| When | the global configuration is chosen in the header |
| Then | the canvas draws the watch `series` again |

## `PGE-050` An imported folder configuration lands at the source root while the global one is edited

| Step | Statement |
| --- | --- |
| Given | the global configuration being edited |
| When | a file named `auto-renamer.toml` is imported |
| Then | the source holds `auto-renamer.toml` at its root, and it is the one being edited |

## `PGE-051` A folder configuration added in a folder is opened

| Step | Statement |
| --- | --- |
| Given | a source holding the folder `Alpha` |
| When | a folder configuration is added in `Alpha` |
| Then | the source holds `Alpha/auto-renamer.toml`, and it is the one being edited |

## `PGE-052` A stage is shown by its name beside the CLI's name

| Step | Statement |
| --- | --- |
| Given | the page shown in Traditional Chinese |
| When | the palette is shown |
| Then | the filter stage reads 篩選 with `filter` beside it |

## `PGE-053` An imported folder configuration replaces the one being edited

| Step | Statement |
| --- | --- |
| Given | `Alpha/auto-renamer.toml` opened from the tree |
| When | a file named `auto-renamer.toml` is imported |
| Then | the imported text is what `Alpha/auto-renamer.toml` holds, and no other folder configuration is added |

## `PGE-054` A download is the configuration being edited

| Step | Statement |
| --- | --- |
| Given | `Alpha/auto-renamer.toml` opened from the tree |
| When | the download is chosen |
| Then | the file offered is named `auto-renamer.toml` |

## `PGE-055` The page opens on the single-episode example

| Step | Statement |
| --- | --- |
| Given | the page just opened |
| When | the trees are shown |
| Then | the source holds the single-episode files of each show, each show folder with its folder configuration |

## `PGE-056` A pipeline's stages are stacked below it in order

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` with the stages filter, format and move |
| When | it is drawn |
| Then | each stage is drawn in the pipeline's column, below the one before it |

## `PGE-057` Every example moves its files where the cases say

| Step | Statement |
| --- | --- |
| Given | an example the page offers |
| When | its watch is triggered |
| Then | each of its files goes where `docs/cases.md` says, or stays where the cases leave it |

## `PGE-058` A joint's remove button drops the pipeline from the watch

| Step | Statement |
| --- | --- |
| Given | the watch `series` joined to the pipeline `video` |
| When | the remove button on that joint is pressed |
| Then | the joint is gone and the configuration text lists no `video` for the watch |
