# `src/github/fetcher.rs` 処理概要

## 1. モジュール構成

```
fetcher.rs
├── get_github_token()     # GitHubトークン取得（ヘルパー関数）
├── RepoRef                # リポジトリ参照を表す構造体
└── GitHubClient           # GitHub APIクライアント
```

---

## 2. `get_github_token()` (8-27行目)

**役割**: GitHub認証トークンを取得

**優先順位**:
1. `GITHUB_TOKEN` 環境変数（CI/CD用）
2. `gh auth token` コマンド（ローカル開発用）

**戻り値**: `Option<String>` - トークンがあれば返す、なければ `None`

---

## 3. `RepoRef` 構造体 (29-69行目)

**役割**: GitHubリポジトリ参照を表す

| フィールド | 型 | 説明 |
|-----------|------|------|
| `owner` | `String` | オーナー名 |
| `repo` | `String` | リポジトリ名 |
| `git_ref` | `Option<String>` | ブランチ/タグ/コミット |

**メソッド**:
- `parse(input)` - `"owner/repo"` または `"owner/repo@ref"` 形式をパース
- `ref_or_default()` - 指定refがあればそれを、なければ `"HEAD"` を返す

---

## 4. `GitHubClient` 構造体 (71-279行目)

**役割**: GitHub API操作を提供

### 4.1 コンストラクタ

| メソッド | 説明 |
|---------|------|
| `new()` | 新規クライアント作成（base_url: `https://api.github.com`）|

### 4.2 内部ヘルパー

| メソッド | 説明 |
|---------|------|
| `auth_token()` | `get_github_token()` を呼び出し |

### 4.3 パブリックAPI

| メソッド | 戻り値 | 説明 |
|---------|-------|------|
| `get_default_branch(repo)` | `Result<String>` | デフォルトブランチ名を取得 |
| `download_archive(repo)` | `Result<Vec<u8>>` | リポジトリをzipでダウンロード |
| `download_archive_by_tag(repo, tag)` | `Result<Vec<u8>>` | 特定タグをzipでダウンロード |
| `get_commit_sha(repo, git_ref)` | `Result<String>` | コミットSHAを取得 |
| `download_archive_with_sha(repo)` | `Result<(Vec<u8>, String, String)>` | zip + ブランチ名 + SHA を返す |
| `fetch_file(repo, path)` | `Result<String>` | 単一ファイル内容を取得 |

### 4.4 内部メソッド

| メソッド | 説明 |
|---------|------|
| `download_with_progress(url)` | プログレスバー付きダウンロード |

---

## 5. 処理フロー図

```
download_archive(repo)
    │
    ├─ repo.git_ref がある？
    │     YES → そのrefを使用
    │     NO  → get_default_branch() でデフォルトブランチ取得
    │
    └─ download_with_progress(url)
           │
           ├─ 認証ヘッダー付与（トークンあれば）
           ├─ リクエスト送信
           ├─ プログレスバー表示
           │     ├─ サイズ既知 → バー表示
           │     └─ サイズ不明 → スピナー表示
           └─ バイト列返却
```

---

## 6. 使用するGitHub APIエンドポイント

| 用途 | エンドポイント |
|-----|---------------|
| リポジトリ情報取得 | `GET /repos/{owner}/{repo}` |
| zipダウンロード | `GET /repos/{owner}/{repo}/zipball/{ref}` |
| コミットSHA取得 | `GET /repos/{owner}/{repo}/commits/{ref}` |
| ファイル取得 | `GET /repos/{owner}/{repo}/contents/{path}?ref={ref}` |

---

## 7. テスト (281-316行目)

`RepoRef::parse()` のテスト:
- シンプル形式: `"owner/repo"`
- ref付き: `"owner/repo@v1.0.0"`
- ブランチ付き: `"owner/repo@main"`
- 無効な形式のエラー確認
