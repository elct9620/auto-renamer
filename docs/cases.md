# 測試案例

每個案例是管線的輸入與預期輸出，純階段不需要檔案系統就能驗證。預期輸出由 `design.md` 推得，實作後以測試確認。

```
  source /Downloads    target /Video    paths below are relative to source
```

## 1 影集影片

### 1.1 單檔集數

管線是 `design.md` 5.13 的 `pipeline.video`，`案例` 欄是 `vars.show`，target 為空。

| 案例 | 輸入 | 輸出 |
|---|---|---|
| Alpha | `Series/Alpha/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv` | `Series/Alpha/Alpha s01e12.mkv` |
| Beta Show | `Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4` | `Series/Beta Show/Beta Show s01e12.mp4` |
| Gamma Show | `Series/Gamma Show/[Team] Gamma Show - 18v2 (1080p) [5E9D2F64](2026).mkv` | `Series/Gamma Show/Gamma Show s01e18.mkv` |
| Delta-Show Kai | `Series/Delta-Show Kai/[Team] Delta-Show! Kaï 13 - Episode Title - 480p.x264.mkv` | `Series/Delta-Show Kai/Delta-Show Kai s01e13.mkv` |
| Epsilon | `Series/Epsilon/[Team]示範作品 Epsilon [1354][2026.09.26][1080P][ZH_JP][MP4].mp4` | `Series/Epsilon/Epsilon s01e1354.mp4` |
| Zeta-Show | `Series/Zeta-Show/Season 03/[Team³] 示範作品 第3季 Zeta-Show! S03 ｜ 10 [繁中] 1080p h.265 OPUS 2.0.mkv` | `Series/Zeta-Show/Season 03/Zeta-Show s03e10.mkv` |

### 1.2 分不出集數

| 案例 | 輸入 | 輸出 |
|---|---|---|
| Eta Show，target 空 | `Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4` | `Series/Eta Show/Season 17/Eta Show s17e01.mp4` |
| 目錄設定 `nth = 2` | 同上 | `Series/Eta Show/Season 17/Eta Show s17e03.mp4` |

候選是前綴標籤裡的 7 與集數 03，17 因等於季數而排除。兩個候選視為不確定，交給 `next`；指定 `nth` 才取第二個。

## 2 字幕與整批

### 2.1 兩集同批

檔案都在 `Series/Show/Season 01/`，`show = Show`，輸出在同一資料夾。

| 輸入 | 輸出 |
|---|---|
| `Show 27.mkv` | `Show s01e27.mkv` |
| `Show 27.cht.ass` | `Show s01e27.zh.01.ass` |
| `Show 27.chs.ass` | `Show s01e27.zh.02.ass` |
| `Show 28.mkv` | `Show s01e28.mkv` |
| `Show 28.cht.ass` | `Show s01e28.zh.ass` |

一個集合、兩個群組：字幕依集數分組，`cht` 排在前面，只有一個字幕的群組不編號。

### 2.2 整包資料夾

整包資料夾是 `Series/Theta_Show/Season 01/[Team][Theta_Show][27][1080p]/`，輸出提到 `Season 01/`。

| 輸入 | 輸出 |
|---|---|
| `[Team][Theta_Show][27][1080p].mkv` | `Theta_Show s01e27.mkv` |
| `[Team][Theta_Show][27][1080p].cht.ass` | `Theta_Show s01e27.zh.01.ass` |
| `[Team][Theta_Show][27][1080p].ass` | `Theta_Show s01e27.zh.02.ass` |

處理完，清空的整包資料夾被刪除。

### 2.3 Subs 子資料夾

單元設為 `{ root = ["Series/*/*"] }`，影片與字幕同屬 `Season 01` 這個單元。

| 輸入 | 輸出 |
|---|---|
| `Series/Show/Season 01/[Rel 05]/Show 05.mkv` | `Series/Show/Season 01/Show s01e05.mkv` |
| `Series/Show/Season 01/[Rel 05]/Subs/Show 05.cht.ass` | `Series/Show/Season 01/Show s01e05.zh.ass` |

`lift` 一次提出 `[Rel 05]/Subs/` 兩層，單一字幕只加 `.zh`。

## 3 遞增與撞名

### 3.1 遞增序號

target 已有 `Show s01e01.mkv`、`Show s01e02.mkv`，檔名沒有集數的檔案依名稱順序補號。

| 輸入 | 輸出 |
|---|---|
| `Show new a.mkv` | `Show s01e03.mkv` |
| `Show new b.mkv` | `Show s01e04.mkv` |
| `Show 07.mkv` | `Show s01e07.mkv` |

檔名有集數的檔案，`next` 不動作。

### 3.2 提出後遞增

target 的 `Series/Show/Season 01/` 已有 `Show s01e05.mkv`，`Season 02/` 有 `Show s02e09.mkv`。

| 輸入 | 輸出 |
|---|---|
| `Series/Show/Season 01/[Rel]/new.mkv` | `Series/Show/Season 01/Show s01e06.mkv` |

`lift` 先提出整包資料夾，`next` 才掃描 `Season 01/`，第 2 季的檔案不計入。

### 3.3 撞名

target 已有 `Show s01e12.mkv`，輸入 `Show 12.mkv`。

| 設定 | 結果 |
|---|---|
| 預設 | 拒絕，檔案留在 source |
| `on_conflict = "suffix"` | `Show s01e12_v2.mkv` |
| `suffix`，`_v2` 也已存在 | 拒絕 |

任何情況都不覆寫既有檔案。

### 3.4 拒絕與不動

這些情況都遵循「不確定就不動」，檔案留在 source 並記錄原因。

| 情況 | 結果 |
|---|---|
| 整包資料夾內的 `notes.nfo` | 沒有管線認領，留在 source |
| `vars` 沒有 `show` | `format` 缺欄位，拒絕 |
| `batch_max = 5`，一批有 6 個檔案 | 整批留在 source |
| 沒設 `batch_max`，一批有 1001 個檔案 | 整批留在 source |
| 路徑不是有效 UTF-8 | 不進管線，拒絕 |
| 資料夾內的 `auto-renamer.toml` | 不處理、不計入 `batch_max` |
| `dry_run = true` | 只記錄計畫路徑，不搬移、不清理 |

## 4 其他領域

### 4.1 電影

管線是 `pipeline.movie`，`strip` 預設只去掉 `[]`，括號 `()` 的年份保留。

| 輸入 | 輸出 |
|---|---|
| `Movies/XXX/[Group] XXX [1080p].mkv` | `Movies/XXX/XXX.mkv` |
| `Movies/Movie (2026)/[Group] Movie (2026).mkv` | `Movies/Movie (2026)/Movie (2026).mkv` |

### 4.2 音樂

管線是 `pipeline.music`，`regex` 的具名群組成為 `track` 與 `title` 欄位。

| 輸入 | 輸出 |
|---|---|
| `Music/Artist/Album/03 - Title.mp3` | `Music/Artist/Album/03 Title.mp3` |
| `Music/Artist/Album/cover.jpg` | 不認領，留在 source |

### 4.3 照片

修改時間以 UTC 計算，`folder` 在計畫路徑下加入年月資料夾。

| 修改時間 | 輸入 | 輸出 |
|---|---|---|
| 2026-09-27 | `Photos/IMG_0001.jpg` | `Photos/2026/09/IMG_0001.jpg` |
| 2026-10-01 | `Photos/IMG_0002.jpg` | `Photos/2026/10/IMG_0002.jpg` |

## 5 單元與清理

### 5.1 單元

| 設定 | 檔案 | 單元 |
|---|---|---|
| `{ root = ["Movies/*"] }` | `Movies/A/x.mkv` | `Movies/A` |
| 同上 | `Movies/A/Subs/x.ass` | `Movies/A` |
| 同上 | `Movies/B/y.mkv` | `Movies/B` |
| `"directory"` | `Movies/A/Subs/x.ass` | `Movies/A/Subs` |

不符合任何樣式的檔案，退回父資料夾。

### 5.2 清理

單元為 `{ root = ["Series/*/*"] }`，`cleanup` 帶 `keep = ["Season *"]`。

| 情況 | 結果 |
|---|---|
| 整包資料夾清空 | 刪除整包資料夾，`Season 01` 保留 |
| `Subs/` 與整包資料夾都清空 | 由內而外都刪除 |
| 整包資料夾內剩 `notes.nfo` | 不刪除 |
| 資料夾內有 `auto-renamer.toml` | 不算清空，保留 |
| 單元之上的 `Series/` 清空 | 保留，清理不超出單元 |
| `unit = "source"`，source 根清空 | 保留，source 根永不刪除 |

## 6 完整組合

Playground 的完整組合把影集情境疊在一起。source 為 `/Downloads`、target 為 `/Video`，管線是 5.13 的兩條，`move` 撞名時加 `_ai`，`batch_max = 5`；每個作品或季的資料夾以目錄設定指定 `show`。

| 情境 | 輸入 | 輸出 |
|---|---|---|
| 季資料夾與雜訊 | `Series/Alpha/Season 01/[Team] Alpha - 12 [1080p HEVC-10bit AAC].mkv` | `Series/Alpha/Season 01/Alpha s01e12.mkv` |
| 每季的作品名 | `Series/Alpha/Season 02/[Team] Alpha Next - 03 [1080p].mkv` | `Series/Alpha/Season 02/Alpha Next s02e03.mkv` |
| 撞名 | `Series/Beta Show/[Studio] Beta Show [12][WEB][HEVC-10bit 1080p AAC][EN].mp4` | `Series/Beta Show/Beta Show s01e12_ai.mp4` |
| 單一字幕 | 同上，副檔名 `.ass` | `Series/Beta Show/Beta Show s01e12.zh.ass` |
| 分不出集數 | `Series/Eta Show/Season 17/[Team-7][Eta Show 17][03][x264 1080p][TC].mp4` | `Series/Eta Show/Season 17/Eta Show s17e01.mp4` |
| 整包資料夾 | 2.2 的三個檔案 | 2.2 的輸出，整包資料夾刪除 |
| 太大的批次 | `Series/Theta_Show/Season 01/[Team][Theta_Show][Batch]/` 的 6 個檔案 | 整批留在 source |
| 一律遞增 | `Series/Kappa/Season 01/[Group] Kappa - 07 [1080p].mkv` | `Series/Kappa/Season 01/Kappa s01e03.mkv` |

target 原有 `Beta Show s01e12.mp4` 與 `Kappa s01e01.mkv`、`Kappa s01e02.mkv`。Kappa 的目錄設定把全域的 `video` 換成不擷取集數的一條，檔名中的 07 因此不採用。

## 7 驗證方式

| 範圍 | 方式 |
|---|---|
| 第 1 到 4 章 | 純函式單元測試 |
| 單元與批次收束 | 以模擬時鐘的整合測試 |
| 事件、寫入穩定 | 真實 inotify 的整合測試 |
| 跨檔案系統搬移 | 兩個掛載的整合測試 |

第 1 到 4 章的案例不需要檔案系統，可以最先實作與驗證。
