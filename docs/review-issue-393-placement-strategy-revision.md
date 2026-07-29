# レビュー: Issue #393 配置方針の再定義（案 A 破棄 → 構造そのまま）

レビュー日: 2026-07-29  
対象: [#393](https://github.com/DIO0550/plugin-manager/issues/393)（OPEN・本文更新済み）  
HEAD: `e605b78`（`main`）  
関連: [#407](https://github.com/DIO0550/plugin-manager/pull/407)（MERGED・案 A 仕様） / [#410](https://github.com/DIO0550/plugin-manager/pull/410)（CLOSED・案 A 実装） / [#392](https://github.com/DIO0550/plugin-manager/issues/392) / [#377](https://github.com/DIO0550/plugin-manager/issues/377) / [#339](https://github.com/DIO0550/plugin-manager/issues/339) / [#96](https://github.com/DIO0550/plugin-manager/issues/96)

## 判定（結論）

**現状のままでは実装に着手できない。** Issue 本文とリポジトリ上の確定仕様が配置方針で正面衝突している。

| 観点 | 状態 |
|------|------|
| Issue 本文（2026-07-29 更新） | **構造そのまま配置**。旧案 A（各 Skill へ複製）・旧案 B（`_shared/`）は明示的に不採用 |
| リポジトリ仕様（#407 / `file-formats.md`） | **案 A（各 Skill 配下へ複製）** が正本のまま |
| concepts | 案 A 記述（`components.md` / `deployment.md`） |
| Rust 実装 | **未着手**（案 A 実装 PR #410 は merge せず CLOSED） |
| 実プラグインの参照表記 | Skill / Agent とも `references/...`（Skill・Agent 相対）。案 A と整合 |

**推奨:** 方針をどちらか一方に再確定してから実装する。仕様・Issue・concepts が一致するまで実装 PR を出すべきではない。

---

## 1. 何が変わったか

### 1.1 Issue 本文（新方針）

- コンポーネント境界は `plugin.json` 宣言（+ 未宣言時のデフォルト規約）が真実源
- 解決されなかったプラグイン直下エントリは **付属リソース**として列挙
- 配置は **プラグインのディレクトリ構造を変えずターゲットへ置く**
- 旧案 A（Skill 配下へ複製）・旧案 B（`_shared/`）は行わない
- 命名衝突ルールは不要（構造を保つため着地先が被らない）
- ライフサイクルでは `.plm-meta.json` の `managedFiles` 登録を再導入

受け入れ条件も「各ターゲットで**プラグイン構造どおりの位置**に存在する」に書き換えられている。

### 1.2 リポジトリ側の正本（#407・未更新）

`docs/architecture/file-formats.md`「Plugin 付属リソース」:

> **各 Skill の配置ディレクトリへ、プラグインルートからの相対パスを保って複製する。**

衝突は Skill 優先 + 警告。`managedFiles` 個別登録は不要。concepts も同趣旨。

### 1.3 時系列

| 日時 | 出来事 |
|------|--------|
| 2026-07-27 | #407 merge — 案 A を仕様確定 |
| 2026-07-27〜28 | 調査・実装ギャップ docs / 案 A 実装 PR #410 |
| 2026-07-29 | Issue #393 本文が「構造そのまま」に更新（案 A/B 明示破棄） |
| 2026-07-29 | #410（案 A 実装）CLOSED・未 merge |

Issue 更新が仕様ドキュメント更新より先に走っており、**正本が分裂**している。

---

## 2. 新方針のブロッカー（現行配置モデルとの非整合）

### 2.1 フラット配置に「プラグイン構造」が存在しない

現行の Skill 配置:

| Target | 配置パス |
|--------|----------|
| Codex / Copilot / Antigravity / Gemini CLI | `<base>/skills/<plugin>_<skill>/` |
| Cursor（#377） | `<base>/skills/<original_name>/` |

Agent / Command は単一ファイル（例: `<base>/agents/<plugin>_<agent>.md`）。

プラグインソース上の兄弟関係:

```text
plugins/spec-plugin/
├── skills/implementation-plan/SKILL.md
├── agents/spec-planner.md
└── references/tdd-guidelines.md   # Skill / Agent の兄弟
```

は、ターゲット上では **兄弟ディレクトリとして再現されない**。Issue が言う「構造どおり」の着地先が、ターゲット別マッピングなしでは定義できない。

### 2.2 Issue 自身がマッピングを未決のまま受け入れ条件に含めている

> 各ターゲットの配置ルート内でプラグイン構造をどうマッピングするか（…）は仕様書で明記する

受け入れ条件は「プラグイン構造どおりの位置」を要求するが、**その位置の定義が仕様に無い**。実装前に必須の設計成果物が欠けている。

### 2.3 候補マッピングとそれぞれへの反論

| 候補 | 概要 | 問題 |
|------|------|------|
| A. 案 A（現行仕様） | 各 Skill dir へ `references/` を overlay | Issue 本文が明示破棄。Agent 単一ファイルには届かない |
| B. `_shared/` 兄弟 | `<base>/skills/<plugin>_shared/` | Issue が破棄。Cursor `original_name` と flatten で相対パスが一意にならない（#407 の不採用理由） |
| C. プラグイン木ごと配置 | `<base>/skills/<plugin>/{skills,references,...}` | Skill フラット名規約・Cursor #377・既存配置物・scan/sync を破壊する大規模変更 |
| D. 環境ルート直下に plugin tree | `.cursor/plugins/<plugin>/...` 等 | 各ターゲットの公式レイアウト外。認識されない可能性大。Claude Code（#96）以外では意味が薄い |
| E. 参照書き換え | 配置時に相対パスを書き換え | Issue は「相対参照はそのまま」を前提。#407 も書き換えしない |

**新方針を採るなら C か D 系の配置モデル変更が本体**であり、「付属リソースのコピー」だけでは閉じない。

---

## 3. 実データ: 参照表記は案 A 向き

本ワークスペースに配置済みの `spec-plugin` 由来ファイル:

```text
.cursor/skills/spec-plugin_implementation-plan/SKILL.md:99
  詳細はプラグインの `references/tdd-guidelines.md`・`references/test-design-patterns.md` を参照。

.cursor/agents/spec-plugin_spec-planner.md
  詳細は `references/test-design-patterns.md` を参照。
  TDD プロセスの詳細は `references/tdd-guidelines.md` を参照。
```

ポイント:

1. パスは **`references/...`（コンポーネント相対）**。`../../references/...`（プラグインルート相対）ではない。
2. #407 が案 A を採った主根拠（「実プラグインの参照の書き方が案 A と一致」）は **今も有効**。
3. 現状 Skill 配置先に Plugin 直下 `references/` は無い（#393 未実装）。Skill 内の `references/system-diagrams.md` 等だけが存在する。
4. Agent は単一 `.md` のため、案 A でも Agent からはファイルシステム上の相対解決はできない（プロンプト上の論理参照の可能性が高い）。新方針でも Agent 単一ファイル配置を変えない限り同様。

「構造そのまま」が救うのは、**プラグインルート相対で書かれた参照**と、**ネイティブに plugin tree を読む環境（Claude Code 等）**。現行の Codex/Copilot/Cursor フラット配置 + Skill 相対参照という実データとは噛み合わない。

---

## 4. 検出・除外まわり（方針に依存しない部分）

Issue 更新後も妥当で、案 A / 新方針のどちらでも必要な共通要件:

| 項目 | 評価 |
|------|------|
| 境界は manifest 宣言 + デフォルトフォールバック | ✅ 正しい。リテラル `skills/` 比較だけだとカスタムパスを誤同梱する（#407 でも同指摘） |
| `.claude-plugin/` 除外 | ✅ |
| Instruction ファイル除外 | ✅ |
| VCS 除外 | ✅ |
| `ComponentKind` を増やさない | ✅（別枠扱い） |
| カスタムパス宣言の統合テスト | ✅ 受け入れ条件として適切 |

不足・要明確化:

- Issue 新本文は README*/LICENSE* / 総量閾値に触れていない（#407 仕様にはある）。方針再確定時に残すか削るかを決める。
- `placement_names.rs` に Plugin 付属除外定数は **未追加**（docs 先走りのまま）。
- Claude Code（#96）を受け入れ対象に含めているが、#96 自体は未実装。依存関係を Issue に書くべき。

---

## 5. 実装ステータス（再確認・2026-07-29）

`main` 上で案 A / 新方針いずれの実装も無い。

| レイヤ | 結果 |
|--------|------|
| `list_plugin_attached*` / `AttachedResource` / overlay | `src/` に 0 件 |
| `Plugin::build_components` | 5 種 Component のみ |
| `deploy_skill` | Skill ソースの `replace_dir` + SKILL.md strip のみ |
| enable `CopyDir` | Skill dir のみ。Plugin root を見ない |
| `placement_names` | Instruction / 環境ルートのみ。付属除外定数なし |
| 衝突警告チャネル | Hook `ConversionWarning` のみ |
| テスト | Skill **内**付属（#392）のみ |

調査ドキュメント（`docs/architecture/issue-393-*.md`）の「未実装」主張はコードと一致。ただしそれらは **案 A 前提**で書かれている。

---

## 6. 受け入れ条件ドラフトの再評価

| 条件（Issue 新本文） | 評価 |
|----------------------|------|
| 境界判定が manifest + デフォルト | 方針非依存で妥当 |
| Scan が付属を列挙 | 妥当。判定ルールは仕様書必須 |
| Deploy が**構造を保ったまま**配置 | **マッピング未定義のため判定不能**。現行フラット配置と衝突 |
| 全ライフサイクル一貫 + `managedFiles` | 新方針なら必要になり得る。案 A では不要だった。再設計が必要 |
| `file-formats.md` 追記 | **既存節が案 A のため、追記ではなく全面書き換えが必要** |
| 統合テスト（構造どおりの位置） | 位置定義が先。Skill 内 `scripts/` 同梱は #392 で既にカバー近い |
| カスタムパスは付属に含めない | 妥当 |

---

## 7. 推奨アクション

### 必須（実装前）

1. **配置方針を再確定する**（メンテナー判断）
   - **案 A を維持する**なら: Issue 本文を #407 に合わせて戻し、`managedFiles` 要求と「構造そのまま」を削除。実装は #410 相当を再開。
   - **構造そのままを採る**なら: #407 仕様・concepts を破棄/改訂し、ターゲット別の具体パス写像（Cursor #377 含む）・所有権・Agent 扱い・Claude Code(#96) 依存を仕様書に書いてから実装する。これは付属リソース機能ではなく **配置モデル変更**としてスコープを切り直すこと。
2. 正本を一つにする（Issue 本文 ⇔ `file-formats.md` ⇔ concepts）。
3. 方針確定まで実装 PR を merge しない（#410 クローズは妥当）。

### 案 A を維持する場合の実装チェックリスト（再掲）

1. `scan/` に plugin-root 付属列挙（`ComponentKind` に混ぜない）
2. `placement_names` + manifest 解決パスで除外合成
3. `deploy_skill` の `replace_dir` **後**に overlay（Skill 同名優先 + 警告チャネル）
4. enable の `CopyDir` 経路にも同梱
5. 全 Skill 対応 TargetKind + 仮想 spec-plugin 統合テスト
6. sync の stale 非対称は本 Issue か別 Issue で明示

### 構造そのままを採る場合に先に書くべき仕様項目

1. 各 Target × Scope での「プラグインルート」相当パス
2. Skill / Agent / Command / Hook の配置がフラット名のままか、plugin tree 内相対に戻すか
3. 既存インストール済みコンポーネントの移行 / 互換
4. Agent・Command（単一ファイル）から付属リソースへの参照をどう解決するか
5. `managedFiles` / uninstall / sync の所有権単位（プラグイン単位）
6. Cursor `original_name`（#377）との整合
7. Claude Code（#96）との前後関係

---

## 8. 既存ドキュメントの扱い

| ファイル | 扱い |
|----------|------|
| `docs/architecture/file-formats.md` Plugin 付属節 | 案 A 正本。方針変更時は書き換え |
| `docs/concepts/{components,deployment}.md` | 案 A。展開パス例は旧 3 階層表記が残っており別途陳腐化 |
| `docs/architecture/issue-393-plugin-root-resources-current-behavior.md` | 実装ギャップ調査として有用。方針前提は案 A |
| `docs/architecture/issue-393-implementation-verification.md` | 同上（未実装検証）。方針前提は案 A |
| 本レビュー | **方針分裂を指摘する最新レビュー** |

Rust ファイルは本レビューでは変更していない。

---

## 参照パス

| パス | 役割 |
|------|------|
| `docs/architecture/file-formats.md` | Plugin 付属リソース仕様（案 A） |
| `src/plugin/content/plugin_content.rs` | `build_components` / flatten |
| `src/component/deployment.rs` | `deploy_skill` |
| `src/plugin/lifecycle/intent.rs` | enable/disable CopyDir/RemoveDir |
| `src/placement_names.rs` | 除外定数の予定真実源（未拡張） |
| `src/target/env/cursor.rs` | Skill を `original_name` 配置（#377） |
| `src/target/placed/placement_helpers.rs` | `skill_dir` / `agent_file` |
