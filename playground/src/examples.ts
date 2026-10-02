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

const EPISODE = '{ number = { into = "episode", exclude = ["season"] } },'

// The built-in `series-video` of `design.md` 5.13 as a folder configuration writes it to replace it, with its
// episode stage swapped or left out.
function seriesVideo(episode = EPISODE): string {
  return `[pipeline.series-video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { regex = { from = "path", pattern = '^(?<show>[^/]+)' } },
  { number = { from = "path", into = "season", prefix = "Season" } },${episode ? `\n  ${episode}` : ''}
  { default = { season = 1 } },
  { lift = { keep = 1 } },
  { folder = "Season {season:02}" },
  { next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } },
  { format = "{show} s{season:02}e{episode:02}" },
]
`
}

const KEEP_SEASONS = 'cleanup = { keep = ["Season *"] }'

// A watch over `/downloads` whose routes run the built-in series pipelines, each written into its route; the
// subtitles have no route when `subtitle` is false.
function series({ video = '', subtitle = '' as string | false, unit = '"directory"', extra = '' } = {}): string {
  const subtitles = subtitle === false ? '' : `, { pipeline = "series-subtitle"${subtitle} }`
  return `[watch.series]
source = "/downloads"
routes = [{ pipeline = "series-video"${video} }${subtitles}]
unit = ${unit}
${extra}`
}

const VIDEO_TARGET = '[target.video]\npath = "/video"\n\n'

// Every series scenario at once, with the source and the target apart: shows named by their folders,
// seasons by theirs, episodes found among noise or counted on, a release folder with its subtitles, a
// clash sent with its group to a second target, and folder configurations only where a folder is an
// exception: a season that bears another name, and a folder that always counts on.
const FULL: Example = {
  id: 'full',
  watch: 'series',
  config: `${VIDEO_TARGET}[target.conflict]
path = "/conflict"

${series({
    video: `, move = "video", ${KEEP_SEASONS}, rejected = { move = "conflict" }`,
    subtitle: ', move = "video", rejected = { move = "conflict" }',
    extra: 'group = ["show", "season", "episode"]\n',
  })}`,
  entries: [
    file('/downloads/Alpha/Season 01/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv'),
    file('/downloads/Alpha/Season 02/auto-renamer.toml', 'vars = { show = "Alpha Next" }\n'),
    file('/downloads/Alpha/Season 02/[Team] Alpha Next - 03 [1080p].mkv'),
    file('/downloads/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4'),
    file('/downloads/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].ass'),
    file('/video/Beta Show/Season 01/Beta Show s01e12.mp4'),
    file('/downloads/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
    file('/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv'),
    file('/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass'),
    file('/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass'),
    file('/downloads/Kappa/Season 01/auto-renamer.toml', seriesVideo('')),
    file('/downloads/Kappa/Season 01/[Group] Kappa - 07 [1080p].mkv'),
    file('/video/Kappa/Season 01/Kappa s01e01.mkv'),
    file('/video/Kappa/Season 01/Kappa s01e02.mkv'),
  ],
}

// The page opens here, so the built-in `series-video` is written out for the canvas to draw its stages.
const SINGLE: Example = {
  id: 'single',
  watch: 'series',
  config: `${seriesVideo()}
${series({ video: `, ${KEEP_SEASONS}`, subtitle: false, unit: '{ root = ["*"] }' })}`,
  entries: [
    'Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv',
    'Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4',
    'Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv',
    'Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv',
    'Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4',
    'Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv',
  ].map((path) => file(`/downloads/${path}`)),
}

const UNSURE: Example = {
  id: 'unsure',
  watch: 'series',
  config: SINGLE.config,
  entries: [
    file('/downloads/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
    file('/downloads/Eta Nth/auto-renamer.toml',
      `vars = { show = "Eta Show" }\n\n${seriesVideo('{ number = { into = "episode", exclude = ["season"], nth = 2 } },')}`),
    file('/downloads/Eta Nth/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4'),
  ],
}

const SUBTITLES: Example = {
  id: 'subtitles',
  watch: 'series',
  config: series(),
  entries: ['Show 27.mkv', 'Show 27.cht.ass', 'Show 27.chs.ass', 'Show 28.mkv', 'Show 28.cht.ass']
    .map((name) => file(`/downloads/Show/Season 01/${name}`)),
}

const RELEASE: Example = {
  id: 'release',
  watch: 'series',
  config: series({ video: `, ${KEEP_SEASONS}`, unit: '{ root = ["*/*"] }' }),
  entries: ['.mkv', '.cht.ass', '.ass']
    .map((ext) => file(`/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p]${ext}`)),
}

const SUBS: Example = {
  id: 'subs',
  watch: 'series',
  config: series({ unit: '{ root = ["*/*"] }' }),
  entries: [
    file('/downloads/Show/Season 01/[Rel 05]/Show 05.mkv'),
    file('/downloads/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass'),
  ],
}

const NEXT: Example = {
  id: 'next',
  watch: 'series',
  config: `${VIDEO_TARGET}${series({ video: ', move = "video"', subtitle: false })}`,
  entries: [
    file('/video/Show/Season 01/Show s01e01.mkv'),
    file('/video/Show/Season 01/Show s01e02.mkv'),
    ...['Show new a.mkv', 'Show new b.mkv', 'Show 07.mkv'].map((name) => file(`/downloads/Show/${name}`)),
  ],
}

const LIFTED: Example = {
  id: 'lifted',
  watch: 'series',
  config: `${VIDEO_TARGET}${series({ video: ', move = "video"', subtitle: false, unit: '{ root = ["*/*"] }' })}`,
  entries: [
    file('/video/Show/Season 01/Show s01e05.mkv'),
    file('/video/Show/Season 02/Show s02e09.mkv'),
    file('/downloads/Show/Season 01/[Rel]/new.mkv'),
  ],
}

const CLASH: Example = {
  id: 'clash',
  watch: 'series',
  config: `${VIDEO_TARGET}[target.conflict]
path = "/conflict"

${series({ video: ', move = "video", rejected = { move = "conflict" }', subtitle: false })}`,
  entries: [
    file('/video/Show/Season 01/Show s01e12.mkv'),
    file('/downloads/Show/Show 12.mkv'),
  ],
}

// The other pipelines of `design.md` 5.14, each alone with the files of its case, renamed in place.
function other(name: string, stages: string, entries: Entry[]): Example {
  return {
    id: name,
    watch: name,
    config: `[pipeline.${name}]\nstages = ${stages}\n\n[watch.${name}]\nsource = "/downloads"\nroutes = [{ pipeline = "${name}" }]\n`,
    entries,
  }
}

const MOVIE = other('movie', '[{ filter = { ext = ["mkv", "mp4"] } }, { strip = {} }]', [
  file('/downloads/Movies/XXX/[Group] XXX [1080p].mkv'),
  file('/downloads/Movies/Movie (2026)/[Group] Movie (2026).mkv'),
])

const MUSIC = other('music', `[
  { filter = { ext = ["mp3", "flac"] } },
  { regex = { pattern = '^(?<track>\\d+)\\s*-\\s*(?<title>.+)$' } },
  { format = "{track:02} {title}" },
]`, [
  file('/downloads/Music/Artist/Album/03 - Title.mp3'),
  file('/downloads/Music/Artist/Album/cover.jpg'),
])

const PHOTO = other('photo', `[
  { filter = { ext = ["jpg", "png"] } },
  { folder = "{mtime:%Y}/{mtime:%m}" },
]`, [
  file('/downloads/Photos/IMG_0001.jpg', '', Date.UTC(2026, 8, 27)),
  file('/downloads/Photos/IMG_0002.jpg', '', Date.UTC(2026, 9, 1)),
])

// Both routes clean up, so a `Subs` folder the subtitles leave empty is removed too.
const CLEANUP: Example = {
  id: 'cleanup',
  watch: 'series',
  config: series({ video: `, ${KEEP_SEASONS}`, subtitle: `, ${KEEP_SEASONS}`, unit: '{ root = ["*/*"] }' }),
  entries: [
    file('/downloads/Show/Season 01/[Rel 05]/Show 05.mkv'),
    file('/downloads/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass'),
    file('/downloads/Show/Season 01/[Rel 06]/Show 06.mkv'),
    file('/downloads/Show/Season 01/[Rel 06]/notes.nfo'),
  ],
}

export const EXAMPLES: Example[] = [FULL, SINGLE, UNSURE, SUBTITLES, RELEASE, SUBS, NEXT, LIFTED, CLASH, MOVIE, MUSIC, PHOTO, CLEANUP]

/** The example the page opens with. */
export const FIRST = SINGLE
