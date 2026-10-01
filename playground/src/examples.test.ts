import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { simulate } from './core'
import { EXAMPLES } from './examples'

// Where `docs/cases.md` says each file of an example goes, by its path under the source; `null` stays put.
const EXPECTED: Record<string, Record<string, string | null>> = {
  full: {
    'Series/Alpha/Season 01/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv': '/video/Series/Alpha/Season 01/Alpha s01e12.mkv',
    'Series/Alpha/Season 02/[Team] Alpha Next - 03 [1080p].mkv': '/video/Series/Alpha/Season 02/Alpha Next s02e03.mkv',
    'Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4': '/video/Series/Beta Show/Beta Show s01e12_ai.mp4',
    'Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].ass': '/video/Series/Beta Show/Beta Show s01e12.zh.ass',
    'Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/video/Series/Eta Show/Season 17/Eta Show s17e01.mp4',
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv': '/video/Series/Theta_Show/Season 01/Theta_Show s01e27.mkv',
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass': '/video/Series/Theta_Show/Season 01/Theta_Show s01e27.zh.01.ass',
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass': '/video/Series/Theta_Show/Season 01/Theta_Show s01e27.zh.02.ass',
    'Series/Kappa/Season 01/[Group] Kappa - 07 [1080p].mkv': '/video/Series/Kappa/Season 01/Kappa s01e03.mkv',
  },
  single: {
    'Series/Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv': '/downloads/Series/Alpha/Alpha s01e12.mkv',
    'Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4': '/downloads/Series/Beta Show/Beta Show s01e12.mp4',
    'Series/Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv': '/downloads/Series/Gamma Show/Gamma Show s01e18.mkv',
    'Series/Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv': '/downloads/Series/Delta-Show Kai/Delta-Show Kai s01e13.mkv',
    'Series/Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4': '/downloads/Series/Epsilon/Epsilon s01e1354.mp4',
    'Series/Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv': '/downloads/Series/Zeta-Show/Season 03/Zeta-Show s03e10.mkv',
  },
  unsure: {
    'Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/downloads/Series/Eta Show/Season 17/Eta Show s17e01.mp4',
    'Series/Eta Nth/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4': '/downloads/Series/Eta Nth/Season 17/Eta Show s17e03.mp4',
  },
  subtitles: {
    'Series/Show/Season 01/Show 27.mkv': '/downloads/Series/Show/Season 01/Show s01e27.mkv',
    'Series/Show/Season 01/Show 27.cht.ass': '/downloads/Series/Show/Season 01/Show s01e27.zh.01.ass',
    'Series/Show/Season 01/Show 27.chs.ass': '/downloads/Series/Show/Season 01/Show s01e27.zh.02.ass',
    'Series/Show/Season 01/Show 28.mkv': '/downloads/Series/Show/Season 01/Show s01e28.mkv',
    'Series/Show/Season 01/Show 28.cht.ass': '/downloads/Series/Show/Season 01/Show s01e28.zh.ass',
  },
  release: {
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].mkv': '/downloads/Series/Theta_Show/Season 01/Theta_Show s01e27.mkv',
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].cht.ass': '/downloads/Series/Theta_Show/Season 01/Theta_Show s01e27.zh.01.ass',
    'Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/[Team][Theta_Show][27][1080p].ass': '/downloads/Series/Theta_Show/Season 01/Theta_Show s01e27.zh.02.ass',
  },
  subs: {
    'Series/Show/Season 01/[Rel 05]/Show 05.mkv': '/downloads/Series/Show/Season 01/Show s01e05.mkv',
    'Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass': '/downloads/Series/Show/Season 01/Show s01e05.zh.ass',
  },
  next: {
    'Series/Show/Season 01/Show new a.mkv': '/video/Series/Show/Season 01/Show s01e03.mkv',
    'Series/Show/Season 01/Show new b.mkv': '/video/Series/Show/Season 01/Show s01e04.mkv',
    'Series/Show/Season 01/Show 07.mkv': '/video/Series/Show/Season 01/Show s01e07.mkv',
  },
  lifted: {
    'Series/Show/Season 01/[Rel]/new.mkv': '/video/Series/Show/Season 01/Show s01e06.mkv',
  },
  clash: {
    'Series/Show/Season 01/Show 12.mkv': '/video/Series/Show/Season 01/Show s01e12_v2.mkv',
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
    'Series/Show/Season 01/[Rel 05]/Show 05.mkv': '/downloads/Series/Show/Season 01/Show s01e05.mkv',
    'Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass': '/downloads/Series/Show/Season 01/Show s01e05.zh.ass',
    'Series/Show/Season 01/[Rel 06]/Show 06.mkv': '/downloads/Series/Show/Season 01/Show s01e06.mkv',
    'Series/Show/Season 01/[Rel 06]/notes.nfo': null,
  },
}

// Folders the cases expect gone after the trigger, or kept.
const FOLDERS: Record<string, { gone: string[]; kept: string[] }> = {
  full: { gone: ['/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]'], kept: [] },
  release: { gone: ['/downloads/Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]'], kept: ['/downloads/Series/Theta_Show/Season 01'] },
  cleanup: {
    gone: ['/downloads/Series/Show/Season 01/[Rel 05]/Subs', '/downloads/Series/Show/Season 01/[Rel 05]'],
    kept: ['/downloads/Series/Show/Season 01/[Rel 06]', '/downloads/Series/Show/Season 01'],
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
