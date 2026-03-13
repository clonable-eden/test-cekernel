# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

cekernelの実証テスト用リポジトリ。Rust + Axum によるシンプルなTODOリスト REST API。

## Development Environment

すべての開発はdevcontainer内で行う。ホストには Node.js 24 LTS + pnpm が必要（devcontainer CLI 用）。Rustはコンテナ内に閉じ込める。

```bash
make setup   # ホスト側: pnpm install (devcontainer CLI)
make up      # devcontainer 起動
make test    # コンテナ内で cargo test
make build   # コンテナ内で cargo build
make run     # コンテナ内で cargo run (http://0.0.0.0:3000)
make fmt     # コード整形 (cargo fmt)
make lint    # clippy チェック
make check   # fmt → lint → test 一括実行
make ci      # CI 再現 (fmt --check → clippy → test → build --release)
make down    # devcontainer 停止
```

git worktreeで複数コンテナを同時起動可能。コンテナ名は `cekernel-<フォルダ名>` で一意になる。

## Development Process (TDD)

1. **RED**: テストを書く → `make test` で失敗を確認
2. **GREEN**: 最小限の実装 → `make test` でパス
3. **REFACTOR**: リファクタ → `make check` で全チェック（fmt + lint + test）
4. **PR前**: `make ci` でCI再現確認

### Worker ライフサイクル

cekernel Worker は以下の順序で devcontainer を利用する:

```
make up → TDD サイクル (RED → GREEN → REFACTOR) → make ci → PR作成 → make down
```

## Architecture

- **Framework**: Axum 0.7（軽量Web）+ SQLite（sqlx async）
- **エントリポイント**: `src/main.rs` — DB接続・サーバー起動（`PORT` 環境変数でポート指定可）
- **ルーター・DB初期化**: `src/lib.rs` — `app()`, `setup_db()`, re-exports
- **モデル**: `src/models.rs` — `Todo`, `CreateTodo`, `UpdateTodo`
- **ハンドラ**: `src/handlers.rs` — CRUD ハンドラ
- **DB**: SQLite ファイル (`todos.db`)。コンテナ内でのみ存在し、gitignore済み。マイグレーションは `setup_db()` でアプリ起動時に実行
- **テスト**: `tests/api.rs` — インテグレーションテスト。`sqlite::memory:` を使うためDBファイル不要

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /todos | 一覧取得 |
| POST | /todos | 作成 (`{"title": "..."}`) |
| PATCH | /todos/:id | 更新 (`{"title": "...", "completed": true}`) |
| DELETE | /todos/:id | 削除 |

## Conventions

- TDDアプローチ: テストを `tests/api.rs` に追加してから実装
- 状態管理は `Arc<SqlitePool>` を Axum の `State` extractor で共有
- テストは `tower::ServiceExt::oneshot` でルーターに直接リクエストを送る（HTTPサーバー不要）
