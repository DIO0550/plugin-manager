# plm link / plm unlink

ファイルやディレクトリ間にシンボリックリンクを作成・削除するコマンド。

## plm link

```bash
plm link <src> <dest> [--force]
```

### 引数

| 引数 | 説明 |
|------|------|
| `src` | リンク元（実体ファイル/ディレクトリ）。存在しなければエラー |
| `dest` | リンク先（シンボリックリンクが作成される場所） |

### オプション

| オプション | 説明 |
|-----------|------|
| `--force` | `dest` に既存ファイルがある場合、上書きを許可 |

### 動作

1. `src` が存在することを確認
2. `dest` が既に存在する場合はエラー（`--force` で上書き可能）
3. `dest` の親ディレクトリが存在しない場合は自動作成
4. シンボリックリンクは**相対パス**で作成（ポータビリティのため）

### 例

```bash
# Instruction の共有
plm link CLAUDE.md .github/copilot-instructions.md

# ターゲット間のスキル共有
plm link .codex/skills/mp/plugin/skill .github/skills/mp/plugin/skill

# 既存ファイルを上書き
plm link --force CLAUDE.md .gemini/GEMINI.md
```

### --force の動作

| 既存の dest | 動作 |
|-------------|------|
| 通常ファイル | 削除して続行 |
| シンボリックリンク | 削除して続行 |
| 空ディレクトリ | 削除して続行 |
| 非空ディレクトリ | エラー（手動削除が必要） |

### 制約

- Unix のみ対応（Windows ではエラー）

## plm unlink

```bash
plm unlink <path>
```

### 引数

| 引数 | 説明 |
|------|------|
| `path` | 削除するシンボリックリンク |

### 動作

1. 指定パスがシンボリックリンクであることを確認
2. シンボリックリンクでなければエラー（通常ファイル/ディレクトリの誤削除を防止）
3. 壊れたシンボリックリンクも削除可能

### 例

```bash
# シンボリックリンクを削除
plm unlink .github/copilot-instructions.md

# 壊れたシンボリックリンクも削除可能
plm unlink broken-link
```
