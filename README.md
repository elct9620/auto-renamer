# auto-renamer

依管線改寫檔案的名稱與位置，再把檔案移到目標目錄。由 Linux 檔案系統事件觸發，不限媒體類型。

```
  source ─► watch ─► pipeline ─► target
```

## 安裝

| 產物 | 取得方式 |
|---|---|
| Linux binary | GitHub Releases，靜態 musl，x86_64 與 aarch64 |
| 容器映像 | `ghcr.io/elct9620/auto-renamer`，`latest` 或版本標籤 |

只支援 Linux，事件語意依賴 inotify。映像是 `scratch`，沒有 shell，所以只做本機檔案操作。

## 快速開始

寫一份設定，把 `/Downloads` 裡的影片移到 `/Video`：

```toml
[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  "move",
]

[watch.downloads]
source = "/Downloads"
target = "/Video"
pipelines = ["video"]
```

```
auto-renamer --config /etc/auto-renamer/config.toml
```

不給 `--config` 時讀取 `/etc/auto-renamer/config.toml`。檔案穩定並安靜一個視窗（預設 5 分鐘）後才會處理。

## 容器執行

```
docker run -d \
  -v ./config:/etc/auto-renamer \
  -v /srv/downloads:/Downloads \
  -v /srv/video:/Video \
  ghcr.io/elct9620/auto-renamer
```

掛載設定檔所在的資料夾，編輯後會自動重新載入。單獨掛載一個檔案時沒有事件，改送 `SIGHUP`。

## 執行行為

| 訊號 | 效果 |
|---|---|
| `SIGHUP` | 重新讀取設定 |
| `SIGTERM`、`SIGINT` | 處理完手上的批次就結束 |

無效的設定會保留舊設定並記錄錯誤。無法判斷的檔案留在 source，並在 stderr 記錄原因。

## 資源用量

| 情境 | CPU 時間 | 記憶體 |
|---|---|---|
| 閒置 | 每分鐘約 10 ms | 不到 2 MB |
| 啟動，10 萬個新檔等待中 | 0.5 s | 45 MB |
| 搬移 10 萬個檔案，每批 10 個 | 約 4 s | 尖峰 16 MB |
| 搬移 10 萬個檔案，同一批 | 3.5–5.5 s | 尖峰 140 MB |
| 跨檔案系統搬 2 GiB | 1.0 s | 3 MB |

數字量自 Apple Silicon 上的容器：記憶體可以直接參考，CPU 時間要在部署的機器上重量。同一批的檔案越多，規劃時佔的記憶體越多；預設的 `batch_max` 是 1000。`scripts/measure.sh` 只需要 docker，不帶參數執行會列出情境。

### 記憶體上限

| 容器上限 | 跨檔案系統搬 2 GiB，5 次 |
|---|---|
| 32 MB | 3 次被核心終止 |
| 64 MB | 1 次被核心終止 |
| 128 MB | 全部完成 |

跨檔案系統搬移寫出的檔案快取算在容器上，上限太低時核心會終止程式。要設上限就留 128 MB 以上，`batch_max` 調大時再加上一批的用量，並在部署的機器上確認：

```
AUTO_RENAMER_MEMORY=128m scripts/measure.sh copy 2
```

## 文件

設計與行為都有文件，依需要挑一份閱讀。

| 文件 | 內容 |
|---|---|
| `docs/design.md` | 設計、設定與內建階段 |
| `docs/cases.md` | 由設計推得的測試案例 |
| `docs/architecture.md` | 模組結構 |
| `.spec/` | sumi 規格：詞彙、行為、契約 |

## 開發

```
sumi fmt --check
sumi verify
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Linux 專屬的測試（監看、訊號、跨檔案系統）在其他系統上略過，用 `docker compose run --rm test` 執行。提交依 Conventional Commits，版本由 release-please 管理。

## 授權

專案以 Apache-2.0 授權，作者與年份填在授權全文中。

| 授權 | 全文 |
|---|---|
| Apache License 2.0 | `LICENSE` |
