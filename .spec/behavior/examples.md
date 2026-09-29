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
