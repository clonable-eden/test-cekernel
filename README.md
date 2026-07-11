# test-cekernel

[cekernel](https://github.com/clonable-eden/cekernel) の実証テスト用リポジトリ。Rust + Axum による HTMX フロントエンド付き TODO Web アプリ。

## Setup

ホストに Node.js 24 LTS + pnpm が必要（devcontainer CLI 用）。Rust はコンテナ内に閉じ込める。

```bash
make setup   # pnpm install (devcontainer CLI)
make up      # devcontainer 起動
make run     # サーバー起動 (http://localhost:3000)
```

## Make Targets

全ターゲットの一覧は `make help` で確認できる。

## API

### HTML/HTMX エンドポイント

| Method | Path | Description |
|--------|------|-------------|
| GET | / | TODO 一覧ページ（HTMX フロントエンド） |
| POST | / | TODO 作成（フォーム送信、HTMX） |
| POST | /todos/{id}/toggle | 完了状態トグル（HTMX） |
| POST | /todos/{id}/delete | 削除（HTMX） |

### REST API エンドポイント

| Method | Path | Description |
|--------|------|-------------|
| GET | /health | ヘルスチェック (`{"status":"ok"}` or `503 {"status":"unhealthy","error":"..."}`) |
| GET | /todos | 一覧取得 |
| POST | /todos | 作成 (`{"title": "...", "content": "..."}`) |
| PATCH | /todos/{id} | 更新 (`{"title": "...", "content": "...", "completed": true}`) |
| DELETE | /todos/{id} | 削除 |

## Development

TDD アプローチで開発する。

1. **RED**: テストを書く → `make test` で失敗を確認
2. **GREEN**: 最小限の実装 → `make test` でパス
3. **REFACTOR**: リファクタ → `make check` で全チェック
4. **PR前**: `make ci` で CI 再現確認

## Project Structure

```
src/
  main.rs        — エントリポイント（DB接続・サーバー起動・tracing初期化）
  lib.rs         — ルーター・DB初期化・re-exports
  models.rs      — データモデル (Todo, CreateTodo, UpdateTodo, CreateTodoForm)
  handlers.rs    — CRUD + HTMX ハンドラ
  errors.rs      — エラー型 (AppError, ErrorResponse)
templates/
  index.html     — メインページテンプレート (Askama)
  todo_item.html — TODO アイテム部分テンプレート
  todo_list.html — TODO リスト部分テンプレート
tests/
  api.rs         — インテグレーションテスト（REST API・HTML・エラー応答・tracing）
dev.mk           — 開発用 Makefile インクルード
renovate.json    — Renovate 設定（依存自動更新）
rust-toolchain.toml — Rust ツールチェーン指定
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
