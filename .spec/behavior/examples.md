# Examples

The examples of the design, each run through the pipeline it gives. They show that the stages, put together, make the names the design promises.

## Includes

- `tests/examples.rs`

## `EX-001` A number after a dash

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Alpha, and the file `Series/Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Alpha/Alpha s01e12.mkv` |

## `EX-002` A number in brackets among other brackets

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Beta Show, and the file `Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4` |
| When | the pipeline is planned |
| Then | the plan is `Series/Beta Show/Beta Show s01e12.mp4` |

## `EX-003` A version, a hash and a year beside the number

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Gamma Show, and the file `Series/Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Gamma Show/Gamma Show s01e18.mkv` |

## `EX-004` A resolution and a codec after the title

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Delta-Show Kai, and the file `Series/Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Delta-Show Kai/Delta-Show Kai s01e13.mkv` |

## `EX-005` A four digit number beside a date

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Epsilon, and the file `Series/Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4` |
| When | the pipeline is planned |
| Then | the plan is `Series/Epsilon/Epsilon s01e1354.mp4` |

## `EX-006` A season in the folder and season marks in the name

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Zeta-Show, and the file `Series/Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Zeta-Show/Season 03/Zeta-Show s03e10.mkv` |

## `EX-007` A tag is removed from a movie

| Step | Statement |
| --- | --- |
| Given | the movie pipeline and the file `Movies/XXX/[Group] XXX [1080p].mkv` |
| When | the pipeline is planned |
| Then | the plan is `Movies/XXX/XXX.mkv` |

## `EX-008` A year in parentheses stays in a movie name

| Step | Statement |
| --- | --- |
| Given | the movie pipeline and the file `Movies/Movie (2026)/[Group] Movie (2026).mkv` |
| When | the pipeline is planned |
| Then | the plan is `Movies/Movie (2026)/Movie (2026).mkv` |

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

## `EX-011` A photo of another month goes into that month

| Step | Statement |
| --- | --- |
| Given | the photo pipeline and the file `Photos/IMG_0002.jpg` modified on 2026-10-01 |
| When | the pipeline is planned |
| Then | the plan is `Photos/2026/10/IMG_0002.jpg` |

## `EX-012` The first episode is not taken for the default season

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Alpha, and the file `Series/Alpha/[Team] Alpha [01].mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Alpha/Alpha s01e01.mkv` |

## `EX-013` A number that cannot be told falls back to the next one

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Eta Show, an empty target, and the file `Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4` |
| When | the pipeline is planned |
| Then | the plan is `Series/Eta Show/Season 17/Eta Show s17e01.mp4` |


## `EX-014` A file without a number follows the target

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Show, a target folder `Series/Show` holding `Show s01e01.mkv` and `Show s01e02.mkv`, and the file `Series/Show/Show new a.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Show/Show s01e03.mkv` |


## `EX-015` The number follows the folder the file is lifted into

| Step | Statement |
| --- | --- |
| Given | the series pipeline, `show` Show, a target folder `Series/Show/Season 01` holding `Show s01e05.mkv` and a target folder `Series/Show/Season 02` holding `Show s02e09.mkv`, and the file `Series/Show/Season 01/[Rel]/new.mkv` |
| When | the pipeline is planned |
| Then | the plan is `Series/Show/Season 01/Show s01e06.mkv` |

## `EX-016` Two episodes and their subtitles in one batch

| Step | Statement |
| --- | --- |
| Given | the video and subtitle pipelines of the design, and in `Series/Show/Season 01/` the files `Show 27.mkv`, `Show 27.cht.ass`, `Show 27.chs.ass`, `Show 28.mkv` and `Show 28.cht.ass`, with `show` Show |
| When | the batch is planned |
| Then | the plans are `Show s01e27.mkv`, `Show s01e27.zh.01.ass`, `Show s01e27.zh.02.ass`, `Show s01e28.mkv` and `Show s01e28.zh.ass`, all in `Season 01` |


## `EX-017` A folder made for one release is emptied into the season

| Step | Statement |
| --- | --- |
| Given | the video and subtitle pipelines of the design, `show` Theta_Show, and in `Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/` the files `[Team][Theta_Show][27][1080p].mkv`, `[Team][Theta_Show][27][1080p].cht.ass` and `[Team][Theta_Show][27][1080p].ass` |
| When | the batch is planned |
| Then | the plans are `Theta_Show s01e27.mkv`, `Theta_Show s01e27.zh.01.ass` and `Theta_Show s01e27.zh.02.ass`, all in `Season 01` |


## `EX-018` Subtitles in a folder of the release follow their video

| Step | Statement |
| --- | --- |
| Given | the video and subtitle pipelines of the design, `show` Show, and the files `Series/Show/Season 01/[Rel 05]/Show 05.mkv` and `Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass` |
| When | the batch is planned |
| Then | the plans are `Show s01e05.mkv` and `Show s01e05.zh.ass`, both in `Season 01` |


## `EX-019` Files without a number continue the target in name order

| Step | Statement |
| --- | --- |
| Given | the video pipeline of the design, `show` Show, a target folder `Series/Show` holding `Show s01e01.mkv` and `Show s01e02.mkv`, and the files `Show new a.mkv`, `Show new b.mkv` and `Show 07.mkv` in `Series/Show/` |
| When | the batch is planned |
| Then | the plans are `Show s01e07.mkv`, `Show s01e03.mkv` and `Show s01e04.mkv` in that order |


## `EX-020` A file no pipeline claims is left

| Step | Statement |
| --- | --- |
| Given | the video and subtitle pipelines of the design, `show` Show, and the files `Series/Show/Show 05.mkv` and `Series/Show/notes.nfo` |
| When | the batch is planned |
| Then | `Show 05.mkv` is planned and `notes.nfo` is unclaimed |
