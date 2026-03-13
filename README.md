# test-cekernel

[cekernel](https://github.com/clonable-eden/plugins) の実証テスト用リポジトリ。Rust + Axum によるシンプルな TODO リスト REST API。

## Setup

ホストに Node.js 24 LTS + pnpm が必要（devcontainer CLI 用）。Rust はコンテナ内に閉じ込める。

```bash
make setup   # pnpm install (devcontainer CLI)
make up      # devcontainer 起動
make run     # サーバー起動 (http://localhost:3000)
```

## Make Targets

| Target | Description |
|--------|-------------|
| `make setup` | ホスト依存関係インストール |
| `make up` | devcontainer 起動 |
| `make down` | devcontainer 停止 |
| `make test` | テスト実行 |
| `make build` | ビルド |
| `make run` | サーバー起動 |
| `make fmt` | コード整形 (`cargo fmt`) |
| `make lint` | lint (`cargo clippy`) |
| `make check` | fmt → lint → test 一括実行 |
| `make ci` | CI 再現 (fmt --check → clippy → test → build --release) |

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | /todos | 一覧取得 |
| POST | /todos | 作成 (`{"title": "..."}`) |
| PATCH | /todos/:id | 更新 (`{"title": "...", "completed": true}`) |
| DELETE | /todos/:id | 削除 |

## Development

TDD アプローチで開発する。

1. **RED**: テストを書く → `make test` で失敗を確認
2. **GREEN**: 最小限の実装 → `make test` でパス
3. **REFACTOR**: リファクタ → `make check` で全チェック
4. **PR前**: `make ci` で CI 再現確認

## Project Structure

```
src/
  main.rs        — エントリポイント（DB接続・サーバー起動）
  lib.rs         — ルーター・DB初期化・re-exports
  models.rs      — データモデル (Todo, CreateTodo, UpdateTodo)
  handlers.rs    — CRUD ハンドラ
tests/
  api.rs         — インテグレーションテスト
```
