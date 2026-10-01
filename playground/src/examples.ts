// The examples the page offers: each is a configuration, the virtual tree it runs over, and the watch to trigger.
// They follow the cases of `docs/cases.md`, so a triggered example moves its files where the cases say.

import type { Entry } from './core'

export interface Example {
  id: string
  config: string
  watch: string
  entries: Entry[]
}

// The modification time is irrelevant to every example but the photos, so the rest share one.
const MODIFIED = Date.UTC(2026, 8, 27)

function file(path: string, text = '', modified = MODIFIED): Entry {
  return { path, folder: false, modified, text }
}

function show(name: string): string {
  return `vars = { show = "${name}" }\n`
}

// The series pipelines of `design.md` 5.13.
const VIDEO = `[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  { number = { into = "episode", exclude = ["season"] } },
  { default = { season = 1 } },
  { lift = { to = "Season *" } },
  { next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } },
  { format = "{show} s{season:02}e{episode:02}" },
  "move",
  { cleanup = { keep = ["Season *"] } },
]
`

const SINGLE: Example = {
  id: 'single',
  watch: 'series',
  config: `[default]
pipelines = ["video"]

${VIDEO}
[watch.series]
source = "/downloads"
unit = { root = ["Series/*"] }
`,
  entries: [
    ['Alpha', '[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv'],
    ['Beta Show', '[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4'],
    ['Gamma Show', '[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv'],
    ['Delta-Show Kai', '[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv'],
    ['Epsilon', '[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4'],
    ['Zeta-Show', 'Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv'],
  ].flatMap(([name, path]) => [
    file(`/downloads/Series/${name}/auto-renamer.toml`, show(name)),
    file(`/downloads/Series/${name}/${path}`),
  ]),
}

export const EXAMPLES: Example[] = [SINGLE]

/** The example the page opens with. */
export const FIRST = SINGLE
