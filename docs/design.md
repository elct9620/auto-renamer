# auto-renamer 設計

auto-renamer 監看資料夾的檔案事件，依管線改寫檔案的名稱與位置，再把檔案移到目標目錄。

```
  source ─► watch ─► pipeline ─► target
```

## 0 總覽

### 0.1 設計承諾

| 承諾 | 意義 |
|---|---|
| 事件驅動 | 由 Linux 檔案系統事件觸發處理 |
| 小階段堆疊 | 每個階段只做一件事，能力來自組合 |
| 前純後動 | 效果階段之前都是純函式，可乾跑 |
| 不確定就不動 | 無法判斷的檔案留在 source 並記錄原因 |
| 關聯限於單元 | 不同單元的檔案不分組、不共號 |
| 只信全域設定 | 目錄設定不能改 source、target 與單元 |
| 只保證 Linux | 依賴 inotify，其他平台不在範圍 |

這七項是設計的固定點。每一章都在套用其中幾項；兩項衝突時，由該章說明哪一項讓步。

### 0.2 系統分層

```
  Linux inotify
      │ events
  ┌───▼────────┐
  │ watcher    │  notify crate; settle, unit, batch
  └───┬────────┘
      │ batch
  ┌───▼────────┐
  │ pipeline   │  pure stages → effect stages
  └───┬────────┘
      │ plan
  ┌───▼────────┐
  │ filesystem │  source → target
  └────────────┘
```

每層只與相鄰的層溝通。設定在啟動與設定檔變動時讀取，交給管線使用。

### 0.3 處理流程

```
  /Downloads/Movies/XXX/XXX.mp4         download finishes
    └ watcher    close-write, file settles
    └ pipeline   XXX.mp4 → XXX-s1e1.mp4
    └ move       copy to temp name, then rename
  /Video/Movies/XXX/XXX-s1e1.mp4        ready for the media server
```

source 與 target 分離，媒體伺服器只掃描 target，它產生的 `.nfo` 等檔案不會再被改名。

### 0.4 單元、批次與群組

| 層 | 決定什麼 | 依據 |
|---|---|---|
| 單元 | 哪些檔案彼此有關 | watch 的 `unit` |
| 批次 | 何時處理 | 單元的一次收集 |
| 群組 | 批次內誰是一組 | 階段算出的欄位 |

批次是單元在視窗內收集到的檔案。群組由欄位相同的檔案組成，與批次和資料夾無關，例如同一集的影片與字幕。

### 0.5 進度

| 章 | 依賴 | 進度 |
|---|---|---|
| 1 監看 | 0 | ✅ |
| 2 管線 | 1 | ✅ |
| 3 設定 | 2 | ✅ |
| 4 執行行為 | 2、3 | ✅ |
| 5 內建階段 | 2 | ✅ |

✅ 該章每一句都有 `.spec/behavior` 的行為與測試對應。章節依實作的依賴排列。

### 0.6 建置與發行

| 產物 | 說明 |
|---|---|
| Linux binary | 靜態 musl |
| 容器映像 | `scratch`，push main 更新 `latest` |

映像沒有 shell、CA 憑證與時區資料，所以只做本機檔案操作。inotify 是核心功能，不受影響。版本由 release-please 管理，release 時映像加上版本標籤。

## 1 監看

### 1.1 監看範圍

| 事件 | notify 對應 | 用途 |
|---|---|---|
| 寫入完成 | `Access(Close(Write))` | 檔案可以處理 |
| 搬入 | `Modify(Name(To))` | 同檔案系統的 `mv` 沒有寫入完成事件 |
| 資料夾出現 | `Create(Folder)`、`Name(To)` | 掃描內容，補上未逐一通報的檔案 |

每個 watch 遞迴監看自己的 source，事件由 notify crate 的 inotify 後端提供，不跟隨符號連結。

### 1.2 寫入穩定

```
  writing    create, modify ...      ignored
  finished   close-write | moved-to  settled
```

只有穩定的檔案才進入處理。下載器、複製與 `mv` 三種來源，都以同一組事件判定。寫到一半的檔案不會產生寫入完成事件，它讓所屬單元保持開啟，直到它穩定、消失，或停止超過最大等待。

### 1.3 單元

| 寫法 | 單元 |
|---|---|
| `"directory"`（預設） | 檔案的父資料夾 |
| `"source"` | 整個 source |
| `{ root = [...] }` | 符合樣式的資料夾，底下的檔案都算 |

樣式是相對於 source 的路徑，不符合任何樣式的檔案退回父資料夾。單元由使用者定義，不假設下載目錄的結構。單元只能由全域設定指定，因為目錄設定要在單元決定之後才找得到。

### 1.4 批次收束

```
  a.mkv settled ┐
  b.mkv settled ├─ same unit ─► batch ─► pipeline
  c.mkv settled ┘  closes on a quiet period or a maximum wait
```

| 設定 | 作用 | 預設 |
|---|---|---|
| `batch_window` | 安靜這麼久就收束 | 5 分鐘 |
| `batch_max_wait` | 從第一個檔案起最多等這麼久 | 30 分鐘 |
| `batch_max` | 超過這個檔案數，整批略過 | 1000 |

收束時間依單元各自計算，`batch_max_wait` 不可小於 `batch_window`。批次不依數量切開，以免拆散同一群組的檔案。`batch_max` 不得超過 10 萬，單元收集到這個數量就不再記錄更多檔案，以免記憶體無限成長。略過的批次留在 source 並記錄原因。

### 1.5 保留檔案

| 檔案 | 處理 |
|---|---|
| `auto-renamer.toml` | 目錄設定；不進管線、不搬移、不刪除 |

`auto-renamer.toml` 是設定而非素材，由使用者管理，不計入 `batch_max`。它變動時，重新載入該目錄的設定。

### 1.6 啟動掃描

```
  start ─► scan source ─► existing files ─► settled ─► batch
                                   └ mtime within batch_window: settled after a quiet window
```

啟動時掃描 source 內既有的檔案，視同穩定事件進入批次，停機期間到達的檔案不會被遺漏。修改時間在 `batch_window` 內的檔案，等修改時間過了一個視窗且期間沒有寫入事件才算穩定，因為它之後不會再有寫入完成事件。

## 2 管線

### 2.1 管線模型

```
  batch (sorted by name)
   │ partition ─ claimed ─► pipeline 1: map … scan … ───────┐
   │           └ rest ────► pipeline 2: map … join(1) … ─────┤
   ▼                                                          ▼
  unclaimed stay in source                    planned batch ─► foreach effects
```

一批檔案是依名稱排序的有序集合，每個階段是「集合 → 集合」的函式，逐檔階段是 map 的簡寫。單一檔案就是只有一個檔案的批次，適用同一套模型。

### 2.2 記錄

```
  record = { plan:   "Movies/XXX/XXX.mp4",
             fields: { name, ext, dir, path, mtime, show, season, episode } }
```

集合中的每個元素是一筆記錄，含計畫路徑與具名欄位。路徑起點是 source 內的相對路徑，即保留結構。計畫路徑的檔名永遠等於 `name` 加 `ext`，改寫這兩個欄位就是改名。欄位的起點見 2.7。

### 2.3 階段形狀

| 形狀 | 作用 | 例子 |
|---|---|---|
| filter | 放行或排除 | `filter` |
| map | 逐檔改寫 | `number`、`format` |
| scan | 帶累積狀態走訪 | `next` |
| group | 依欄位分組或配對 | `rank`、`take` |
| effect | 碰檔案系統，依序執行 | `move`、`cleanup` |

前四種是純函式，整條管線因此可以乾跑預覽。階段不改動副檔名；正規表達式只用於內建階段處理不了的情況。

### 2.4 階段輸出

| 輸出 | 意義 |
|---|---|
| 改寫 | 記錄改變，交給下一階段 |
| 不變 | 階段不適用，原樣交給下一階段 |
| 排除 | filter 不符，檔案不再往下走 |
| 拒絕 | 停止該檔案，留在 source 並記錄原因 |

拒絕逐檔短路，依賴被拒絕檔案的階段（如 `take`）連帶拒絕，其他檔案照常處理。階段的輸入是批次、target 現況與前面管線的結果。

### 2.5 認領與排除

| 情況 | 結果 |
|---|---|
| 開頭的 `filter` 接受 | 這條管線認領該檔案 |
| 開頭的 `filter` 不接受 | 交給下一條管線 |
| 中途的 `filter` 排除 | 留在 source，不轉給其他管線 |
| 沒有管線認領 | 留在 source |

一個 watch 可以有多條管線，檔案由第一條認領它的管線處理。管線內不寫條件分支，分支由多條管線加 `filter` 表達。

### 2.6 效果階段

| 階段 | 作用 |
|---|---|
| `move` | 把計畫路徑套用到 target |
| `cleanup` | 刪除單元內已清空的資料夾 |

效果階段必須明寫，並排在所有純階段之後，依列出的順序執行。整批先全部規劃完，才執行效果。沒有效果階段的管線只乾跑並警告。

### 2.7 起始欄位

| 欄位 | 內容 |
|---|---|
| `name` | 主檔名 |
| `ext` | 副檔名 |
| `dir` | 所在資料夾名 |
| `path` | 相對於 source 的資料夾路徑 |
| `mtime` | 修改時間，UTC |
| `vars` | 設定的 `vars` |

欄位值有文字、數字與日期三種。映像沒有時區資料，所以日期一律以 UTC 表示。路徑不是有效 UTF-8 時，沒有 `name`、`ext`、`dir`、`path`，讀名稱的階段拒絕該檔案；`folder`、`move`、`cleanup` 以原始位元組處理。

## 3 設定

### 3.1 全域設定

```toml
[default]
batch_window = "5m"
pipelines = ["video"]

[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  "move",
]

[watch.series]
source = "/Downloads"
target = "/Video"
unit = { root = ["Movies/*"] }
vars = { show = "Alpha" }
```

`pipeline.*` 定義具名管線；`watch` 綁定 source、target、`unit` 與 `vars`；`default` 供所有 watch 共用。省略 target 表示原地改名。

### 3.2 管線宣告

| 寫法 | 意義 |
|---|---|
| `"move"` | 沒有參數的階段 |
| `{ format = "..." }` | 一個參數 |
| `{ filter = { ext = [...] } }` | 具名參數 |

`stages` 是階段的陣列，由上而下依序處理，每個階段是單鍵內嵌表格或字串。

### 3.3 目錄設定

```toml
# /Downloads/Series/XXX/auto-renamer.toml
vars = { show = "Eta Show" }

[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { regex = { pattern = '\]\[(\d+)\]\[x264', into = "episode" } },
  { format = "{show} s01e{episode}" },
  "move",
]
```

資料夾內的 `auto-renamer.toml` 覆蓋 watch 的設定，處理例外。作用方式如 `.gitignore`：管該資料夾與其下層，最近的優先。單元所在的資料夾由 source 往下逐層尋找，由遠到近套用。

### 3.4 覆寫順序

```
  [default]  <  [watch.*]  <  auto-renamer.toml (parent)  <  auto-renamer.toml (nearest)
  low                                                          high
```

後面的層覆蓋前面的層。`vars` 逐鍵覆蓋，`pipeline.*` 依名稱整條取代，不逐階段合併。

### 3.5 覆寫界線

| 項目 | 目錄設定 |
|---|---|
| 管線、`vars`、`batch_max` | 可覆寫，`batch_max` 不得超過 10 萬 |
| `source`、`target`、`unit`、`dry_run` | 不可覆寫 |
| 跳出 target 根的路徑 | 任何層都拒絕 |

下載內容不可信，夾帶的 `auto-renamer.toml` 不能決定檔案的去向或分組。它不得超過 64 KiB，管線不得超過 64 個階段。

### 3.6 設定載入

| 項目 | 規則 |
|---|---|
| 全域設定檔 | `/etc/auto-renamer/config.toml` |
| 指定路徑 | `--config` 參數 |
| 重新載入 | 設定檔被寫入或換掉，或收到 SIGHUP |
| 無效的設定 | 保留舊設定並記錄錯誤 |
| 停止 | SIGTERM 或 SIGINT，處理完批次就結束 |

設定與目錄設定變動時重新載入，不需重啟。讀取設定檔也會產生事件，只有寫入與換掉才算變動；單獨掛載的檔案沒有事件，用 SIGHUP 觸發。停止時放棄等待中的批次，檔案仍在 source，下次啟動會重新掃描。

### 3.7 乾跑

```toml
[watch.series]
dry_run = true      # log each plan, run no effect stage
```

`dry_run = true` 時管線照常規劃，把每個檔案的原路徑、計畫路徑與拒絕原因寫進 log，但不執行效果階段。預設關閉，適合第一次寫管線時先看結果。

## 4 執行行為

### 4.1 路徑對應

```
  source root   /Downloads
  plan          Movies/XXX/XXX-s1e1.mp4
  target root   /Video
  result        /Video/Movies/XXX/XXX-s1e1.mp4
```

結果是 target 根加計畫路徑，缺少的目錄會建立。計畫路徑離開 target 根時拒絕。

### 4.2 跨檔案系統

```
  same filesystem    rename ─► done
  other filesystem   copy to temp name ─► rename ─► delete source
```

容器內 source 與 target 常是不同的掛載，`rename` 無法跨越。先寫成 `.` 開頭的暫存名，保留修改時間，只留讀寫權限，再改名，媒體伺服器就不會掃到寫到一半的檔案。暫存名每次不同，中斷時留下的暫存檔不會擋住下次搬移。

### 4.3 撞名

```
  a.mp4 exists ─► reject                            default
  a.mp4 exists ─► a_v2.mp4 ─ exists ─► reject       on_conflict = "suffix"
```

`move` 預設在目標已存在時拒絕，檔案留在 source。設 `on_conflict = "suffix"` 會加上 `suffix`（預設 `_v2`）重試一次，仍撞名就拒絕。任何情況都不覆寫既有檔案，符號連結與資料夾不處理也不跟隨。同一檔案系統以不覆寫的 rename 搬移，監看者看到的是搬入。

### 4.4 冪等

| 情況 | 結果 |
|---|---|
| target 位於 source 內 | 設定驗證拒絕，程式不啟動 |
| watch 的 source 彼此重疊 | 設定驗證拒絕 |
| target 與其他 watch 的 source 重疊 | 設定驗證拒絕 |
| 事件來自 target | 不監看，不處理 |
| 計畫路徑等於目前路徑 | 不動作 |
| 原地連續改名 5 次 | 第 6 次拒絕並記錄 |

改名與搬移本身會產生新事件，冪等確保處理過的檔案不會再被處理。管線若每次都改寫自己的結果就不冪等，深度限制擋下這種無限改名；被判定為不動的檔案歸零。

### 4.5 清理

```
  Season 01/[Rel]/Subs/   empty after move ─► removed
  Season 01/[Rel]/        empty             ─► removed
  Season 01/              matches keep      ─► kept, stop
  Series/                  above the unit    ─► never touched
```

`cleanup` 只刪除單元之內清空的資料夾（含單元本身），由檔案原本所在處往上，遇到非空、含 `auto-renamer.toml`、符合 `keep` 或是連結的資料夾就停。source 根永不刪除；乾跑只列出現在就是空的資料夾。

## 5 內建階段

### 5.1 階段清單

| 階段 | 形狀 | 作用 |
|---|---|---|
| `filter` | filter | 依副檔名或名稱樣式放行 |
| `number` | map | 依序號規則取出數字 |
| `regex` | map | 擷取或改寫，通用的逃生口 |
| `set` | map | 設定固定值，覆蓋既有 |
| `default` | map | 欄位未設時填入固定值 |
| `replace`、`case`、`strip` | map | 字面取代、大小寫、去括號標籤 |
| `format` | map | 依樣板組出檔名 |
| `lift`、`folder` | map | 計畫路徑往上提、往下放 |
| `next` | scan | 欄位未設時，取 target 最大值加一 |
| `rank` | group | 同組內排序並編號 |
| `take` | group | 依名稱前綴向同批影片取欄位 |
| `move`、`cleanup` | effect | 搬移與清理 |

每個階段只做一件事，複雜的行為由堆疊產生。常見的改寫優先由內建階段提供，`regex` 留給內建處理不了的情況。

### 5.2 序號擷取

| 參數 | 作用 | 預設 |
|---|---|---|
| `from` | 從哪個欄位找 | `name` |
| `into` | 寫入哪個欄位 | 必填 |
| `nth` | 明指取第幾個候選，負數從尾端算 | 無 |
| `prefix` | 只取緊接在這個字之後的數字 | 無 |
| `exclude` | 排除這些欄位目前的值 | 無 |

`number` 從欄位中取出一個數字。季數是它的變體：`from = "path"` 加 `prefix = "Season"`。

### 5.3 判斷順序

```
  strong marker   S01E02 | EP02 | - 02     ─► use it
  scan            drop noise ─► one candidate     ─► use it
                             ─► several candidates ─► unset, unless nth
```

強標記優先，`exclude` 只作用在掃描。掃描恰好一個候選才採用，多個視為不確定，欄位維持未設；明指 `nth` 時只做掃描並依它挑選。

### 5.4 雜訊清單

| 雜訊 | 例子 |
|---|---|
| 解析度 | `1080p` |
| 編碼、位元深度 | `x264`、`h.265`、`10bit` |
| 版本 | `18v2` |
| 日期、年份 | `2026.09.26` |
| 音訊規格 | `2.0` |
| 季數標記、序數 | `S03`、`第3季`、`2nd` |
| 雜湊、上標數字 | `5E9D2F64`、`7³` |

掃描前先排除這些數字，它們不會成為候選。

### 5.5 預設值

```toml
{ number = { from = "path", into = "season", prefix = "Season" } },
{ number = { into = "episode", exclude = ["season"] } },
{ default = { season = 1 } },
```

`default` 只在欄位未設時填入，須放在擷取之後：季數沒找到就套預設，`exclude` 才不會把預設值當成已知的季數，誤排除第 1 集。

### 5.6 遞增序號

```
  target      Alpha s01e01, Alpha s01e02   matched by like
  episode     unset ─► max 2 + 1 = 3
```

| `like` 的部分 | 規則 |
|---|---|
| 已有值的欄位 | 代入目前的值，當成字面文字 |
| `into` 欄位 | 唯一的擷取群組，`:02` 表示至少 2 位數 |
| 其他未設的欄位 | 拒絕該檔案 |
| 副檔名 | 不比對，只看主檔名 |
| 掃描的資料夾 | 計畫路徑目前所在的 target 資料夾 |

`next` 只在欄位未設時填入，取符合的既有檔案的最大值加一，沒有符合就是 1。同批的檔案依序遞增；路徑階段如 `lift` 必須排在它之前。

### 5.7 分組編號

```
  Show 27.mkv       ─ number ─► 27  ── video group
  Show 27.cht.ass   ─ number ─► 27  ┐
  Show 27.chs.ass   ─ number ─► 27  ┴─ rank: cht → 01, chs → 02
  Show 28.cht.ass   ─ number ─► 28  ── rank: alone → unset
```

`rank` 在管線認領的檔案中，依 `by` 欄位分組，原始檔名含 `prefer` 中較前字串（不分大小寫）的排前面，其餘依名稱排序。缺 `by` 欄位的檔案拒絕；只有一個檔案時不設 `into`。

### 5.8 篩選

| 參數 | 作用 |
|---|---|
| `ext` | 副檔名清單，不分大小寫，不含點 |
| `glob` | 檔名樣式 |
| `invert` | 反轉結果，用於排除 |

`ext` 與 `glob` 同時給時都要符合。開頭的 `filter` 認領檔案，中途的 `filter` 排除檔案（2.5）。

### 5.9 名稱改寫

| 階段 | 參數 | 作用 |
|---|---|---|
| `regex` | `pattern`、`from`、`into` | 擷取，具名群組成欄位 |
| `regex` | `pattern`、`from`、`replace` | 改寫欄位內容 |
| `set` | 欄位 = 值 | 設定固定值，覆蓋既有 |
| `replace` | `find`、`with`、`field` | 字面取代 |
| `case` | `to`、`field` | `lower`、`upper`、`title` |
| `strip` | `groups`、`field` | 去掉括號標籤 |

`from` 與 `field` 預設是 `name`。`regex` 用 Rust `regex` crate 的語法，沒有比對到時不變，只由數字組成的擷取值成為數字。`strip` 的 `groups` 預設只有 `["[]"]`，並整理多餘空白。

### 5.10 路徑改寫

| 階段 | 參數 | 作用 |
|---|---|---|
| `lift` | `to` 或整數 | 計畫路徑往上提 |
| `folder` | 樣板 | 依樣板加入子資料夾 |

`lift` 的 `to` 是資料夾名稱樣式，提到最近符合的祖先，沒有符合就不變；整數超過資料夾層數就拒絕。`folder` 的樣板以 `/` 分段，欄位缺值或任何一段不是可用的資料夾名就拒絕。

### 5.11 名稱配對

| 參數 | 作用 | 預設 |
|---|---|---|
| `fields` | 要複製的欄位 | 必填 |
| `from` | 來源管線名稱 | 前面所有管線 |

`take` 只在字幕自己偵測不到欄位時使用。影片的原始主檔名是字幕主檔名的前綴（含相等）才配對，取最長者；沒有、並列，或配對到的檔案沒被規劃就拒絕。

### 5.12 名稱樣板

| 語法 | 意義 |
|---|---|
| `{field}` | 取欄位值 |
| `{field:02}` | 數字補零到至少 2 位 |
| `{mtime:%Y-%m}` | 日期欄位依 strftime 格式輸出 |
| `[...]` | 可選片段，欄位都有值才輸出 |
| `{{`、`}}`、`[[`、`]]` | 字面的括號 |

必要位置的欄位沒有值就拒絕；日期欄位必須帶格式，補零至多 20 位。`format` 只寫檔名，副檔名自動接在最後。

### 5.13 影集管線

```toml
[pipeline.video]
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

[pipeline.subtitle]
stages = [
  { filter = { ext = ["ass", "srt"] } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  { number = { into = "episode", exclude = ["season"] } },
  { default = { season = 1 } },
  { rank = { into = "index", by = ["season", "episode"], prefer = ["cht"] } },
  { lift = { to = "Season *" } },
  { format = "{show} s{season:02}e{episode:02}.zh[.{index:02}]" },
  "move",
]
```

影集不是內建功能，而是階段的堆疊。字幕與影片各自跑 `number`，得到相同的季與集，不必互相查找。

### 5.14 其他管線

```toml
[pipeline.movie]        # tags out of the name
stages = [{ filter = { ext = ["mkv", "mp4"] } }, { strip = {} }, "move"]

[pipeline.music]        # "03 - Title.mp3" → "03 Title.mp3"
stages = [
  { filter = { ext = ["mp3", "flac"] } },
  { regex = { pattern = '^(?<track>\d+)\s*-\s*(?<title>.+)$' } },
  { format = "{track:02} {title}" },
  "move",
]

[pipeline.photo]        # sort into year/month by modified time
stages = [
  { filter = { ext = ["jpg", "png"] } },
  { folder = "{mtime:%Y}/{mtime:%m}" },
  "move",
]
```

同一套階段處理影集以外的檔案，不需要新增階段。

## 附錄

### 刻意排除

設計明確不做的事，避免日後被誤當成待辦。

| 項目 | 理由 |
|---|---|
| 非 Linux 平台 | 事件語意依賴 inotify |
| 目錄設定改 source、target、單元 | 下載內容不可信 |
| 覆寫既有檔案 | 改名不應毀損資料 |
| 跨單元關聯 | 關聯範圍以單元為界 |
| 依數量切批次 | 會拆散同一群組的檔案 |
| 目錄層宣告單元 | 單元先於目錄設定決定 |
| 符號連結 | 現階段略過，不處理也不跟隨 |
| 檔案內容標籤（EXIF、ID3） | 讀內容破壞純函式承諾 |
| 從檔名推斷作品名 | 作品名由使用者指定 |
