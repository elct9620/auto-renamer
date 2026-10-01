// The examples the page offers: each is a configuration, the virtual tree it runs over, and the watch to trigger.
// They follow `docs/cases.md`, so a triggered example moves its files where the cases say.

import type { Entry } from './core'

export interface Example {
  id: string
  config: string
  watch: string
  entries: Entry[]
}

// Only the photos are sorted by their modification time; every other file shares this one.
const MODIFIED = Date.UTC(2026, 8, 27)

function file(path: string, text = '', modified = MODIFIED): Entry {
  return { path, folder: false, modified, text }
}

function show(name: string): string {
  return `vars = { show = "${name}" }\n`
}

const EPISODE = '{ number = { into = "episode", exclude = ["season"] } },'

// The series pipelines of `design.md` 5.13; `episode` and `move` are the stages examples vary.
function video({ episode = EPISODE, move = '"move"' } = {}): string {
  return `[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { number = { from = "path", into = "season", prefix = "Season" } },${episode ? `\n  ${episode}` : ''}
  { default = { season = 1 } },
  { lift = { to = "Season *" } },
  { next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } },
  { format = "{show} s{season:02}e{episode:02}" },
  ${move},
  { cleanup = { keep = ["Season *"] } },
]
`
}

function subtitle(move = '"move"'): string {
  return `[pipeline.subtitle]
stages = [
  { filter = { ext = ["ass", "srt"] } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  ${EPISODE}
  { default = { season = 1 } },
  { rank = { into = "index", by = ["season", "episode"], prefer = ["cht"] } },
  { lift = { to = "Season *" } },
  { format = "{show} s{season:02}e{episode:02}.zh[.{index:02}]" },
  ${move},
]
`
}

function series(pipelines: string, watch: string): string {
  return `[default]
pipelines = [${pipelines}]

${watch}`
}

const SUFFIX_AI = '{ move = { on_conflict = "suffix", suffix = "_ai" } }'

// Every series scenario at once, with the source and the target apart: seasons named by their folders,
// episodes found among noise or counted on, a release folder with its subtitles, a release too large,
// a clash, a name for each season, and a folder that always counts on.
const FULL: Example = {
  id: 'full',
  watch: 'series',
  config: `${series('"video", "subtitle"', `${video({ move: SUFFIX_AI })}
${subtitle(SUFFIX_AI)}`)}
[watch.series]
source = "/downloads"
target = "/video"
batch_max = 5
`,
  entries: [
    file('/downloads/Series/Alpha/Season 01/auto-renamer.toml', show('Alpha')),
    file('/downloads/Series/Alpha/Season 01/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv'),
    file('/downloads/Series/Alpha/Season 02/auto-renamer.toml', show('Alpha Next')),
    file('/downloads/Series/Alpha/Season 02/[Team] Alpha Next - 03 [1080p].mkv'),
    file('/downloads/Series/Beta Show/auto-renamer.toml', show('Beta Show')),
    file('/downloads/Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4'),
    file('/downloads/Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].ass'),
    file('/video/Series/Beta Show/Beta Show s01e12.mp4'),
    file('/downloads/Series/Eta Show/auto-renamer.toml', show('Eta Show')),
    file('/downloads/Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
    file('/downloads/Series/Theta_Show/auto-renamer.toml', show('Theta_Show')),
    file('/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv'),
    file('/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass'),
    file('/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass'),
    ...[1, 2, 3, 4, 5, 6].map((n) =>
      file(`/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][Batch]/[Team][Theta_Show][0${n}][1080p].mkv`)),
    file('/downloads/Series/Kappa/Season 01/auto-renamer.toml', `${show('Kappa')}\n${video({ episode: '' })}`),
    file('/downloads/Series/Kappa/Season 01/[Group] Kappa - 07 [1080p].mkv'),
    file('/video/Series/Kappa/Season 01/Kappa s01e01.mkv'),
    file('/video/Series/Kappa/Season 01/Kappa s01e02.mkv'),
  ],
}

const SINGLE: Example = {
  id: 'single',
  watch: 'series',
  config: `${series('"video"', video())}
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

const UNSURE: Example = {
  id: 'unsure',
  watch: 'series',
  config: SINGLE.config,
  entries: [
    file('/downloads/Series/Eta Show/auto-renamer.toml', show('Eta Show')),
    file('/downloads/Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
    file('/downloads/Series/Eta Nth/auto-renamer.toml',
      `${show('Eta Show')}\n${video({ episode: '{ number = { into = "episode", exclude = ["season"], nth = 2 } },' })}`),
    file('/downloads/Series/Eta Nth/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
  ],
}

// The configurations of the cases that rename a season folder of one show in place.
function season(pipelines: string, unit: string, target = ''): string {
  return `${series(pipelines, `${video()}\n${subtitle()}`)}
[watch.series]
source = "/downloads"${target ? `\ntarget = "${target}"` : ''}
unit = ${unit}
vars = { show = "Show" }
`
}

const SUBTITLES: Example = {
  id: 'subtitles',
  watch: 'series',
  config: season('"video", "subtitle"', '"directory"'),
  entries: ['Show 27.mkv', 'Show 27.cht.ass', 'Show 27.chs.ass', 'Show 28.mkv', 'Show 28.cht.ass']
    .map((name) => file(`/downloads/Series/Show/Season 01/${name}`)),
}

const RELEASE: Example = {
  id: 'release',
  watch: 'series',
  config: season('"video", "subtitle"', '{ root = ["Series/*/*"] }').replace('show = "Show"', 'show = "Theta_Show"'),
  entries: ['.mkv', '.cht.ass', '.ass']
    .map((ext) => file(`/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p]${ext}`)),
}

const SUBS: Example = {
  id: 'subs',
  watch: 'series',
  config: season('"video", "subtitle"', '{ root = ["Series/*/*"] }'),
  entries: [
    file('/downloads/Series/Show/Season 01/[Rel 05]/Show 05.mkv'),
    file('/downloads/Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass'),
  ],
}

const NEXT: Example = {
  id: 'next',
  watch: 'series',
  config: season('"video"', '"directory"', '/video'),
  entries: [
    file('/video/Series/Show/Season 01/Show s01e01.mkv'),
    file('/video/Series/Show/Season 01/Show s01e02.mkv'),
    ...['Show new a.mkv', 'Show new b.mkv', 'Show 07.mkv'].map((name) => file(`/downloads/Series/Show/Season 01/${name}`)),
  ],
}

const LIFTED: Example = {
  id: 'lifted',
  watch: 'series',
  config: season('"video"', '{ root = ["Series/*/*"] }', '/video'),
  entries: [
    file('/video/Series/Show/Season 01/Show s01e05.mkv'),
    file('/video/Series/Show/Season 02/Show s02e09.mkv'),
    file('/downloads/Series/Show/Season 01/[Rel]/new.mkv'),
  ],
}

const CLASH: Example = {
  id: 'clash',
  watch: 'series',
  config: season('"video"', '"directory"', '/video').replace('  "move",\n  { cleanup', '  { move = { on_conflict = "suffix" } },\n  { cleanup'),
  entries: [
    file('/video/Series/Show/Season 01/Show s01e12.mkv'),
    file('/downloads/Series/Show/Season 01/Show 12.mkv'),
  ],
}

const TOO_LARGE: Example = {
  id: 'large',
  watch: 'series',
  config: season('"video"', '"directory"').replace('vars = {', 'batch_max = 5\nvars = {'),
  entries: [1, 2, 3, 4, 5, 6].map((n) => file(`/downloads/Series/Show/Season 01/Show 0${n}.mkv`)),
}

// The other pipelines of `design.md` 5.14, each alone with the files of its case.
function other(name: string, stages: string, entries: Entry[]): Example {
  return {
    id: name,
    watch: name,
    config: `[pipeline.${name}]\nstages = ${stages}\n\n[watch.${name}]\nsource = "/downloads"\npipelines = ["${name}"]\n`,
    entries,
  }
}

const MOVIE = other('movie', '[{ filter = { ext = ["mkv", "mp4"] } }, { strip = {} }, "move"]', [
  file('/downloads/Movies/XXX/[Group] XXX [1080p].mkv'),
  file('/downloads/Movies/Movie (2026)/[Group] Movie (2026).mkv'),
])

const MUSIC = other('music', `[
  { filter = { ext = ["mp3", "flac"] } },
  { regex = { pattern = '^(?<track>\\d+)\\s*-\\s*(?<title>.+)$' } },
  { format = "{track:02} {title}" },
  "move",
]`, [
  file('/downloads/Music/Artist/Album/03 - Title.mp3'),
  file('/downloads/Music/Artist/Album/cover.jpg'),
])

const PHOTO = other('photo', `[
  { filter = { ext = ["jpg", "png"] } },
  { folder = "{mtime:%Y}/{mtime:%m}" },
  "move",
]`, [
  file('/downloads/Photos/IMG_0001.jpg', '', Date.UTC(2026, 8, 27)),
  file('/downloads/Photos/IMG_0002.jpg', '', Date.UTC(2026, 9, 1)),
])

// The subtitles clean up after themselves too, so a `Subs` folder they leave empty is removed.
const CLEANUP: Example = {
  id: 'cleanup',
  watch: 'series',
  config: season('"video", "subtitle"', '{ root = ["Series/*/*"] }')
    .replace('  "move",\n]', '  "move",\n  { cleanup = { keep = ["Season *"] } },\n]'),
  entries: [
    file('/downloads/Series/Show/Season 01/[Rel 05]/Show 05.mkv'),
    file('/downloads/Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass'),
    file('/downloads/Series/Show/Season 01/[Rel 06]/Show 06.mkv'),
    file('/downloads/Series/Show/Season 01/[Rel 06]/notes.nfo'),
  ],
}

export const EXAMPLES: Example[] = [FULL, SINGLE, UNSURE, SUBTITLES, RELEASE, SUBS, NEXT, LIFTED, CLASH, TOO_LARGE, MOVIE, MUSIC, PHOTO, CLEANUP]

/** The example the page opens with. */
export const FIRST = SINGLE
