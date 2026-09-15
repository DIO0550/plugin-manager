# Issue #445 レビュー: 公式 `argument-hint: [message]` のパース不能と round-trip 破壊

| 項目 | 内容 |
|------|------|
| Issue | [#445 [bug] 公式形式 argument-hint: [message]（非クオート）がパース不能、かつ自前シリアライズも再パース不能な形式を出力する](https://github.com/DIO0550/plugin-manager/issues/445) |
| レビュー日 | 2026-09-15 |
| 対象ブランチ | `main`（`cabacbc` — #479 マージ後） |
| ラベル | `bug` |
| Rust 変更 | なし（本 PR はレビュー文書のみ） |

## サマリー

Issue の中核主張は現状コードと一致する。

1. Anthropic 公式例の `argument-hint: [message]` は YAML フローシーケンス `["message"]` になり、`Option<String>` へのデシリアライズが `PlmError::Yaml` で失敗する。
2. `to_markdown()` は `escape_yaml_string` 経由で同じ非クオート形式を出力し、`command_test.rs` がその誤りを固定している。**自分で書いた Claude Code コマンドを自分で再パースできない。**

一方、修正案の「sequence / scalar 両対応の custom deserializer（または `Option<Vec<String>>`）」は **`[message]` 単体には効くが、公式ドキュメントが同時に示す複数引数形式を救えない。** 公式は `argument-hint: [filename] [format]` や `argument-hint: [arg1] [arg2]` を例示しており、これらは YAML として壊れるため deserializer に到達しない。

コードベースは既にこの不正 YAML を Skill の行単位 strip で認識している（`file-formats.md` / `strip_skill_frontmatter_fields`）。Command パーサーだけが serde の型に引きずられている。

**結論: バグは妥当。方針は承認するが、主案を「行単位プリプロセス + `Option<String>` 維持」に差し替える。`escape_yaml_string` の `[` クオートは本 Issue に含める（独立 Issue は未起票）。**

## 問題の再現性（現状コード）

### 入力側

| 項目 | 根拠 |
|------|------|
| 型 | `ClaudeCodeCommandFrontmatter.argument_hint: Option<String>`（`#[serde(default)]`） |
| パース経路 | `ClaudeCodeCommand::parse` → `deserialize_frontmatter` → `serde_yaml::from_str` → 失敗時 `PlmError::Yaml` |
| 既存テストが使う入力 | **クオート済み** `argument-hint: "[message]"` / `"[topic]"` のみ |
| 仕様書の例 | **非クオート** `argument-hint: [message]`（`docs/architecture/file-formats.md`） |

公式ドキュメント（[Slash Commands / Skills](https://code.claude.com/docs/en/slash-commands)）の `argument-hint` は string 扱いで、例は `[issue-number]` と `[filename] [format]`。プラグイン開発リファレンスも `argument-hint: [arg1] [arg2]` を正規例にする。

YAML としての実測（PyYAML。`serde_yaml` 0.9 も YAML 1.1 フローシーケンスとして同じ分類になる）:

| 入力 | YAML としての意味 | `Option<String>` |
|------|-------------------|------------------|
| `argument-hint: "[message]"` | 文字列 `"[message]"` | 成功（現行テスト） |
| `argument-hint: [message]` | シーケンス `["message"]` | **失敗** |
| `argument-hint: [commit message]` | シーケンス `["commit message"]`（1 要素。空白はスカラー内） | **失敗** |
| `argument-hint: [filename] [format]` | 不正 YAML（隣接フローシーケンス） | **失敗（deserializer 未到達）** |
| `argument-hint: [arg1] [arg2]` | 同上 | **失敗（deserializer 未到達）** |
| `argument-hint: message` | 文字列 | 成功 |

### 出力側（round-trip 破壊）

```133:135:src/parser/claude_code/command.rs
        if let Some(ref v) = self.argument_hint {
            fields.push(format!("argument-hint: {}", convert::escape_yaml_string(v)));
        }
```

`escape_yaml_string` のクオート条件は `:` / `"` / `#` / 改行 / 先頭末尾空白のみ。`[` は対象外。

```345:352:src/parser/convert.rs
pub fn escape_yaml_string(s: &str) -> String {
    let needs_quote = s.contains(':')
        || s.contains('"')
        || s.contains('#')
        || s.contains('\n')
        || s.starts_with(' ')
        || s.ends_with(' ');
```

内部値が `"[message]"` のとき出力は `argument-hint: [message]` になる。これを再パースするとシーケンスになり失敗する。

固定化しているテスト:

```263:263:src/parser/claude_code/command_test.rs
    assert!(md.contains("argument-hint: [message]")); // Brackets don't need quotes
```

コメントは YAML として誤り。ブロックスカラーの plain scalar は `[` で始まるとフローシーケンスになる。

### 影響範囲（Command 配置）

`ClaudeCodeCommand::parse` は同一形式コピーでは呼ばれない。失敗するのは **Claude Code → Copilot / Codex の変換** だけ。

| 配置先 | `command_format()` | 経路 | `#445` で install 失敗するか |
|--------|-------------------|------|------------------------------|
| Cursor | `ClaudeCode` | 同一形式コピー | しない |
| OpenCode | `ClaudeCode` | 同一形式コピー | しない |
| Copilot | `Copilot` | `parse` → 変換 | **する** |
| Codex | `Codex` | `parse` → 変換 | **する**（hint 自体は捨てるが、parse 失敗で配置全体が落ちる） |

呼び出し: `convert_content`（`src/component/convert.rs`）が `ClaudeCodeCommand::parse` する。`install` はターゲットの `command_format()` で `ConversionConfig::Command` を組む。

変換テストのサンプル（`sample_claude_code_content`）は `argument-hint` を含まないため、この回帰を拾っていない。

### Skill 経路は別問題（本 Issue の必須範囲外）

Skill は `ClaudeCodeCommand` を通らない。Codex / Gemini CLI 向けは `strip_skill_frontmatter_fields` が **行単位** で `argument-hint` を落とす。動機は本 Issue と同じ不正 YAML である。

```307:310:src/component/convert.rs
/// frontmatter 全体を YAML として再パースせず、**行ベース**で top-level キーを判定して
/// 除去する。これは、サポート外フィールドが不正な YAML 値を持つ場合（例:
/// `argument-hint: [threshold] [min-lines]` はフローシーケンスとして解釈され壊れる）でも、
/// 該当行を安全に取り除けるようにするためである。
```

Cursor / Copilot / OpenCode の Skill は frontmatter をそのままコピーする。Command の Copilot/Codex 変換だけが serde で公式形式に衝突する。

## 設計判断のレビュー

### 1. ドメイン型は `Option<String>` を維持する

#### 結論: **承認（`Option<Vec<String>>` は不採用）**

公式の型は String（autocomplete 用の表示ヒント）。複数引数も **1 本の表示文字列** `[filename] [format]` であり、YAML 配列 `["filename", "format"]` ではない。

`Vec<String>` にすると:

- `[message]` → `["message"]`（ブラケットが消える）
- `[commit message]` → `["commit message"]`（1 要素。再構成でブラケットを足す必要がある）
- `[filename] [format]` → YAML 不正のため配列化できない
- Copilot 変換・`to_markdown`・仕様書の string 契約と不一致

内部正は **元の表示文字列**（ブラケット含む）のままにする。

### 2. 入力: sequence/scalar deserializer だけでは不足

#### 結論: **Issue 案は `[message]` 限定なら正しい。公式全体を満たす主案にはしない。**

| 案 | `[message]` | `"[message]"` | `[filename] [format]` | 評価 |
|----|-------------|---------------|----------------------|------|
| A. string または seq の deserializer | ○（再構成が必要） | ○ | ×（YAML 不正） | 公式複数引数が残る |
| B. 行単位プリプロセスで値をクオートしてから serde | ○ | ○（二重クオートしない） | ○ | **推奨** |
| C. serde を使わず当該行を生テキスト抽出 | ○ | ○ | ○ | B と同等。既存 `extract_frontmatter` と相性が良い |

**推奨は B**（必要なら C）。既存の `FrontmatterSchema::normalize_description_examples` と同じ「公式が YAML として壊れている箇所を、serde の前に直す」パターン。Command スキーマは既に `argument_hint: true` を持っているので、正規化をここに足すのが最短。

プリプロセスのルール（実装時にテストで固定する）:

- `argument-hint:` の同一行の値が、未クオート（`"` / `'` で始まらない）なら、値全体をダブルクオートし内部の `"` `\` をエスケープする。
- 既にクオート済み、または空なら触らない。
- ブロックスカラー（`|` / `>`）や次行インデントの複数行値は、公式例に無い。MVP では同一行のみでよい。触らないか、既存の description 正規化と同様に境界を明示する。

これで deserializer 追加は不要になる。A を足しても B の後では常に String なので冗長。

値の正規化結果:

| 入力 | 内部値 |
|------|--------|
| `[message]` | `"[message]"` |
| `"[message]"` | `"[message]"` |
| `[filename] [format]` | `"[filename] [format]"` |
| `message` | `"message"` |

Copilot 変換は現状 `trim_start_matches('[').trim_end_matches(']')` なので、単一ブラケット対の公式例は従来どおり `Enter message` になる。

### 3. 出力: `escape_yaml_string` で `[` をクオートする（本 Issue に含める）

#### 結論: **承認。独立 Issue は不要。**

Issue 本文の「別 Issue」はリポジトリ上見つからない。round-trip を閉じる変更なので本 Issue に含める。

最低限の追加条件:

- 値が `[` / `]` / `{` / `}` を含む（フローコレクション開始）
- 可能なら YAML plain scalar の他の禁止文字（`,` `&` `*` 行頭 `-` など）もまとめて直す。ただし **本 Issue の受け入れは `[` を含む値の round-trip** で足りる。広げすぎない。

出力例: `argument-hint: "[message]"`。

公式の非クオート表記を **再出力で再現する必要はない。** 入力で受理し、出力は妥当な YAML にする。Claude Code はクオート済み string を読む（現行テストがその経路）。

`escape_yaml_string` は Agent / Copilot / Codex の他フィールドからも使われる。`[` で始まる name/description は稀だが、クオートは安全側。既存テスト（colon / hash / newline）は維持。`[` 用のテストを `convert_test.rs` に追加する。

### 4. Copilot hint 変換の複数ブラケット対（フォローアップ）

#### 結論: **#445 の必須範囲外。parse 修正後に顕在化するので Issue に注記する。**

```174:178:src/parser/claude_code/command.rs
        // Hint conversion: [message] -> "Enter message"
        let hint = self.argument_hint.as_ref().map(|h| {
            let inner = h.trim_start_matches('[').trim_end_matches(']');
            format!("Enter {}", inner)
        });
```

`[filename] [format]` を文字列として受理したあと、この trim は `Enter filename] [format` になる（先頭の `[` と末尾の `]` だけ剥がす）。仕様は単一の `[message]` → `"Enter message"` のみ定義している。

MVP: 単一ブラケット対は現状維持。複数対は生文字列を `Enter ` なしで渡す、または `[]` を空白に正規化する、は別判断。parse 失敗を直す方が先。

## 仕様ドキュメント

`docs/architecture/file-formats.md` は型を string としつつ例は非クオート。実装修正後に短い注記を足す:

- 公式例は非クオート。YAML ではシーケンスまたは不正値になる。
- PLM は表示文字列として受理する。
- 再出力はクオートする（`argument-hint: "[message]"`）。
- Skill の行単位 strip 説明（`[threshold] [min-lines]`）はそのまま有効。Command は受理してから変換する、という対比を一文で書く。

## 推奨テスト

既存のクオート済み parse は残す。誤りを固定している出力アサーションだけ直す。

| ID | 対象 | 入力 / 操作 | 期待 |
|----|------|-------------|------|
| T1 | parse | 非クオート `[message]` | `Some("[message]")` |
| T2 | parse | クオート `"[message]"` | 現状どおり（回帰） |
| T3 | parse | 非クオート `[filename] [format]` | `Some("[filename] [format]")` |
| T4 | parse | 非クオート `[commit message]` | `Some("[commit message]")` |
| T5 | to_markdown | 内部値 `"[message]"` | `argument-hint: "[message]"`（クオート必須） |
| T6 | round-trip | T1 の結果を `to_markdown` → 再 parse | 値が一致。`PlmError::Yaml` にならない |
| T7 | Copilot 変換 | 非クオート `[message]` のファイル | `hint: Enter message`（または現行の hint 行） |
| T8 | Codex 変換 | 非クオート `[message]` のファイル | parse 成功。出力に `argument-hint` なし |
| T9 | `escape_yaml_string("[message]")` | — | `"\"[message]\""` |

T3 は deserializer だけの実装だと落ちる。受け入れに含めることで主案 B を強制できる。

T7/T8 は `sample_claude_code_content` に hint を足すか、専用フィクスチャにする。現状サンプルに hint が無いので convert 層の回帰が空いている。

## 実装手順（提案）

1. **Red**: T1 / T3 / T5 / T6 を追加。現行は T1/T3/T6 が失敗、T5 は「非クオートを期待」しているので期待値を先に直すと Red になる。
2. **Green**: `normalize_description_examples` の前後で `argument-hint` 行をクオート。`escape_yaml_string` に `[` `]` `{` `}` を追加。
3. **Refactor**: 正規化を schema の責務としてテスト可能な関数に切り出す。ドメイン型は触らない。
4. `file-formats.md` に受理/再出力の注記を追加。

## 判定

| 項目 | 判定 |
|------|------|
| バグの存在 | **確認**。公式非クオート単一値は sequence、複数ブラケット対は不正 YAML |
| 影響 | Copilot / Codex への Command 変換。Cursor / OpenCode のコピー経路は対象外 |
| Issue 修正案（deserializer / Vec） | `[message]` には不足なく、公式複数引数には **不足** |
| 推奨修正 | 行単位プリプロセス + `Option<String>` 維持 + `escape_yaml_string` のフローインジケータクオート |
| 誤り固定テスト | `to_markdown_full_command` の Brackets コメントを破棄 |
| 独立の escape Issue | 不要。本 Issue に含める |
| Copilot 複数引数 hint 変換 | フォローアップ。parse 修正のブロッカーではない |

**実装に入ってよい。deserializer のみで閉じないこと。T3（`[filename] [format]`）を受け入れに含めること。**
