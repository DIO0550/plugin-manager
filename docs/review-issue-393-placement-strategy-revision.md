# レビュー: Issue #393 Plugin 付属リソース（構造そのまま配置）

レビュー日: 2026-07-29  
正本: [#393](https://github.com/DIO0550/plugin-manager/issues/393) 現行本文  
HEAD: `e605b78`（`main`）  
関連: [#392](https://github.com/DIO0550/plugin-manager/issues/392) / [#339](https://github.com/DIO0550/plugin-manager/issues/339) / [#377](https://github.com/DIO0550/plugin-manager/issues/377) / [#96](https://github.com/DIO0550/plugin-manager/issues/96)

## 判定

Issue #393 現行本文を配置方針の正本とする。

| レイヤ | 状態 |
|--------|------|
| 方針 | **確定** — プラグイン構造を変えずターゲットへ配置。Skill 配下への複製や `_shared/` は行わない |
| 検出（manifest + デフォルト境界） | 方針は妥当。実装は未着手 |
| 配置マッピング（Target × 着地パス） | **仕様未記入** — Issue が「仕様書で明記」と明示。実装前に `file-formats.md` へ書く必要あり |
| ライフサイクル（plugin 単位 + `managedFiles`） | 方針は妥当。実装は未着手 |
| Rust / テスト | **未実装** |
| 旧ドキュメント | `file-formats.md` / concepts の Plugin 付属記述は Issue と不一致のため **本 PR で Issue 方針に書き換え** |

**結論:** 方針は Issue どおりで進める。ブロッカーは「各ターゲットでの着地パス写像」が仕様にまだ無い点。写像を `file-formats.md` に書いたうえで Scan → Deploy → ライフサイクル → 統合テストの順に実装する。

---

## 1. 方針サマリ（正本の再掲）

1. **境界:** `plugin.json` の宣言が真実源。未宣言種別は Claude Code Plugin デフォルト（`skills/` / `agents/` / `commands|prompts/` / `hooks/`）にフォールバック。
2. **付属リソース:** 上記でコンポーネントとして解決されなかったプラグイン直下エントリ（再帰）。`ComponentKind` は 5 種のまま別枠。
3. **配置:** 付属リソースはプラグイン直下の構造を保ったまま配置。コンポーネントは従来どおり各ターゲット規則で変換・配置。Skill 内付属は #392（構造維持のまま同梱）。
4. **衝突:** 構造維持のため着地先が被らない → 衝突解決ルール不要。
5. **ライフサイクル:** install / update（`managedFiles` で差分掃除）/ disable・uninstall（プラグイン単位）/ sync（プラグイン単位）。

除外（付属からも配置からも外す）: VCS 系。  
コンポーネント扱い（付属から除外）: `.claude-plugin/`、manifest 解決パス、デフォルト規約 dir、Instruction ファイル。

---

## 2. 検出設計のレビュー

| 項目 | 評価 |
|------|------|
| manifest 宣言を境界の真実源にする | ✅ カスタム `skills` / `hooks` パスの誤同梱を防げる |
| 未宣言時のデフォルト規約フォールバック | ✅ Claude Code Plugin と整合 |
| `ComponentKind` を増やさない | ✅ Feature 境界が明確 |
| リテラル固定ではなく解決後パスで除外 | ✅ #339 と整合。実装時は `PluginManifest::*_dir` 結果を除外集合に入れる |
| VCS 除外 | ✅ |
| Scan を既存 `list_*` と並列に追加 | ✅ `build_components` に混ぜない方がよい |

### 実装時の注意（コード根拠）

- 現状 `Plugin::build_components`（`src/plugin/content/plugin_content.rs`）は 5 種のみ。付属列挙 API は `src/scan/` に無い。
- `hooks` がファイル（`./hooks/hooks.json`）のとき、除外対象はファイル自体か親 `hooks/` か。**親ディレクトリを除外しないと `hooks/` 内の他ファイルが付属に混ざる**。仕様に「宣言パスがファイルなら、その親コンポーネントディレクトリも除外」と明記することを推奨。
- デフォルトに `prompts/` を含める記述は Copilot 配置名と混同しやすい。ソース側のデフォルトは `commands/`（`ComponentKind::default_subdir`）。`prompts/` はターゲット側配置名（`placement_names::COPILOT_COMMAND_SUBDIR`）。検出除外のデフォルトはソース規約に揃える。

---

## 3. 配置設計のレビュー（最大の未決）

Issue の受け入れに必須だが、本文は写像を仕様書へ委譲している。

> 各ターゲットの配置ルート内でプラグイン構造をどうマッピングするか（既存の Skill 配置パスとの整合）は仕様書で明記する（#377 の Cursor ディレクトリ名との整合を含む）。

### 現行配置（実装）

| Target | Skill | Agent |
|--------|-------|-------|
| Codex 等 | `<base>/skills/<plugin>_<skill>/` | `<base>/agents/<plugin>_<agent>.agent.md` 等 |
| Cursor | `<base>/skills/<original_name>/`（#377） | `<base>/agents/<flattened>.md` |

プラグインソース上の兄弟（`skills/` と `references/`）はターゲット上に存在しない。

### 仕様で必ず決めること

1. **付属リソースのルート**  
   各 Target × Scope で、プラグイン直下の `references/` がどの絶対パスに着地するか。
2. **「リポジトリと同じ相対関係」の定義**  
   コンポーネントは従来どおりフラット配置、付属だけ構造維持、のとき Skill/Agent ファイルから見た相対パスはソースと一致しない。  
   - 相対関係を文字どおり保つなら、付属だけでなく **プラグイン木全体（または参照が辿れる部分木）の着地**が必要  
   - あるいは「ホストがプラグインルートを基準に解決する」前提を仕様に書く  
   どちらにするかを `file-formats.md` に明示する。
3. **#377（Cursor `original_name`）**  
   Skill ディレクトリ名がプラグイン内名のままでも、付属が別ツリーだと `../references` は届かない。写像とセットで検証手順を書く。
4. **Claude Code（#96）**  
   受け入れ対象に含まれるが #96 未実装。本 Issue の DoD に含めるか、#96 後追いとするかを Issue / 仕様に書く。
5. **Skill を持たないプラグイン**  
   付属のみのプラグインでも配置するか（Issue は Skill 対応ターゲット向けと読むが、所有権はプラグイン単位）。

`file-formats.md` の Plugin 付属節は本 PR で Issue 方針に合わせて書き換え、上記 1–5 を「仕様で確定が必要な項目」として残す。

---

## 4. ライフサイクルのレビュー

| 操作 | Issue | 現行コードとのギャップ |
|------|-------|------------------------|
| install | 配置 | 付属配置パス無し |
| update | `managedFiles` 登録 + 削除分掃除 | `managedFiles` は共有 destination（hooks / Cursor skill）用途。付属用の登録・GC が必要 |
| disable / uninstall | プラグイン単位で除去 | 現状はコンポーネント単位の RemoveDir が中心。付属ルートの削除を追加 |
| sync | プラグイン単位で差分 | 現状はコンポーネント単位。付属を単位に含める設計が必要 |
| enable | （Issue に明示薄い） | install と別経路（`intent` の CopyDir）。付属の再配置も必要 |

`managedFiles` 再導入は、付属が Skill dir 外に出る本方針と整合する。登録粒度（ファイル単位 / 付属ルートディレクトリ単位）を仕様で決める。

---

## 5. 受け入れ条件の評価

| 条件 | 評価 |
|------|------|
| 境界が manifest + デフォルト | ✅ |
| Scan が付属を列挙（判定を仕様書へ） | ✅。hooks ファイル宣言時の親 dir 扱いを追記推奨 |
| Deploy が構造を保ったまま | ✅。**前提として着地写像の仕様化が必要** |
| 全ライフサイクル一貫 | ✅。enable も明示した方がよい |
| `file-formats.md` 追記 | ✅（本 PR で方針節を Issue 準拠へ置換。写像は未決枠を残す） |
| 統合テスト: `references/tdd-guidelines.md` が構造どおり | ✅。着地パスが決まってからパスを固定 |
| 統合テスト: Skill 内 `scripts/` も構造維持 | ✅。#392 と重複しうるが回帰として妥当 |
| 統合テスト: カスタムパスは付属に含めない | ✅ |

---

## 6. 実装ギャップ（`main`）

| 箇所 | 現状 |
|------|------|
| `src/scan/` | plugin-root 非 Component 列挙 API 無し |
| `Plugin::build_components` | 5 種のみ |
| `deploy_skill` / place | Skill ソース `replace_dir` のみ。Plugin 直下を見ない |
| `intent` enable/disable | Skill/Component 単位。付属ルート無し |
| `placement_names.rs` | 付属除外定数未追加 |
| `managedFiles` | 付属用の登録・掃除無し |
| テスト | Skill **内**付属（#392）のみ |

---

## 7. 推奨する作業順

1. **仕様:** `file-formats.md` にターゲット別着地写像を確定記入（本レビューの「仕様で必ず決めること」）。concepts を追随。
2. **検出:** `scan` に付属列挙 + manifest/デフォルト/VCS 除外（#339 の定数集約）。
3. **配置:** 写像に従い install / enable の両方へ配置。
4. **所有権:** `managedFiles`（または同等）で update / uninstall / disable / sync。
5. **テスト:** カスタムパス除外 + spec-plugin 相当レイアウトで全 Skill 対応 Target。

Rust 実装は本レビューでは行わない。

---

## 参照

| パス | 役割 |
|------|------|
| Issue #393 | 正本 |
| `docs/architecture/file-formats.md` | Plugin 付属リソース仕様（本 PR で Issue 準拠へ更新） |
| `src/plugin/content/plugin_content.rs` | `build_components` |
| `src/component/deployment.rs` | `deploy_skill` |
| `src/plugin/lifecycle/intent.rs` | enable/disable |
| `src/plugin/meta/meta.rs` | `managedFiles` |
| `src/target/env/cursor.rs` | Skill `original_name`（#377） |
| `src/placement_names.rs` | 除外リテラルの予定真実源 |
