# Page

The playground page draws a configuration for people who do not read TOML: each watch is joined to the pipelines its routes run, and each pipeline is its stages in order. The table the core reads is the baseline; the drawing and every edit are made from it and back into it, so what is downloaded is what was drawn. Where a node sits is only its layout, which is neither written nor kept.

## Includes

- `playground/src/**/*.test.ts`
- `playground/e2e/*.e2e.ts`

## `PGE-001` A pipeline is drawn as its stages in order

| Step | Statement |
| --- | --- |
| Given | a configuration whose pipeline `video` has the stages filter, format and strip |
| When | it is drawn |
| Then | the pipeline holds the filter leading to the format, and the format leading to the strip |

## `PGE-002` A watch holds its routes in order, each joined to its pipeline

| Step | Statement |
| --- | --- |
| Given | a configuration whose watch `series` has routes of `video` and `subtitle` |
| When | it is drawn |
| Then | the watch holds a route of `video` above a route of `subtitle`, each leading to its pipeline |

## `PGE-003` A watch without its own routes is joined to the default's

| Step | Statement |
| --- | --- |
| Given | a configuration whose default has a route of `video` and whose watch has none |
| When | it is drawn |
| Then | the watch holds a route of `video`, drawn as followed from the default |

## `PGE-004` A stage added to a pipeline goes after the others

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter and strip |
| When | a format stage is added to it |
| Then | its stages are filter, strip and format |

## `PGE-005` A stage moved earlier is written earlier

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter, strip and format |
| When | the format stage is moved one place earlier |
| Then | its stages are filter, format and strip |

## `PGE-006` A removed stage leaves the others in order

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter, format and strip |
| When | the format stage is removed |
| Then | its stages are filter and strip |

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
| When | a strip stage is added with no parameters |
| Then | the stage is written as the text `strip` |

## `PGE-009` A configuration that is refused cannot be downloaded

| Step | Statement |
| --- | --- |
| Given | a configuration the core refuses |
| When | a download is asked for |
| Then | no file is offered and the reason is shown |

## `PGE-010` A stage dropped between two stages is written between them

| Step | Statement |
| --- | --- |
| Given | a pipeline with the stages filter and strip |
| When | a format stage is dropped between them |
| Then | its stages are filter, format and strip |

## `PGE-011` A watch joined to a pipeline routes it last

| Step | Statement |
| --- | --- |
| Given | a watch with a route of `video` |
| When | it is joined to `subtitle` |
| Then | it has routes of `video` and `subtitle` |

## `PGE-012` A watch following the default has its own routes once joined

| Step | Statement |
| --- | --- |
| Given | a default with a route of `video` and a watch with none |
| When | the watch is joined to `subtitle` |
| Then | the watch has its own routes of `video` and `subtitle` |

## `PGE-013` A joint removed drops the pipeline from the watch

| Step | Statement |
| --- | --- |
| Given | a watch with routes of `video` and `subtitle` |
| When | its joint to `video` is removed |
| Then | it has a route of `subtitle` only |

## `PGE-014` A renamed pipeline is renamed where it is listed

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` routed by the default and by a watch |
| When | it is renamed `episode` |
| Then | the routes of the default and the watch run `episode` and not `video` |

## `PGE-015` A removed pipeline is listed nowhere

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` routed by the default and by a watch |
| When | it is removed |
| Then | no route of the default or the watch runs `video` |

## `PGE-016` A new stage starts with the example the core describes

| Step | Statement |
| --- | --- |
| Given | the stages the core describes |
| When | a regex stage is added |
| Then | the stage is written with the regex example |

## `PGE-017` The virtual tree is shown as the watch's source and target

| Step | Statement |
| --- | --- |
| Given | a watch from `/downloads` whose route moves into a target at `/video`, and files under both |
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
| Given | a strip stage whose `groups` is set |
| When | `groups` is cleared in the form |
| Then | the stage is written as the text `strip` |

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
| Given | a case stage added and selected |
| When | its `to` is opened |
| Then | the choices are `lower`, `upper` and `title` |

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

## `PGE-040` A watch offers the settings for when a unit is processed

| Step | Statement |
| --- | --- |
| Given | the watch `series` selected |
| When | its form is shown |
| Then | `quiet`, `max_wait` and `max_files` are offered |

## `PGE-041` A watch routes pipelines picked from those defined

| Step | Statement |
| --- | --- |
| Given | a configuration defining the pipelines `video` and `new_pipeline` |
| Given | the watch `series` with a route of `video` |
| When | `new_pipeline` is picked in its form |
| Then | the watch has routes of `video` and `new_pipeline` |

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

## `PGE-045` The canvas sits between the palette and the inspector, above the panel

| Step | Statement |
| --- | --- |
| Given | the page just opened |
| When | it is laid out |
| Then | the palette, canvas and inspector run left to right above one panel holding the trees and the results |

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
| Then | the source holds the single-episode files of each show, named by its folder |

## `PGE-056` A pipeline's stages run inside it from left to right

| Step | Statement |
| --- | --- |
| Given | a pipeline `video` with the stages filter, format and strip |
| When | it is drawn |
| Then | each stage is drawn inside the pipeline, right of the one before it |

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
| Then | the joint is gone and the configuration text has no route of `video` for the watch |

## `PGE-059` A new folder configuration starts with the watch's values

| Step | Statement |
| --- | --- |
| Given | a default with `vars = { group = "Team" }` and the watch `series` with `vars = { show = "Alpha" }` |
| When | a folder configuration is started for `series` |
| Then | its text sets `group = "Team"` and `show = "Alpha"` and nothing else |

## `PGE-060` Adding a folder configuration is offered without hovering

| Step | Statement |
| --- | --- |
| Given | the source tree of the opening example |
| When | it is shown |
| Then | each folder shows its button adding a folder configuration, and the tree says how to add one |

## `PGE-061` The header lists every folder configuration to edit

| Step | Statement |
| --- | --- |
| Given | the opening example with a folder configuration started for `Alpha` |
| When | the configurations are listed in the header |
| Then | the global configuration and the folder configuration of `Alpha` are offered |

## `PGE-062` A folder configuration added from the header is started and opened

| Step | Statement |
| --- | --- |
| Given | the folder `Alpha` without a folder configuration |
| When | adding one for `Alpha` is chosen in the header |
| Then | the source holds `Alpha/auto-renamer.toml`, and it is the one being edited |

## `PGE-063` A pipeline overridden in a folder is copied into its folder configuration

| Step | Statement |
| --- | --- |
| Given | a folder configuration holding `vars = { show = "Alpha" }` and a pipeline `subtitle` |
| When | the global pipeline `video` is overridden in that folder |
| Then | it holds the same values, its `subtitle`, and a `video` with the global stages |

## `PGE-064` Overriding a pipeline in a folder opens that folder's configuration

| Step | Statement |
| --- | --- |
| Given | the pipeline `series-video` selected in the global configuration |
| When | it is overridden in the folder `Alpha` |
| Then | `Alpha/auto-renamer.toml` is the one being edited, and its canvas draws `series-video` |

## `PGE-065` A file's timeline marks only what each step changed

| Step | Statement |
| --- | --- |
| Given | the steps of a file whose number stage set `episode` and whose format stage rewrote `name` |
| When | its timeline is drawn |
| Then | the number step shows only `episode`, and the format step shows only `name` from the old to the new value |

## `PGE-066` Choosing a result shows how its file was planned

| Step | Statement |
| --- | --- |
| Given | the opening example triggered |
| When | a file is chosen among the results |
| Then | its steps are shown in order, from the pipeline that claimed it to the stage that named it |

## `PGE-067` A stage lists the files it ran on, apart from those a folder configuration took

| Step | Statement |
| --- | --- |
| Given | a simulation where one file ran the global `video` and another ran a `video` its folder configuration replaced |
| When | the files after the second stage of the global `video` are asked for |
| Then | the first file is listed with what it held after that stage, and the second is named as taken by a folder configuration |

## `PGE-068` A selected stage shows the files after it

| Step | Statement |
| --- | --- |
| Given | the opening example triggered |
| When | the format stage of `video` is selected |
| Then | the panel shows the stage's tab, listing each file with each field the format changed, before and after |

## `PGE-069` Editing the configuration clears the simulation

| Step | Statement |
| --- | --- |
| Given | the opening example triggered |
| When | a stage is removed |
| Then | no result is shown until the watch is triggered again |

## `PGE-070` A leading filter lists the files it claimed

| Step | Statement |
| --- | --- |
| Given | a simulation where the leading filter of `video` claimed a video and left a subtitle |
| When | the files after that filter are asked for |
| Then | the video is listed and the subtitle is not |

## `PGE-071` A target added in the defaults form is declared

| Step | Statement |
| --- | --- |
| Given | the global configuration with nothing selected |
| When | a new target `conflict` is named in the form |
| Then | the configuration declares `target.conflict` |

## `PGE-072` A route moves into a target picked from those declared

| Step | Statement |
| --- | --- |
| Given | the target `conflict` declared and the first route of the watch `series` selected |
| When | `conflict` is picked as where it moves |
| Then | the route moves into `conflict` |

## `PGE-073` A route is joined to the target it moves to

| Step | Statement |
| --- | --- |
| Given | a configuration declaring the target `dst`, whose watch has a route moving to `dst` |
| When | it is drawn |
| Then | `dst` is drawn and the route leads to it |

## `PGE-074` A rejected route is joined apart from its route

| Step | Statement |
| --- | --- |
| Given | a route whose rejected route runs `fallback` and moves to `conflict` |
| When | it is drawn |
| Then | the route leads to `fallback` and to `conflict` by dashed joints |

## `PGE-075` A built-in pipeline a route names is drawn locked with its stages

| Step | Statement |
| --- | --- |
| Given | a route of `series-video`, which the configuration does not define |
| When | it is drawn |
| Then | `series-video` is drawn with the stages the core builds in, and can be neither changed nor removed on the canvas |

## `PGE-076` A pipeline defined under a built-in's name is drawn as its own

| Step | Statement |
| --- | --- |
| Given | a configuration defining `series-video` itself |
| When | it is drawn |
| Then | `series-video` is drawn with the stages the configuration gives it, open to change |

## `PGE-077` A stage dropped on a built-in pipeline is not added

| Step | Statement |
| --- | --- |
| Given | a drawn built-in pipeline |
| When | a stage is dropped on it |
| Then | no pipeline is chosen to take the stage |

## `PGE-078` A route moved above another in its watch claims before it

| Step | Statement |
| --- | --- |
| Given | a watch whose routes run `video` then `subtitle` |
| When | the route of `subtitle` is moved above the route of `video` |
| Then | the watch's routes run `subtitle` then `video` |

## `PGE-079` A route joined to a target moves there

| Step | Statement |
| --- | --- |
| Given | a route of `video` without a move, and the declared target `dst` |
| When | the route is joined to `dst` |
| Then | the route moves to `dst` |

## `PGE-080` A removed route leaves the watch's other routes in order

| Step | Statement |
| --- | --- |
| Given | a watch whose routes run `video`, `subtitle` and `extra` |
| When | the route of `subtitle` is removed |
| Then | the watch's routes run `video` then `extra` |

## `PGE-081` A removed target is moved to by no route

| Step | Statement |
| --- | --- |
| Given | routes moving to `dst`, one of them sending what it refuses to `dst` too, in the default and in a watch |
| When | the target `dst` is removed |
| Then | no route nor rejected route moves to `dst`, and each keeps its pipeline |

## `PGE-082` A route's move joint removed renames in place

| Step | Statement |
| --- | --- |
| Given | a route of `video` moving to `dst` |
| When | its joint to `dst` is removed |
| Then | the route has no move |

## `PGE-083` A refused file is sent through a pipeline picked from those defined or built in

| Step | Statement |
| --- | --- |
| Given | the first route of the watch `series` selected |
| When | `series-subtitle` is picked as the pipeline its rejected route runs |
| Then | the route's rejected route runs `series-subtitle` |

## `PGE-084` A route that cleans up keeps the folders it lists

| Step | Statement |
| --- | --- |
| Given | a selected route that does not clean up |
| When | it is set to clean up, keeping `Season *` |
| Then | the route cleans up and keeps `Season *` |

## `PGE-085` A built-in pipeline copied becomes the configuration's own

| Step | Statement |
| --- | --- |
| Given | a configuration routing the built-in `series-video` |
| When | `series-video` is copied |
| Then | the configuration defines `series-video` with the stages the core builds in |

## `PGE-086` A built-in pipeline overridden in a folder is copied from the core

| Step | Statement |
| --- | --- |
| Given | a global configuration routing the built-in `series-video` without defining it |
| When | `series-video` is overridden in a folder |
| Then | the folder configuration defines `series-video` with the stages the core builds in |

## `PGE-087` A file a rejected route claims without a pipeline is told so

| Step | Statement |
| --- | --- |
| Given | a route whose pipeline refuses a file, and whose rejected route runs no pipeline |
| When | the watch is simulated |
| Then | the file's timeline says the rejected route claimed it, rather than a pipeline without a name |

## `PGE-088` Routes whose order changed are drawn in their order

| Step | Statement |
| --- | --- |
| Given | a watch whose second route was moved by hand |
| When | its routes are put in another order |
| Then | each of its routes is drawn where its order places it |

## `PGE-089` A default route is edited in full in the defaults form

| Step | Statement |
| --- | --- |
| Given | the target `conflict` declared and nothing selected |
| When | a route of `series-video` is added to the default's routes and set to move to `conflict` |
| Then | the default's route of `series-video` moves to `conflict` |

## `PGE-090` The target of a rejected route is a root of the virtual tree

| Step | Statement |
| --- | --- |
| Given | a route moving to `video` whose rejected route moves to `conflict` |
| When | the roots of the watch are taken |
| Then | the targets are the roots of `video` and `conflict` |

## `PGE-091` A folder configuration already there is opened as it is

| Step | Statement |
| --- | --- |
| Given | `Alpha/auto-renamer.toml` giving `show` as Beta |
| When | a folder configuration is started in `Alpha` |
| Then | its text is unchanged |

## `PGE-092` An imported global configuration replaces the one being edited

| Step | Statement |
| --- | --- |
| Given | the global configuration being edited |
| When | a `config.toml` declaring the watch `movies` is imported |
| Then | the configuration declares `movies` |

## `PGE-093` A removed watch is written nowhere

| Step | Statement |
| --- | --- |
| Given | the watch `series` selected |
| When | it is removed |
| Then | the configuration has no watch `series` |

## `PGE-094` A selected target's path is edited in its form

| Step | Statement |
| --- | --- |
| Given | the target `conflict` declared and selected |
| When | its path is set to `/clash` |
| Then | the target `conflict` has the path `/clash` |

## `PGE-095` A refused file moves into a target picked from those declared

| Step | Statement |
| --- | --- |
| Given | the target `conflict` declared and the first route of the watch `series` selected |
| When | `conflict` is picked as where its refused files move |
| Then | the route's rejected route moves into `conflict` |

## `PGE-096` A file a simulation took is marked where it was

| Step | Statement |
| --- | --- |
| Given | a simulation that moved `/downloads/Alpha/x.mkv` to `/video/Alpha/Alpha.mkv` |
| When | the trees are drawn from it |
| Then | `/downloads/Alpha/x.mkv` is marked as moved |

## `PGE-097` Each root counts the files a simulation changed in it

| Step | Statement |
| --- | --- |
| Given | a simulation that moved one file of `/downloads` to `/video` and left another that nothing claims |
| When | the changes of each root are counted |
| Then | `/downloads` and `/video` each count one |

## `PGE-098` A tree's tab counts the files a simulation changed in it

| Step | Statement |
| --- | --- |
| Given | the opening example |
| When | its watch is triggered |
| Then | the tabs of the source and of the target each show how many of their files changed |

## `PGE-099` Triggering a watch shows its results

| Step | Statement |
| --- | --- |
| Given | the source tree shown in the panel |
| When | the watch is triggered |
| Then | the panel shows where each file went |

## `PGE-100` A stage's tab leaves with the stage

| Step | Statement |
| --- | --- |
| Given | the opening example triggered and its format stage selected |
| When | the canvas is clicked where nothing is drawn |
| Then | the stage's tab is gone and the panel shows the results |
