# Issue #340 parser ヘルパー共通化の設計レビュー

## 概要

Issue #340 は、`src/parser/` 配下の 6 パーサー（codex/copilot/claude_code × agent/prompt(command)）に散在する同一ヘルパーを共通化する提案である。

本レビューは現行実装（`main` 時点）を突き合わせ、重複の真偽・差分・共通化境界・却下案・段階移行を整理する。今回の成果物は設計レビュー文書のみとし、Rust 実装は行わない。

関連:

- Issue #340
- Issue #337（フォーマット系 enum 統一）
- `docs/architecture/file-formats.md`
- `docs/architecture/module-substructure.md`

## 結論

- Issue の重複指摘は概ね正しい。`extract_name_from_path` ×6、`normalize_name` ×4、frontmatter envelope ×6、YAML 配列直列化 ×2 は実在する。
- **今すぐ共通化すべきは小型ヘルパーのみ**である。
  - `stem_without_suffixes(path, suffixes) -> Option<String>`
  - `normalize_optional_name(name) -> Option<String>`（命名は後述）
  - `emit_frontmatter(fields, body) -> String`
  - `yaml_single_quoted_array(items) -> String`
- `trait ParsedDocumentFile` による `parse()` / `load()` 畳み込みは **見送る**。型ごとのフィールド写像と `load` の名前解決意味が揃っていない。
- frontmatter 出力を一律 `serde_yaml` 直列化へ寄せる案も **現時点では見送る**。現行は手書き YAML（単一引用配列・handoffs インデント・空コレクション省略）をテストで固定しており、バイト一致を崩しやすい。
- 共通化先は `parser` 内部モジュール（候補: `frontmatter.rs` 拡張、または `parser/name.rs` + `parser/emit.rs`）に閉じる。`marketplace::normalize_name` と名前衝突させない。

## 現状突き合わせ

### 1. `extract_name_from_path`（×6・指摘どおり）

骨格は 6 箇所とも同一で、差分はサフィックスだけである。

| ファイル | サフィックス順 |
|---|---|
| `codex/agent.rs` | `.agent.md` → `.md` |
| `copilot/agent.rs` | `.agent.md` → `.md` |
| `claude_code/agent.rs` | `.md` |
| `codex/prompt.rs` | `.md` |
| `copilot/prompt.rs` | `.prompt.md` → `.md` |
| `claude_code/command.rs` | `.md` |

`stem_without_suffixes(path, &[".agent.md", ".md"])` のような共通 API で置換できる。長いサフィックスを先に試す現行順は維持必須。

### 2. `normalize_name`（×4・指摘どおり）

次の 4 ファイルでバイト単位同一:

- `copilot/agent.rs`
- `copilot/prompt.rs`
- `claude_code/agent.rs`
- `claude_code/command.rs`

Codex 側に無いのは正しい（frontmatter に `name` が無く、常にファイル名から取るため）。

注意: クレート内には既に `marketplace::normalize_name(&str) -> Result<String, String>` がある（小文字化・文字種検証）。parser 側の trim/empty→None とは別物。共通化する際は `normalize_optional_name` など別名にし、`pub use` で外へ出さない。

### 3. frontmatter envelope（×6・指摘どおり）

6 箇所すべてが次の形:

```text
fields が空 → body のみ
fields が非空 → "---\n{fields}\n---\n\n{body}"
```

フィールド組み立て自体は型ごとに異なるが、envelope 部分だけは `emit_frontmatter(&fields, &body)` に切り出せる。

### 4. YAML 配列直列化（×2・指摘どおり、ただし空配列挙動に差分）

`copilot/agent.rs` と `copilot/prompt.rs` で:

```rust
.map(|t| format!("'{}'", t.replace('\'', "''"))).join(", ")
```

が逐語重複。ここは `yaml_single_quoted_array` に畳める。

ただし空 `tools` の扱いが揃っていない:

| 実装 | `Some(vec![])` の出力 |
|---|---|
| `CopilotAgent::to_markdown` | `tools:` を省略（`!tools.is_empty()`） |
| `CopilotPrompt::to_markdown` | `tools: []` を出力し得る |

共通化時に偶然揃えてしまわないこと。挙動を意図的に統一する場合は別 Issue / テスト更新が必要。

### 5. `parse()` / `load()` ボイラープレート（部分的に正しい）

共通骨格はあるが、意味差分がある。

| 種別 | `parse` の name | `load` の name |
|---|---|---|
| Codex agent/prompt | 常に `None` | **常に** path 由来で上書き |
| Copilot / Claude Code | `normalize_name(fm.name)` | path 由来は **name が None のときだけ** fallback |

このため「デフォルト `load()` を 1 つに畳む」trait は、Codex の上書き仕様を隠すか分岐を増やすかのどちらかになり、削減効果が薄い。`from_frontmatter` もフィールド集合が型ごとに違い、generic 化の勝ちが小さい。

Claude Code 側はさらに `to_format` / `to_copilot` / `to_codex` を同一ファイルに持ち、`TargetFormat` 実装箇所も Codex/Copilot と揃っていない（Claude は inherent `to_markdown`）。ここまで含めた trait 化は #337 のフォーマット統一と同時に再検討する方がよい。

## 推奨 API 候補

配置候補 A（小さく始める）: `src/parser/frontmatter.rs` に writer / name helper を同居させる。

配置候補 B（責務分離）:

```text
src/parser/
├── frontmatter.rs      # parse 既存 + emit_frontmatter
├── name.rs             # stem_without_suffixes, normalize_optional_name
└── ...
```

モジュール数が増える場合は `module-substructure.md` の起動基準（本体 4+ かつ 2+ グループ）を満たすか確認する。現状は A で十分。

```rust
use std::path::Path;

/// ファイル名から、与えたサフィックスを長い順に剥がした stem を返す。
pub(crate) fn stem_without_suffixes(path: &Path, suffixes: &[&str]) -> Option<String> {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| {
            suffixes
                .iter()
                .find_map(|suffix| s.strip_suffix(suffix))
                .unwrap_or(s)
                .to_string()
        })
        .filter(|s| !s.is_empty())
}

/// frontmatter 由来の name を trim し、空なら None。
pub(crate) fn normalize_optional_name(name: Option<String>) -> Option<String> {
    name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// YAML frontmatter envelope を組み立てる。fields が空なら body のみ。
pub(crate) fn emit_frontmatter(fields: &[String], body: &str) -> String {
    if fields.is_empty() {
        body.to_string()
    } else {
        format!("---\n{}\n---\n\n{}", fields.join("\n"), body)
    }
}

/// Copilot 系 tools 配列の単一引用 YAML 断片。
pub(crate) fn yaml_single_quoted_array(items: &[String]) -> String {
    let arr = items
        .iter()
        .map(|t| format!("'{}'", t.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{}]", arr)
}
```

各パーサー側の置換イメージ:

```rust
// copilot/agent.rs
agent.name = stem_without_suffixes(path, &[".agent.md", ".md"]);

// to_markdown 末尾
emit_frontmatter(&fields, &self.body)
```

フィールド push（`name:` / `handoffs:` など）は各型に残す。

## 所有関係

| 所有者 | 責務 | 方針 |
|---|---|---|
| `parser/frontmatter.rs`（または name/emit） | 名前抽出・正規化・envelope・配列断片 | crate 内 `pub(crate)` |
| 各 `*Agent` / `*Prompt` / `*Command` | FM 型、フィールド写像、型固有直列化 | Feature 内に維持 |
| `parser/convert.rs` | `escape_yaml_string`・tool/model 変換・`TargetFormat` | 既存のまま。配列 helper は frontmatter 側へ寄せる |
| `marketplace::normalize_name` | marketplace ID 検証 | 触らない・別名衝突を避ける |

## 却下案

### `trait ParsedDocumentFile { type Fm; fn from_frontmatter(...); }` + デフォルト `load()`

- Codex とそれ以外で `load` の name 解決意味が違う。
- `from_frontmatter` の写像は型固有で、共通化しても 6 個の写像関数が残る。
- Claude Code の conversion API まで含めると trait 境界が肥大化する。
- #337 で `Format` / `TargetType` を整理したあとに、変換パイプライン側から再検討する。

### `*Frontmatter` を `serde_yaml::to_string` で直列化

- 現行出力は `escape_yaml_string`・単一引用配列・handoffs の手書きインデントに依存。
- `serde_yaml` のキー順・引用スタイル・`None` 省略は現行 golden と一致しない可能性が高い。
- パース側は既に serde、出力側だけ手書き、という非対称は実害よりテスト安定性の方が大きい。出力を serde に寄せるなら、先に全 `to_markdown_*` テストをスナップショット化し差分許可方針を決める。

### helper を `convert.rs` に寄せる

- `convert` は tool/model/body 変換と `TargetFormat` の場所。path stem / frontmatter envelope は変換ではない。
- `escape_yaml_string` は既にここにあるが、配列断片をさらに足すと「変換」と「文書組み立て」が混ざる。envelope/array は frontmatter 側が自然。

## 将来移行手順

1. 既存の `*_test.rs`（特に `to_markdown_*` / `extract_name_*` / load fallback）を現状固定の回帰として確認する。
2. `frontmatter.rs` に 4 helper を追加し、単体テストを `frontmatter_test.rs` に置く。
3. 6 パーサーの `extract_name_from_path` / 4 パーサーの `normalize_name` を置換し、ローカル定義を削除する。
4. 6 箇所の envelope を `emit_frontmatter` に置換する。
5. Copilot agent/prompt の配列直列化を `yaml_single_quoted_array` に置換する。**空 tools 挙動は現状維持**。
6. `cargo test` で parser 配下を通す。出力差分が出たら helper ではなく呼び出し側の条件分岐を見直す。
7. （任意・別 PR）空 `tools` の agent/prompt 非対称を仕様として揃えるかを決める。
8. （将来）#337 完了後に `parse`/`load` のさらなる抽象化を再評価する。

## テスト方針

今回は設計レビューのみで自動テスト対象はない。実装 PR では次を最低限にする。

- `stem_without_suffixes`: `.agent.md` / `.prompt.md` / `.md` の優先順、空ファイル名、非 UTF-8 ファイル名相当（失敗して None）。
- `normalize_optional_name`: `None` / `""` / `"  "` / `" name "`。
- `emit_frontmatter`: 空 fields / 複数 fields / body のみ。
- `yaml_single_quoted_array`: 空配列、単一引用エスケープ（既存 `to_markdown_single_quote_in_tools` 相当）。
- 既存 6 パーサーの `to_markdown_*`・`load` テストが無変更で通ること。

## #337 / #96 との関係

- #337（Format enum 統一）は変換ディスパッチの型安全が主眼。本 Issue の小型 helper 共通化とは直交し、**先に #340 の Phase 1 を入れても衝突しにくい**。
- #96（Claude Code ターゲット追加）でパーサーが増える前に helper を置く、という Issue 動機は妥当。trait 化まで急ぐ必要はない。

## 手動検証

- 6 ファイルの `extract_name_from_path` 骨格一致とサフィックス差分を確認した。
- 4 ファイルの `normalize_name` がバイト単位同一であること、Codex に無い理由を確認した。
- envelope パターンが 6 箇所同一であることを確認した。
- Copilot agent/prompt の空 `tools` 出力差分を確認した。
- Codex の `load` が常時 path 上書き、他は fallback のみであることを確認した。
- `marketplace::normalize_name` との意味差を確認し、別名方針とした。
- 本変更は `docs/` の設計レビューのみで、`src/` は変更しない。
