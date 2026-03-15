# test-cekernel

[cekernel](https://github.com/clonable-eden/cekernel) の実証テスト用リポジトリ。Rust + Axum によるシンプルな TODO リスト REST API。

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
