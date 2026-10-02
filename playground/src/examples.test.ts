import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { simulate } from './core'
import { EXAMPLES } from './examples'

// Where `docs/cases.md` says each file of an example goes, by its path under the source; `null` stays put.
const EXPECTED: Record<string, Record<string, string | null>> = {
  full: {
    'Alpha/Season 01/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv': '/video/Alpha/Season 01/Alpha s01e12.mkv',
    'Alpha/Season 02/[Team] Alpha Next - 03 [1080p].mkv': '/video/Alpha/Season 02/Alpha Next s02e03.mkv',
    'Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4': '/conflict/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4',
    'Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].ass': '/conflict/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].ass',
    'Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/video/Eta Show/Season 17/Eta Show s17e01.mp4',
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv': '/video/Theta_Show/Season 01/Theta_Show s01e27.mkv',
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass': '/video/Theta_Show/Season 01/Theta_Show s01e27.zh.01.ass',
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass': '/video/Theta_Show/Season 01/Theta_Show s01e27.zh.02.ass',
    'Kappa/Season 01/[Group] Kappa - 07 [1080p].mkv': '/video/Kappa/Season 01/Kappa s01e03.mkv',
  },
  single: {
    'Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv': '/downloads/Alpha/Season 01/Alpha s01e12.mkv',
    'Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4': '/downloads/Beta Show/Season 01/Beta Show s01e12.mp4',
    'Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv': '/downloads/Gamma Show/Season 01/Gamma Show s01e18.mkv',
    'Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv': '/downloads/Delta-Show Kai/Season 01/Delta-Show Kai s01e13.mkv',
    'Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4': '/downloads/Epsilon/Season 01/Epsilon s01e1354.mp4',
    'Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv': '/downloads/Zeta-Show/Season 03/Zeta-Show s03e10.mkv',
  },
  unsure: {
    'Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/downloads/Eta Show/Season 17/Eta Show s17e01.mp4',
    'Eta Nth/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/downloads/Eta Nth/Season 17/Eta Show s17e03.mp4',
  },
  subtitles: {
    'Show/Season 01/Show 27.mkv': '/downloads/Show/Season 01/Show s01e27.mkv',
    'Show/Season 01/Show 27.cht.ass': '/downloads/Show/Season 01/Show s01e27.zh.01.ass',
    'Show/Season 01/Show 27.chs.ass': '/downloads/Show/Season 01/Show s01e27.zh.02.ass',
    'Show/Season 01/Show 28.mkv': '/downloads/Show/Season 01/Show s01e28.mkv',
    'Show/Season 01/Show 28.cht.ass': '/downloads/Show/Season 01/Show s01e28.zh.ass',
  },
  release: {
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv': '/downloads/Theta_Show/Season 01/Theta_Show s01e27.mkv',
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass': '/downloads/Theta_Show/Season 01/Theta_Show s01e27.zh.01.ass',
    'Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass': '/downloads/Theta_Show/Season 01/Theta_Show s01e27.zh.02.ass',
  },
  subs: {
    'Show/Season 01/[Rel 05]/Show 05.mkv': '/downloads/Show/Season 01/Show s01e05.mkv',
    'Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass': '/downloads/Show/Season 01/Show s01e05.zh.ass',
  },
  next: {
    'Show/Show new a.mkv': '/video/Show/Season 01/Show s01e03.mkv',
    'Show/Show new b.mkv': '/video/Show/Season 01/Show s01e04.mkv',
    'Show/Show 07.mkv': '/video/Show/Season 01/Show s01e07.mkv',
  },
  lifted: {
    'Show/Season 01/[Rel]/new.mkv': '/video/Show/Season 01/Show s01e06.mkv',
  },
  clash: {
    'Show/Show 12.mkv': '/conflict/Show/Show 12.mkv',
  },
  movie: {
    'Movies/XXX/[Group] XXX [1080p].mkv': '/downloads/Movies/XXX/XXX.mkv',
    'Movies/Movie (2026)/[Group] Movie (2026).mkv': '/downloads/Movies/Movie (2026)/Movie (2026).mkv',
  },
  music: {
    'Music/Artist/Album/03 - Title.mp3': '/downloads/Music/Artist/Album/03 Title.mp3',
    'Music/Artist/Album/cover.jpg': null,
  },
  photo: {
    'Photos/IMG_0001.jpg': '/downloads/Photos/2026/09/IMG_0001.jpg',
    'Photos/IMG_0002.jpg': '/downloads/Photos/2026/10/IMG_0002.jpg',
  },
  cleanup: {
    'Show/Season 01/[Rel 05]/Show 05.mkv': '/downloads/Show/Season 01/Show s01e05.mkv',
    'Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass': '/downloads/Show/Season 01/Show s01e05.zh.ass',
    'Show/Season 01/[Rel 06]/Show 06.mkv': '/downloads/Show/Season 01/Show s01e06.mkv',
    'Show/Season 01/[Rel 06]/notes.nfo': null,
  },
}

// Folders the cases expect gone after the trigger, or kept.
const FOLDERS: Record<string, { gone: string[]; kept: string[] }> = {
  full: { gone: ['/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]'], kept: [] },
  release: { gone: ['/downloads/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]'], kept: ['/downloads/Theta_Show/Season 01'] },
  cleanup: {
    gone: ['/downloads/Show/Season 01/[Rel 05]/Subs', '/downloads/Show/Season 01/[Rel 05]'],
    kept: ['/downloads/Show/Season 01/[Rel 06]', '/downloads/Show/Season 01'],
  },
}

describe('examples', () => {
  // @behavior PGE-057
  it.each(EXAMPLES.map((example) => [example.id, example] as const))('moves the files of %s where the cases say', (id, example) => {
    const { outcomes, entries } = simulate(example.config, example.watch, example.entries)

    expect(Object.fromEntries(outcomes.map((outcome) => [outcome.origin, outcome.to]))).toEqual(EXPECTED[id])
    const paths = entries.map((entry) => entry.path)
    for (const folder of FOLDERS[id]?.gone ?? []) expect(paths).not.toContain(folder)
    for (const folder of FOLDERS[id]?.kept ?? []) expect(paths).toContain(folder)
  })
})
