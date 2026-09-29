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
| 外圍 | `effects`、`watcher` 與 `main` |
| 平台 | 監看模組以 `cfg(target_os = "linux")` 隔開 |

`Dockerfile` 只複製 `Cargo.toml`、`Cargo.lock` 與 `src`，`.dockerignore` 是白名單，release-please 只管根套件，所以多 crate workspace 會破壞映像建置與發版。

## 模組地圖

```
  main ─► watcher ─► engine ─► stages ─► template ─► record
             │          │
             └─► config ┘
  engine ─► effects            only effects touch the filesystem
```

依賴由外向內指向 `record`，內層不知道外層。

## 模組責任

| 模組 | 責任 | 碰外部 |
|---|---|---|
| `record` | 記錄與欄位 | 否 |
| `template` | 名稱樣板輸出與反推 | 否 |
| `stages` | 各階段 | 否 |
| `engine` | 批次、認領與管線執行 | 否 |
| `config` | 設定解析、層疊與驗證 | 讀檔 |
| `effects` | `move` 與 `cleanup` | 檔案系統 |
| `watcher` | 事件、單元與批次收束 | notify、時間 |

`engine` 與 `watcher` 的狀態機吃注入的資料與時鐘，所以不需要真實的檔案系統就能測試。

## 測試分層

| 層 | 方式 |
|---|---|
| 核心 | 以 TOML 管線加 `docs/cases.md` 的案例做整合測試 |
| `effects` | 暫存目錄 |
| `watcher` 狀態機 | 注入時鐘與事件 |
| notify 與跨檔案系統 | 容器內的 Linux 與兩個 volume |

邊界內的行為由 `.spec/behavior` 宣告，測試以 `@behavior` 認領。
