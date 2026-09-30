# 架構

auto-renamer 是單一套件：`src/lib.rs` 放全部邏輯，`src/main.rs` 只負責串接。

```
  src/lib.rs   all logic, as modules
  src/main.rs  wiring only
```

## 結構風格

| 決定 | 內容 |
|---|---|
| 套件 | 單一套件，`lib.rs` 加 `main.rs` |
| 邊界 | 模組，不是 crate |
| 核心 | 純函式，不碰檔案系統、時間與 notify |
| 外圍 | `effects`、`scan`、`service`、`runner` 與 `main` |
| 平台 | `runner` 以 `cfg(target_os = "linux")` 隔開，其他平台的 `main` 直接拒絕 |

`Dockerfile` 只複製 `Cargo.toml`、`Cargo.lock` 與 `src`，`.dockerignore` 是白名單，release-please 只管根套件，所以多 crate workspace 會破壞映像建置與發版。

## 模組地圖

```
  main ─► runner ─┬─► service ─┬─► engine ──► pipeline ─► stages ─┬─► template ─► record
                  │            ├─► effects ─► stages               ├─► context
                  │            └─► config ──► pipeline, reader     └─► reader ───► record
                  ├─► watcher ─► config
                  └─► scan ────► watcher
  only effects, scan, service and runner touch the filesystem
```

依賴由外向內指向 `record`，內層不知道外層。

## 模組責任

| 模組 | 責任 | 碰外部 |
|---|---|---|
| `record` | 記錄與欄位 | 否 |
| `template` | 名稱樣板輸出與反推 | 否 |
| `context` | target 現況與同批已發出的號碼 | 否 |
| `stages` | 階段、宣告解析、批次與 effect 描述 | 否 |
| `pipeline` | 階段清單與順序驗證 | 否 |
| `engine` | 認領順序與每個檔案的結論 | 否 |
| `reader` | 逐鍵讀取 TOML 表格，拒絕剩下的鍵 | 否 |
| `config` | 設定解析、層疊與驗證 | 否 |
| `effects` | 執行 `move` 與 `cleanup` | 檔案系統、`mv` |
| `watcher` | 事件、單元與批次收束的狀態機 | 否 |
| `watcher` 的翻譯 | notify 事件轉成單元事件 | 否 |
| `watcher` 的佇列 | 篩選通知、限制數量、記下遺失 | 否 |
| `scan` | 分次列出資料夾內的一般檔案 | 檔案系統 |
| `service` | 批次處理、目錄設定、改名次數 | 檔案系統 |
| `cli` | 命令列參數 | 否 |
| `runner` | notify、時鐘、每圈的工作量、重新載入、停止 | notify、時間、設定檔 |

`engine` 與 `watcher` 的狀態機吃注入的資料與時鐘，所以不需要真實的檔案系統就能測試。

## 階段模式

### 階段 trait

```
  trait Stage { name; run(batch, context) }
  Declared (enum) ── one match ──► &dyn Stage
```

每個階段實作 `Stage`，是一個作用在批次上的函式。`Declared` enum 只是可宣告的封閉清單，全 crate 只有一處把它轉成 `&dyn Stage`，新增階段時編譯器會在那裡要求補上。

### 批次組合

| 形狀 | `Batch` 上的用法 | 例子 |
|---|---|---|
| filter、map | `each` | `filter`、`number` |
| scan | `each` 加 `Context` | `next` |
| group | `live` 或 `earlier`，再 `each` | `rank`、`take` |
| effect | `schedule` | `move`、`cleanup` |

形狀是階段向 `Batch` 要的東西，不是階段的分類。`each` 是唯一決定檔案能否繼續的地方：階段回 `Err(Stop)`，之後的階段就看不到那個檔案。

### 輸入樣式

```rust
let rewritten = change(text_field(stage, &record, field)?);
```

階段用回傳 `Result<_, Stop>` 的讀取函式說出它要的輸入，對不上就以 `?` 帶著理由停下。路徑不是有效 UTF-8 的檔案在 `Record::new` 就被拒絕，所以階段拿到的記錄一定有名稱欄位。

### Effect 描述

```
  plan     move.run ──► batch.schedule(Effect::Move)    no filesystem
  perform  service  ──► apply_effects(file.effects)     per file, in order
```

effect 階段在規劃時只記下要做的事，整批規劃完才由 `effects` 逐檔執行。核心因此不碰檔案系統；乾跑時 `move` 只回報會搬到哪裡，`cleanup` 不執行。「effect 排在最後」是 `pipeline` 的宣告規則，引擎不依賴它。

### 檢視時機

| 情況 | 要重新決定的事 |
|---|---|
| 階段由外部提供 | enum 換成 `Box<dyn Stage>` |
| effect 要看前一個的結果 | effect 改為當場執行 |
| 支援非 UTF-8 路徑 | 記錄帶位元組形狀 |

這些情況出現時，上面的模式不再是最小的做法。

## 測試分層

| 層 | 方式 |
|---|---|
| 核心 | 以 TOML 管線加 `docs/cases.md` 的案例做整合測試 |
| `effects` | 暫存目錄 |
| `watcher` 狀態機 | 注入時鐘與事件 |
| `scan`、`service` | 暫存目錄 |
| notify 與跨檔案系統 | 容器內的 Linux 與 `/dev/shm` |
| 訊號與 log | 啟動真實的程式，讀它的結束碼與 stderr |

邊界內的行為由 `.spec/behavior` 宣告，測試以 `@behavior` 認領。
