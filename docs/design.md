# auto-renamer 設計

auto-renamer 監控資料夾的檔案事件，依路線上的管線改寫檔案的名稱與位置，再把檔案移到具名的 target。

```
  source ─► watch ─► route: pipeline ─► effects ─► target
```

## 0 總覽

### 0.1 設計承諾

| 承諾 | 意義 |
|---|---|
| 事件驅動 | 由 Linux 檔案系統事件觸發處理 |
| 小階段堆疊 | 每個階段只做一件事，能力來自組合 |
| 前純後動 | 管線是純函式，效果在路線上 |
| 不確定就不動 | 無人接手的失敗留在 source |
| 關聯限於單元 | 不同單元的檔案不分組、不共號 |
| 只信全域設定 | 目錄設定不能改 source、target、路線與單元 |
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
  │ routes     │  pure pipelines → plans
  └───┬────────┘
      │ plan
  ┌───▼────────┐
  │ filesystem │  effects: source → target
  └────────────┘
```

每層只與相鄰的層溝通。設定在啟動與設定檔變動時讀取，交給路線使用。

### 0.3 處理流程

```
  /Downloads/Movies/XXX/XXX.mp4         download finishes
    └ watcher    close-write, file settles
    └ pipeline   XXX.mp4 → XXX-s1e1.mp4
    └ move       rename, or mv across mounts
  /Video/Movies/XXX/XXX-s1e1.mp4        ready for the media server
```

source 與 target 分離，媒體伺服器只掃描 target，它產生的 `.nfo` 等檔案不會再被改名。

### 0.4 單元、批次與一組

| 層 | 決定什麼 | 依據 |
|---|---|---|
| 單元 | 檔案在哪裡收集、何時處理 | watch 的 `unit` 與收齊（1.4） |
| 批次 | 單元的一次處理 | 收齊時已穩定的檔案 |
| 一組 | 誰和誰有關 | watch 的 `group`（2.10） |

單元收齊就處理一個批次，路線認領的那部分也是一個批次。一組由欄位相同的檔案組成，與資料夾無關，例如同一集的影片與字幕。

### 0.5 進度

| 章 | 依賴 | 進度 |
|---|---|---|
| 1 監控 | 0 | 🚧 |
| 2 管線與路線 | 1 | 🚧 |
| 3 設定 | 2 | 🚧 |
| 4 執行行為 | 2、3 | 🚧 |
| 5 內建階段 | 2 | 🚧 |
| 6 Playground | 2、3、4 | 🚧 |

✅ 該章每一句都有 `.spec/behavior` 的行為與測試對應，4.6 標明由量測確認的除外；🚧 尚有句子未對應。章節依實作的依賴排列。

### 0.6 建置與發行

| 產物 | 說明 |
|---|---|
| Linux binary | 靜態 musl |
| 容器映像 | `scratch`，push main 更新 `latest` |
| Playground | GitHub Pages，push main 更新 |

映像沒有 shell、CA 憑證與時區資料，所以只做本機檔案操作。inotify 是核心功能，不受影響。版本由 release-please 管理，release 時映像加上版本標籤。

## 1 監控

### 1.1 監控範圍

| 事件 | notify 對應 | 用途 |
|---|---|---|
| 寫入完成 | `Access(Close(Write))` | 檔案可以處理 |
| 搬入 | `Modify(Name(To))` | 同檔案系統的 `mv` 沒有寫入完成事件 |
| 資料夾出現 | `Create(Folder)`、`Name(To)` | 確認監控後掃描，補上漏報的檔案 |

每個 watch 遞迴監控自己的 source，事件由 notify crate 的 inotify 後端提供，不跟隨符號連結。

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

### 1.4 單元收齊

```
  a.mkv settled ┐
  b.mkv settled ├─ same unit ─► batch ─► routes
  c.mkv settled ┘  closes when quiet, or after a maximum wait
```

| 設定 | 作用 | 預設 |
|---|---|---|
| `quiet` | 安靜這麼久就收齊 | 5 分鐘 |
| `max_wait` | 從第一個檔案起最多等這麼久 | 30 分鐘 |
| `max_files` | 單元超過這個檔案數就不動 | 1000 |

收齊依單元各自計算，`max_wait` 不可小於 `quiet`。批次不依數量切開，以免拆散一組。`max_files` 擋下誤放的大量檔案，不得超過 10 萬；收集到這個數量就不再記錄更多。不動的批次留在 source 並記錄原因。

### 1.5 保留檔案

| 檔案 | 處理 |
|---|---|
| `auto-renamer.toml` | 目錄設定；不進管線、不搬移、不刪除 |

`auto-renamer.toml` 是設定而非素材，由使用者管理，不計入 `max_files`。它在每次處理單元時讀取，改了就從下一個批次生效。

### 1.6 啟動掃描

```
  start ─► scan source ─► existing files ─► settled ─► batch
                                   └ mtime within quiet: settled after a quiet period
```

啟動時掃描 source 內既有的檔案，視同穩定事件進入批次，停機期間到達的檔案不會被遺漏。修改時間在 `quiet` 內的檔案，等修改時間過了 `quiet` 且期間沒有寫入事件才算穩定，因為它之後不會再有寫入完成事件。

### 1.7 資料夾監控

```
  folder ─► confirm it is watched ─► read its content
```

每個資料夾都先確認已被監控才讀取，啟動時既有的與之後出現的都一樣。讀取前建立的檔案由掃描找到，之後的由事件通報，中間沒有空檔。程式啟動時監控不成就不啟動；啟動完成後，例如重新載入時，只記錄是哪一個。

### 1.8 通知篩選

```
  notification ─► says a file changed? ─ no ──► dropped at the door
                                       └ yes ─► bounded queue ─► loop
```

開啟與讀取不代表檔案改變，在進入佇列前就丟棄。佇列有上限，別的程式大量讀取 source 不會佔用處理的時間與記憶體。

### 1.9 重新同步

```
  queue full | kernel queue overflow ─► notifications were lost
                                     ─► drop what is waiting ─► scan the source again
```

通知遺失後，程式知道的狀態已不可信。這時等同重新啟動：放下等待中的批次，照 1.6 重新掃描 source。檔案仍在原地，所以不會遺漏。一批搬走的檔案比佇列容量多時也算遺失，留在 source 的檔案會再判斷一次。

### 1.10 工作排程

| 每圈處理 | 上限 |
|---|---|
| 通知 | 固定數量 |
| 掃描 | 固定數量的項目 |
| 批次 | 一個 |

迴圈每圈只做固定的量，停止訊號與新事件不必等一長串批次做完。這些量是程式內的常數。沒有待辦時只等事件與最近的到期時間，不掃描 source 也不查看 target，只為了回應停止訊號每秒醒來一次。

## 2 管線與路線

### 2.1 處理模型

```
  batch (sorted by name)
   │ route 1 claims ─► pipeline: map … scan … ─► planned ─┐
   │ rest ─► route 2 claims ─► pipeline … ────────────────┤
   ▼                   │ rejected                         ▼
  unclaimed stay       └► rejected route           effects per file
```

一批檔案是依名稱排序的有序集合。路線依序認領檔案，交給自己的管線；每個階段是「批次 → 批次」的函式，逐檔階段是 map 的簡寫。單一檔案就是只有一個檔案的批次。整批規劃完，才逐檔執行效果。

### 2.2 記錄

```
  record = { origin: "Movies/XXX/XXX.mp4",
             plan:   "Movies/XXX/XXX.mp4",
             fields: { name, ext, dir, path, mtime, show, season, episode } }
```

每筆記錄含原始路徑、計畫路徑與具名欄位。原始路徑不變；計畫路徑由 source 內的相對路徑起算，即保留結構。計畫路徑的檔名永遠等於 `name` 加 `ext`，改寫這兩個欄位就是改名。欄位的起點見 2.7。

### 2.3 階段形狀

| 形狀 | 作用 | 例子 |
|---|---|---|
| filter | 放行或拒絕 | `filter` |
| map | 逐檔改寫 | `number`、`format` |
| scan | 帶累積狀態走訪 | `next` |
| group | 依欄位分組或配對 | `rank`、`take` |

四種都是純函式，管線因此可以乾跑預覽。效果不是階段，由路線執行（2.6）。正規表達式只用於內建階段處理不了的情況。

### 2.4 階段輸出

| 輸出 | 意義 |
|---|---|
| 改寫 | 記錄改變，交給下一階段 |
| 不變 | 階段不適用，原樣交給下一階段 |
| 拒絕 | 停止該檔案，交給失敗路線（2.9） |

拒絕逐檔短路，依賴被拒絕檔案的階段（如 `take`）連帶拒絕，其他檔案照常處理。階段的輸入是批次、路線 target 的現況與前面管線的結果。

### 2.5 認領與拒絕

| 情況 | 結果 |
|---|---|
| 開頭的 `filter` 接受 | 這條路線認領該檔案 |
| 開頭的 `filter` 不接受 | 交給下一條路線 |
| 中途的 `filter` 不符 | 拒絕，`reason` 是 `filter` |
| 沒有路線認領 | 留在 source |

一個 watch 可以有多條路線，檔案由第一條認領它的路線處理。管線內不寫條件分支，分支由多條路線加 `filter` 表達。

### 2.6 路線

```toml
[target.anime]
path = "/video/animate"

[target.conflict]
path = "/video/conflict"

[watch.anime]
source = "/video/downloads"
routes = [
  { pipeline = "series-video", move = "anime", cleanup = { keep = ["Season *"] }, rejected = { move = "conflict" } },
]
```

路線把管線接上效果，`routes` 依陣列的順序認領。target 只在全域設定宣告，路線以名稱引用。

| 鍵 | 作用 |
|---|---|
| `pipeline` | 認領並命名的管線 |
| `move` | 搬到具名的 target，省略就原地改名 |
| `cleanup` | 刪除單元內清空的資料夾（4.5） |
| `rejected` | 失敗路線（2.9） |

每個檔案先 `move` 再 `cleanup`。`[default]` 的 `routes` 供所有 watch 共用。

### 2.7 起始欄位

| 欄位 | 內容 |
|---|---|
| `name` | 主檔名 |
| `ext` | 副檔名 |
| `dir` | 所在資料夾名 |
| `path` | 相對於 source 的資料夾路徑 |
| `mtime` | 修改時間，UTC |
| `vars` | 設定的 `vars`，是答案（2.8） |

欄位值有文字、數字與日期三種。映像沒有時區資料，所以日期一律以 UTC 表示。路徑不是有效 UTF-8 的檔案做不出記錄，不進任何管線，留在 source 並記錄原因。

### 2.8 答案

`vars` 是使用者給的答案，用在偵測不出來或偵測錯的地方；目錄設定的答案最具體（3.4）。

| 階段 | 欄位已有答案時 |
|---|---|
| 偵測：`regex` 擷取、`number`、`next`、`rank`、`take` | 不寫入 |
| 管線的值：`set`、`default` | 不寫入 |
| 加工：`replace`、`case`、`strip`、`regex` 改寫、`format` | 照常加工 |

沒有答案的欄位，才由偵測或管線填入。全域靠偵測涵蓋多數情況，遇到例外，只要在那個資料夾給一個答案。

### 2.9 失敗路線

```
  route ─ pipeline ─ planned ─────────────► move, cleanup
             │ rejected: planned, reason
             └► rejected route ─ pipeline ─► move, cleanup
                     │ rejected again, or unclaimed
                     └► stays in source
```

被拒絕的記錄只走 `rejected`，不回到一般的認領。它帶著兩個欄位：

| 欄位 | 內容 |
|---|---|
| `planned` | 拒絕當下的計畫路徑 |
| `reason` | 拒絕它的階段；撞名是 `move`，連帶是 `group` |

起始欄位重設回原始值，其他欄位保留，所以預設是原檔名與原結構。失敗路線的鍵與路線相同，但沒有 `rejected`，`pipeline` 可省略，也不繼承 `cleanup`。它只走一層：再次拒絕或不認領，檔案就留在 source。

### 2.10 一組

一組是同一批次中 `group` 列出的欄位都相同的檔案，可跨路線，例如 `group = ["show", "season", "episode"]` 讓同一集的影片與字幕成為一組。

| 情況 | 結果 |
|---|---|
| 組內有檔案被拒絕 | 其他組員也拒絕，`reason` 是 `group` |
| 缺 `group` 的欄位 | 不屬於任何一組 |
| 不寫 `group` | 不連帶 |
| 不同批次的組員 | 各自處理 |

連帶在所有路線規劃完、失敗路線開始前判定。

## 3 設定

### 3.1 全域設定

```toml
[default]
quiet = "5m"
routes = [{ pipeline = "video", move = "video" }]

[pipeline.video]
stages = [{ filter = { ext = ["mkv", "mp4"] } }]

[target.video]
path = "/Video"

[watch.series]
source = "/Downloads"
unit = { root = ["Movies/*"] }
vars = { show = "Alpha" }
```

`pipeline.*` 定義具名管線，`target.*` 宣告搬移的根；`watch` 綁定 source、`unit`、`group`、`vars` 與 `routes`；`default` 供所有 watch 共用。

### 3.2 管線宣告

| 寫法 | 意義 |
|---|---|
| `"strip"` | 沒有參數的階段 |
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
]
```

資料夾內的 `auto-renamer.toml` 覆蓋 watch 的設定，處理例外。從 source 往下到單元所在的資料夾，逐層尋找並由遠到近套用，最近的優先。

### 3.4 覆寫順序

```
  built-in < [default] < [watch.*] < auto-renamer.toml (parent) < (nearest)
  low                                                              high
```

後面的層覆蓋前面的層，內建管線（5.13）在最低層。`vars` 逐鍵覆蓋，`pipeline.*` 依名稱整條取代，不逐階段合併。

### 3.5 覆寫界線

| 項目 | 目錄設定 |
|---|---|
| 管線、`vars`、`max_files` | 可覆寫，`max_files` 不得超過 10 萬 |
| `source`、`unit`、`group`、`routes`、`dry_run` | 不可覆寫 |
| `target.*` | 只在全域設定宣告 |
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

設定變動時重新載入，目錄設定則每個批次讀取，都不需重啟。讀取設定檔也會產生事件，只有寫入與換掉才算變動；單獨掛載的檔案沒有事件，用 SIGHUP 觸發。停止時放棄等待中的批次，檔案仍在 source，下次啟動會重新掃描。

### 3.7 乾跑

```toml
[watch.series]
dry_run = true      # log each plan, run no effect
```

`dry_run = true` 時路線照常規劃，把每個檔案的原路徑、計畫路徑與拒絕原因寫進 log，但不執行效果。被拒絕的檔案記為原路徑、`planned` 與 `reason`。預設關閉，適合第一次寫管線時先看結果。

## 4 執行行為

### 4.1 路徑對應

```
  source root   /Downloads
  plan          Movies/XXX/XXX-s1e1.mp4
  target root   /Video
  result        /Video/Movies/XXX/XXX-s1e1.mp4
```

結果是路線 target 的根加計畫路徑，原地改名時是 source 的根；缺少的目錄會建立。計畫路徑離開根時拒絕。

### 4.2 跨檔案系統

```
  same mount    rename without replacing ─► done
  other mount   mv -n to a temp name ─► rename without replacing ─► done
```

`rename` 跨不過掛載點，同一顆硬碟掛兩處也不能，這時由 `mv -n` 搬到 `.` 開頭的暫存名再改名。中斷不留下看似完整的檔案，殘留的暫存檔也不擋下次搬移。複製方式與檔案屬性由 `mv` 決定，出錯時如下：

| 情況 | 結果 |
|---|---|
| 暫存名已被佔用 | 拒絕，不交給 `mv` |
| `mv` 失敗 | 拒絕，刪掉它留下的暫存檔 |
| 最後一步撞名 | 拒絕，保留暫存檔並記錄路徑 |

### 4.3 撞名

```
  plan taken in target          ─► reject, reason move     planning
  same plan as an earlier file  ─► reject, reason move     planning
  taken after planning          ─► stays in source         moving
```

`move` 在規劃時比對路線 target 的現況與同批較早的計畫，撞名就拒絕並交給失敗路線，例如搬到另一個 target。任何情況都不覆寫既有檔案，符號連結與資料夾不處理也不跟隨。同一檔案系統以不覆寫的 rename 搬移，監控者看到的是搬入。

### 4.4 冪等

| 情況 | 結果 |
|---|---|
| target 位於 source 內 | 設定驗證拒絕，程式不啟動 |
| watch 的 source 彼此重疊 | 設定驗證拒絕 |
| target 與任何 source 重疊 | 設定驗證拒絕 |
| 設定檔位於 source 內 | 設定驗證拒絕，程式不啟動 |
| source 或 target 寫有 `.`、`..` | 設定驗證拒絕 |
| 事件來自 target | 不監控，不處理 |
| 計畫路徑等於目前路徑 | 不動作 |
| 原地連續改名 5 次 | 第 6 次拒絕並記錄 |

位置在啟動與重新載入時以解析連結後的實際路徑再比一次，因為監控跟著連結到實際的資料夾。改名與搬移會產生新事件，冪等確保處理過的檔案不再處理。每次改寫自己結果的管線不冪等，深度限制擋下無限改名；判定為不動的檔案歸零。

### 4.5 清理

```
  Season 01/[Rel]/Subs/   empty after move ─► removed
  Season 01/[Rel]/        empty             ─► removed
  Season 01/              matches keep      ─► kept, stop
  Series/                  above the unit    ─► never touched
```

路線的 `cleanup` 只刪除單元內清空的資料夾，含單元本身，由檔案原本所在處往上，遇到非空、含 `auto-renamer.toml`、符合 `keep` 或是連結的資料夾就停。source 根永不刪除。

### 4.6 規模

| 項目 | 承諾 | 確認 |
|---|---|---|
| 等待中的檔案 | 10 萬個以內，總成本線性成長 | 測試 |
| 通知佇列 | 有上限 | 測試 |
| `next` | 一批對一個 target 資料夾只列一次 | 測試 |
| 記憶體 | 隨等待數與批次成長，過後回落 | 量測 |

在這個範圍內，每個事件與每次到期檢查的成本不隨等待中的檔案數成長，超過 10 萬個不保證。量測用 `scripts/measure.sh`，數字隨機器而異。

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
| 掃描的資料夾 | 路線 target 中計畫路徑所在的資料夾 |

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
| `invert` | 反轉結果，用於擋下特定檔案 |

`ext` 與 `glob` 同時給時都要符合。開頭的 `filter` 認領檔案，中途的 `filter` 拒絕檔案（2.5）。

### 5.9 名稱改寫

| 階段 | 參數 | 作用 |
|---|---|---|
| `regex` | `pattern`、`from`、`into` | 擷取，具名群組成欄位 |
| `regex` | `pattern`、`from`、`replace` | 改寫欄位內容 |
| `set` | 欄位 = 值 | 設定固定值，不覆蓋答案 |
| `replace` | `find`、`with`、`field` | 字面取代 |
| `case` | `to`、`field` | `lower`、`upper`、`title` |
| `strip` | `groups`、`field` | 去掉括號標籤 |

`from` 與 `field` 預設是 `name`。`regex` 用 Rust `regex` crate 的語法，沒有比對到時不變，只由數字組成的擷取值成為數字。`strip` 的 `groups` 預設只有 `["[]"]`，並整理多餘空白。

### 5.10 路徑改寫

| 階段 | 參數 | 作用 |
|---|---|---|
| `lift` | `to` | 提到最近符合的祖先，沒有就不變 |
| `lift` | `keep` | 只留 source 下前幾層，不夠就不變 |
| `lift` | 整數 | 往上提幾層，超過就拒絕 |
| `folder` | 樣板 | 依樣板加入子資料夾 |

`keep` 讓不同的下載結構整理成同一層。`to` 與 `keep` 只能給一個。`folder` 的樣板以 `/` 分段，欄位缺值或任何一段不是可用的資料夾名就拒絕。

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

### 5.13 內建影集管線

```toml
[pipeline.series-video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { regex = { from = "path", pattern = '^(?<show>[^/]+)' } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  { number = { into = "episode", exclude = ["season"] } },
  { default = { season = 1 } },
  { lift = { keep = 1 } },
  { folder = "Season {season:02}" },
  { next = { into = "episode", like = "{show} s{season:02}e{episode:02}" } },
  { format = "{show} s{season:02}e{episode:02}" },
]

[pipeline.series-subtitle]
stages = [
  { filter = { ext = ["ass", "srt"] } },
  { regex = { from = "path", pattern = '^(?<show>[^/]+)' } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  { number = { into = "episode", exclude = ["season"] } },
  { default = { season = 1 } },
  { rank = { into = "index", by = ["season", "episode"], prefer = ["cht"] } },
  { lift = { keep = 1 } },
  { folder = "Season {season:02}" },
  { format = "{show} s{season:02}e{episode:02}.zh[.{index:02}]" },
]
```

這兩條管線內建，設定寫同名的管線就整條取代。它們假設作品資料夾在 source 下第一層，`show` 取自它的名稱，`vars` 可給答案。影集不是內建功能，而是階段的堆疊；字幕與影片各自跑 `number`，得到相同的季與集。

### 5.14 其他管線

```toml
[pipeline.movie]        # tags out of the name
stages = [{ filter = { ext = ["mkv", "mp4"] } }, { strip = {} }]

[pipeline.music]        # "03 - Title.mp3" → "03 Title.mp3"
stages = [
  { filter = { ext = ["mp3", "flac"] } },
  { regex = { pattern = '^(?<track>\d+)\s*-\s*(?<title>.+)$' } },
  { format = "{track:02} {title}" },
]

[pipeline.photo]        # sort into year/month by modified time
stages = [
  { filter = { ext = ["jpg", "png"] } },
  { folder = "{mtime:%Y}/{mtime:%m}" },
]
```

同一套階段處理影集以外的檔案，不需要新增階段。

## 6 Playground

在瀏覽器編輯設定，在虛擬目錄上模擬改名，再下載成全域設定或目錄設定。Playground 不監控任何資料夾，所以「只保證 Linux」只約束 CLI。

```
  virtual tree ─► trigger ─► core (WASM) ─► virtual move
  built by user   one batch   same as CLI    tree updated
```

### 6.1 真實與模擬

只有管線是真的，監控與搬移都在虛擬目錄上模擬。

| 部分 | 執行 | 方式 |
|---|---|---|
| 監控 | 模擬 | 觸發時每個單元各成一個批次 |
| 管線 | 真實 | 與 CLI 同一份核心，含目錄設定 |
| 搬移 | 模擬 | 依核心的結論搬動虛擬檔案 |

撞名、清理與乾跑的規則與 CLI 相同。虛擬目錄的根是所有 source 與 target，檔案帶修改時間。寫入穩定、收齊與原地改名次數取決於時間，不模擬。

### 6.2 匯入與下載

設定文字只由核心讀寫，所以下載的檔案與 CLI 讀到的一致。

| 動作 | 規則 |
|---|---|
| 匯入 | 能通過解析的設定都能編輯 |
| 匯入目錄設定 | 取代正在編輯的那一份 |
| 下載 | 先以 CLI 的解析器驗證 |
| 往返 | 註解與排版不保留 |

全域設定與目錄設定都能匯入與下載，下載的是正在編輯的那一份。編輯全域設定時匯入目錄設定，它放到 source 的根並打開，對每個單元生效。

### 6.3 編輯版面

設定畫在中央，虛擬目錄排在下方，觸發後兩棵樹直接顯示檔案的去向。目錄設定就是 source 樹裡的 `auto-renamer.toml`，點它就換成編輯它（6.8）。編輯目錄設定時沒有 watch，模擬一律用全域設定加上樹裡的目錄設定。

```
+---------+------------------+-----------+
| palette |      canvas      | inspector |
+---------+----+---------+---+-----------+
| source tree  | trigger | target tree   |
+--------------+---------+---------------+
| results | configuration text           |
+----------------------------------------+
```

### 6.4 畫布操作

watch 框住它的路線，管線框住它的階段，target 畫在 watch 旁。路線連到它的管線與 target，失敗路線以虛線相連。

```
  watch [ route 1 ]──────────► pipeline [ stage, stage ]
        [ route 2 ]── move ──► target
             └┄┄┄ rejected ┄┄► target
```

每個階段有一個圖示，在 palette 與畫布上相同。

#### 改寫

畫布上的操作都改寫設定表格，所以下載的就是畫出來的。

| 操作 | 改寫 |
|---|---|
| 拖入階段 | 插入在落點的位置 |
| 拖動階段到同一管線 | 調整階段順序 |
| 在 watch 內拖動路線 | 調整認領順序 |
| 拖動其他節點 | 不改寫，只移動位置 |
| watch 連到管線 | 新增只有 `pipeline` 的路線 |
| 路線連到 target | 路線搬到那個 target |
| 按連線上的 ×，或刪除連線 | 移除那條路線，或路線的搬移 |
| 刪除節點 | 移除該 watch、路線、管線、階段或 target |
| 刪除或改名管線、刪除 target | 一併改寫引用它的路線 |

watch 沒寫 `routes` 時畫出預設路線，修改前先把預設路線複製給它。失敗路線與 target 路徑改在 inspector 編輯。

#### 內建管線

路線引用、設定沒有定義的內建管線，畫成上鎖的管線，它的階段不能拖入、移動或刪除。

| 動作 | 結果 |
|---|---|
| 複製成自訂管線 | 同名管線取代內建，可編輯 |
| 在資料夾覆寫 | 目錄設定複製核心內建的階段 |

#### 節點排版

設定是基準，節點的位置只是排版，不寫進設定也不保留。階段在管線內、路線在 watch 內由上而下依序排列，不能自由接線，因為兩者都是有序清單。

| 動作 | 節點位置 |
|---|---|
| 階段或路線順序改變 | 依新順序排列 |
| 重設版面 | 所有節點依順序排列 |
| 選擇範例 | 換成範例的設定與虛擬目錄（6.9） |

### 6.5 表單

核心描述每個階段的參數，表單依此產生；解析器讀的是同一份描述，所以表單的參數就是 CLI 接受的參數。參數之間的規則，在下載前的驗證中檢查。

| 參數資訊 | 表單用途 |
|---|---|
| 名稱 | 標籤旁的原文鍵 |
| 型別 | 輸入元件 |
| 必填 | 未填時提示 |
| 可選值 | 下拉選單 |
| 範例 | 新階段的初始值 |

#### 頁面描述的鍵

watch、default 與 target 的鍵由頁面自己描述，涵蓋 CLI 讀的鍵。它們是模擬環境的輸入，重複一份比讓核心多開介面簡單，寫錯的值仍由下載前的驗證擋下。

| 鍵 | 寫在 |
|---|---|
| `source` | watch |
| `unit`、`group`、`routes`、`dry_run`、`quiet`、`max_wait` | watch、default |
| `vars`、`max_files` | watch、default、目錄設定 |
| `path` | target |

#### 路線的表單

預設路線沒有節點，所以在預設值表單逐條編輯。

| 要編輯的 | 在哪裡 |
|---|---|
| watch 一條路線的鍵 | 選取該路線 |
| watch 的路線順序 | watch 表單 |
| 預設路線 | 預設值表單 |

### 6.6 輸入輔助

能從設定或虛擬目錄得知的值，就不讓使用者整段打字。

| 輸入 | 輔助 |
|---|---|
| 新增檔案的路徑 | 依既有資料夾補全 |
| 資料夾列 | 可直接在其下新增檔案 |
| 路線與失敗路線的 `pipeline` | 從已定義與內建的管線挑選 |
| 路線與失敗路線的 `move` | 從已宣告的 target 挑選 |
| `cleanup` 的 `keep` | 填樣式清單 |
| `unit` | 從三種寫法挑選，`root` 填樣式清單 |

### 6.7 介面語言

介面有繁體中文與英文，預設依瀏覽器偏好的語言顯示，可在頁首切換。階段、參數與設定鍵以介面語言的名稱標示，旁附 CLI 的原文名稱；名稱與說明只存在介面語言中，核心不描述它們。設定文字與核心的訊息維持原文。

| 瀏覽器偏好 | 介面語言 |
|---|---|
| `zh` 開頭 | 繁體中文 |
| 其他 | 英文 |

### 6.8 目錄設定的入口

目錄設定放在資料夾裡才生效，所以入口都從資料夾出發，建立後直接打開；已有一份就打開它。

| 入口 | 新建的內容 |
|---|---|
| 資料夾列常駐的按鈕 | watch 的 `vars` |
| 頁首切換器的新增 | watch 的 `vars` |
| 管線的「在資料夾覆寫」 | 再加上該管線 |

source 樹頂端提示從資料夾新增例外。內容複製自涵蓋該資料夾的 watch，不從資料夾名推斷作品名；覆寫只換掉同名的管線。

### 6.9 範例

頁首有範例選單。範例是一組設定、虛擬目錄與要觸發的 watch，選擇後換掉目前的內容，開頁時載入單檔集數。

| 範例 | 來源 |
|---|---|
| 完整組合 | 影集情境疊在一起 |
| 單檔集數到撞名 | `cases.md` 第 1 到 3 章 |
| 電影、音樂、照片 | `cases.md` 第 4 章 |
| 清理 | `cases.md` 5.2 |

| 完整組合涵蓋 | 寫法 |
|---|---|
| 作品名 | 取自作品資料夾（5.13） |
| 季數與集數 | 5.13 的內建管線 |
| 撞名 | 失敗路線搬到另一個 target |
| 連帶 | `group` |
| 換名的季 | 季資料夾的 `vars` |
| 一律遞增 | 不擷取集數的目錄設定 |

完整組合假設 source 與 target 分開，目錄設定只放在例外的資料夾。每個範例觸發後的去向都列在 `cases.md`。

### 6.10 中間狀態

觸發後，每個檔案都看得到它走過的每一步：認領它的路線，每個階段之後的欄位與計畫路徑，直到拒絕。

```
  claimed by series-video ─► number: season = 3 ─► number: episode = 10 ─► format: name = Zeta-Show s03e10 ─► move
```

| 看的位置 | 顯示 |
|---|---|
| 結果分頁的檔案 | 每一步改了什麼 |
| 畫布上的階段 | 跑完這一步的檔案 |
| 開頭的 `filter` | 認領的檔案 |

走的是目錄設定取代的管線，在階段的清單上另外標示。核心在規劃時逐步交出中間狀態，CLI 不收集，規模與記憶體不受影響。

## 附錄

### 刻意排除

設計明確不做的事，避免日後被誤當成待辦。

| 項目 | 理由 |
|---|---|
| 非 Linux 平台 | 事件語意依賴 inotify |
| 目錄設定改 source、target、路線、單元 | 下載內容不可信 |
| 覆寫既有檔案 | 改名不應毀損資料 |
| 跨單元關聯 | 關聯範圍以單元為界 |
| 依數量切批次 | 會拆散一組 |
| `max_files` 的寬容區間 | 超過時整個單元不動，不會只做一半 |
| 目錄層宣告單元 | 單元先於目錄設定決定 |
| 符號連結 | 現階段略過，不處理也不跟隨 |
| ASCII 與 UTF-8 以外的路徑編碼 | 轉成文字後會指向別的檔案 |
| 檔案內容標籤（EXIF、ID3） | 讀內容破壞純函式承諾 |
| 從檔名推斷作品名 | 作品名由使用者指定 |
