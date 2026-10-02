# Examples

The examples of the design, each run through the pipeline it gives. They show that the stages, put together, make the names the design promises.

## Includes

- `tests/examples.rs`

## `EX-006` A season in the folder and season marks in the name

| Step | Statement |
| --- | --- |
| Given | the `series-video` pipeline, `show` Zeta-Show, and the file `Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Zeta-Show/Season 03/Zeta-Show s03e10.mkv` |

## `EX-007` A tag is removed from a movie

| Step | Statement |
| --- | --- |
| Given | the movie pipeline and the file `Movies/XXX/[Group] XXX [1080p].mkv` |
| When | the pipeline is planned |
| Then | the plan is `Movies/XXX/XXX.mkv` |

## `EX-009` A track number and title are rewritten

| Step | Statement |
| --- | --- |
| Given | the music pipeline and the file `Music/Artist/Album/03 - Title.mp3` |
| When | the pipeline is planned |
| Then | the plan is `Music/Artist/Album/03 Title.mp3` |

## `EX-010` A photo goes into a folder for its month

| Step | Statement |
| --- | --- |
| Given | the photo pipeline and the file `Photos/IMG_0001.jpg` modified on 2026-09-27 |
| When | the pipeline is planned |
| Then | the plan is `Photos/2026/09/IMG_0001.jpg` |

## `EX-012` The first episode is not taken for the default season

| Step | Statement |
| --- | --- |
| Given | the `series-video` pipeline, `show` Alpha, and the file `Alpha/[Team] Alpha [01].mkv` |
| When | the pipeline is planned |
| Then | the plan is `Alpha/Season 01/Alpha s01e01.mkv` |

## `EX-013` A number that cannot be told falls back to the next one

| Step | Statement |
| --- | --- |
| Given | the `series-video` pipeline, `show` Eta Show, an empty target, and the file `Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4` |
| When | the pipeline is planned |
| Then | the plan is `Eta Show/Season 17/Eta Show s17e01.mp4` |


## `EX-015` The number follows the folder the file is lifted into

| Step | Statement |
| --- | --- |
| Given | the `series-video` pipeline, `show` Show, a target folder `Show/Season 01` holding `Show s01e05.mkv` and a target folder `Show/Season 02` holding `Show s02e09.mkv`, and the file `Show/Season 01/[Rel]/new.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Show/Season 01/Show s01e06.mkv` |

## `EX-016` Two episodes and their subtitles in one batch

| Step | Statement |
| --- | --- |
| Given | the `series-video` and `series-subtitle` pipelines, and in `Show/Season 01/` the files `Show 27.mkv`, `Show 27.cht.ass`, `Show 27.chs.ass`, `Show 28.mkv` and `Show 28.cht.ass`, with `show` Show |
| When | the batch is planned |
| Then | the plans are `Show s01e27.mkv`, `Show s01e27.zh.01.ass`, `Show s01e27.zh.02.ass`, `Show s01e28.mkv` and `Show s01e28.zh.ass`, all in `Season 01` |


## `EX-018` Subtitles in a folder of the release follow their video

| Step | Statement |
| --- | --- |
| Given | the `series-video` and `series-subtitle` pipelines, `show` Show, and the files `Show/Season 01/[Rel 05]/Show 05.mkv` and `Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass` |
| When | the batch is planned |
| Then | the plans are `Show s01e05.mkv` and `Show s01e05.zh.ass`, both in `Season 01` |


## `EX-019` Files without a number continue the target in name order

| Step | Statement |
| --- | --- |
| Given | the `series-video` pipeline, `show` Show, a target folder `Show/Season 01` holding `Show s01e01.mkv` and `Show s01e02.mkv`, and the files `Show new a.mkv`, `Show new b.mkv` and `Show 07.mkv` in `Show/` |
| When | the batch is planned |
| Then | the plans are `Show s01e07.mkv`, `Show s01e03.mkv` and `Show s01e04.mkv` in that order, all in `Show/Season 01` |

## `EX-020` A folder per episode ends up in its season folder

| Step | Statement |
| --- | --- |
| Given | the `series-video` and `series-subtitle` pipelines, `show` Alpha, and the files `Alpha/[Team] Alpha - 01 [1080p]/Alpha 01.mkv` and `Alpha/[Team] Alpha - 01 [1080p]/Alpha 01.cht.ass` |
| When | they are planned in one batch |
| Then | the plans are `Alpha/Season 01/Alpha s01e01.mkv` and `Alpha/Season 01/Alpha s01e01.zh.ass` |
