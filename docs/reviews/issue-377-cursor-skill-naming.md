# Issue #377 レビュー: Cursor 配置時のフラット化フォルダ名と SKILL.md frontmatter `name` の不一致

| 項目 | 内容 |
|------|------|
| Issue | [#377 [fix] Cursor 配置時のフラット化フォルダ名と SKILL.md frontmatter `name` の不一致でスキルが認識されない可能性](https://github.com/DIO0550/plugin-manager/issues/377) |
| レビュー日 | 2026-07-19 |
| 対象ブランチ | `main`（`13b85ff` — #378 TLS 修正マージ後） |
| ラベル | `bug` |
| 設計コメント | [Issue comment (設計判断)](https://github.com/DIO0550/plugin-manager/issues/377#issuecomment-5013463769) |

## サマリー

現象・原因の整理はコードと一致しており、**Cursor ターゲットのみ元のスキル名で配置する**方針は妥当。frontmatter をフラット化名に書き換える案は、スキル間相互参照を壊すため不採用で正しい。

一方、実装に入る前に次の横断影響を設計に明示する必要がある。特に **sync の名前キー不一致** と **`list_all_placed` / `is_enabled` フォールバックの前提崩壊** は Issue 本文・設計コメントでは十分に扱われていない。

**結論: 方針承認。実装前に下記ブロッカー相当の論点を Issue / 実装メモへ追記すること。**

## 問題の再現性（現状コード）

| 経路 | 挙動 | 根拠 |
|------|------|------|
| 名前生成 | `flatten_name(plugin, original)` → `"{plugin}_{original}"` | `src/plugin/content/plugin_content.rs` |
| Component 保持 | `Component.name` はフラット化済みのみ。元名は破棄 | 同 `flatten_components` |
| Cursor 配置 | `placement_location` が `context.name()`（= フラット化名）をディレクトリ名に使用 | `src/target/env/cursor.rs` L170–171 |
| Skill 変換 | `skill_allowed_fields(Cursor) = None`（frontmatter 無改変） | `src/component/convert.rs` |
| Cursor 要求 | frontmatter `name` と親フォルダ名の完全一致（小文字・ハイフン） | [Cursor Forum](https://forum.cursor.com/t/skills-visible-in-settings-but-not-appearing-as-slash-commands-in-agent-chat/154434)（Staff コメント） |

結果として `.cursor/skills/spec-plugin_spec-driven-dev/` に `name: spec-driven-dev` の `SKILL.md` が置かれ、スラッシュメニュー非表示の条件を満たしうる。ドキュメントも現状フラット配置を正規としている（`docs/concepts/targets.md` / `docs/concepts/deployment.md`）。

## 設計判断のレビュー

### 1. Cursor のみ元名配置（全ターゲット統一しない）

#### 結論: **承認**

| 観点 | 評価 |
|------|------|
| frontmatter 書き換え回避 | ◎ 相互参照（例: spec-plugin）を壊さない |
| 他ターゲット無変更 | ◎ Codex / Copilot / Antigravity / Gemini CLI は現状問題なし |
| `parse_component_name` | ○ `_` 分解せず不透明識別子として扱うため、ターゲット別命名の構造的障害はない（ただし sync マッチングは別問題 — 後述） |

### 2. `Component` / `ComponentRef` に `original_name`（+ `plugin_name`）を保持

#### 結論: **承認。ただし責務分離を明確化する**

推奨モデル:

```text
Component.name          … フラット化済み識別子（他ターゲット・カタログ・メタ互換の正）
Component.original_name … スキャン時のディレクトリ名（Cursor Skill 配置用）
Component.plugin_name   … Manifest.name（衝突メッセージ・レガシーパス生成用）
```

- `ComponentRef` / `PlacementContext` にも配置解決に必要な最小セットを載せる（少なくとも Skill の Cursor 配置で `original_name` を読めること）
- `From<&Component> for ComponentRef` を更新し、`install` / `PluginIntent` / `import` / sync `resolve_path` が同じコンテキストを渡せるようにする
- **内部正は引き続き `name`（フラット化）**。Cursor のディレクトリ名だけが例外で `original_name` を使う、という不変条件をコメントとテストで固定する

`plugin_name` を Component に冗長保持するか、`origin` / Plugin 側から都度渡すかは実装選択でよい。レガシーフォールバックで `flatten_name(plugin, original)` を再計算するなら、どちらかに必ず届く必要がある。

### 3. 配置名選択を Target trait 側へ寄せる

#### 結論: **承認。API 形は 2 案のうち B を推奨**

| 案 | 内容 | 評価 |
|----|------|------|
| A | Cursor の `placement_location` 内だけで `original_name` を読む | 変更最小。他ターゲットは現状維持 |
| B | `Target::placement_name(&self, ctx) -> &str` を追加し、デフォルトは `ctx.name()`、Cursor Skill のみ `original_name` | 意図が型に現れ、Agents/Commands 追従が容易 |

**推奨は B（または同等のヘルパー）**。デフォルト実装で既存 5 ターゲットを壊さず、Cursor だけ override できる。

注意: `list_placed` が返す名前は **実際のディレクトリ名** になる（元名）。一方 `PluginIntent` / `install` はキャッシュ上の `Component.name`（フラット化）で `placement_location` を呼ぶ。したがって **配置パス解決は必ず `original_name` をコンテキスト経由で渡す経路に統一**し、「list が返す名前」と「キャッシュ Component.name」を混同しないこと。

### 4. 衝突検出（必須・ブロッカー級）

#### 結論: **MVP に含める。確認プロンプトより先にハードエラーでよい**

元名配置ではプレフィックスによる名前空間が消える。

| シナリオ | 期待 |
|----------|------|
| 同名 Skill を持つ別プラグインを Cursor へ追加 install | 配置前にエラー（既存ディレクトリが自プラグイン管理外なら拒否） |
| 同一プラグインの再 install / update | 自管理パスへの上書きは許可 |
| Personal と Project の同名 | スコープが別なので衝突しない |

既存の Hook 向け `hook_overwrite_error` + `managedFiles` と同型のガードを Skill ディレクトリに適用するのが最短。対話プロンプトは CLI 非対話前提（CI / Cloud）と相性が悪いため、**第一弾はエラー + メッセージで十分**。

プラグイン内衝突は現状 `detect_name_collisions` がフラット化名で検出している。元名同士の衝突はスキャン段階で既に同一 basename になりうる（ネスト skills の重複 basename）ため、既存 Validation の延長でカバーされるか要確認。

### 5. レガシー掃除（必須・ブロッカー級）

#### 結論: **uninstall / disable / update の削除経路で旧パスをフォールバック削除する**

| パス | 新 | 旧（フォールバック） |
|------|----|----------------------|
| Cursor Skill | `.cursor/skills/<original>/` | `.cursor/skills/<plugin>_<original>/` |

- 新 `placement_location` だけだと旧フォルダが孤児化する
- `PluginIntent::build_file_operation(RemoveDir)` は単一パス前提のため、Cursor Skill 削除時は **新パス + 旧フラット化パスの両方を RemoveDir 候補にする**（存在するものだけ削除）か、削除後に明示的な legacy sweeper を呼ぶ
- update（再配置）では旧パス削除 → 新パス配置の順を保証する

Personal（`~/.cursor/skills/`）も同様。なお現行 `PluginIntent::create_operation` は `Scope::Project` 固定であり、Personal 配置の disable 経路は既存制限がある。本 Issue で Personal まで直すかは任意だが、**Project のレガシー掃除は必須**。

### 6. Skills のみ先に直す

#### 結論: **承認（MVP 範囲として妥当）**

Agents / Commands はファイル名が識別子で、frontmatter `name` 一致制約が Skills ほど明確ではない。効果確認後の追従でよい。ただし追従時は同じ `placement_name` フックを再利用できる形にしておく（上記 API 案 B）。

## Issue 設計で不足している横断影響

### A. sync の `PlacedRef` マッチング（要追記）

`sync` は source / dest を `(kind, name, scope)` で突合する。`list_placed` が返す `name` がターゲット間で異なると **同一スキルが別物扱い** になる。

| Source | Dest | Skill の name キー | 結果 |
|--------|------|-------------------|------|
| Codex | Copilot | 両方 `plugin_skill` | 一致（現状） |
| Codex | Cursor（修正後） | `plugin_skill` vs `skill` | **不一致** |
| Cursor | Codex | 同上 | **不一致** |

#### 推奨

1. **本 Issue の受入から「Cursor を含む sync」を明示的に除外または制限する**（ドキュメントとテストで固定）
2. または sync キーをフラット化名に正規化するレイヤを入れる（Cursor `list_placed` は元名を返すが、sync 時だけ `plugin_name` を復元して突き合わせる — 配置物から plugin を復元できない現状では困難）

現実的には **(1) を本 Issue のスコープとし、sync 正規化はフォローアップ Issue** がよい。`PluginOrigin::Unknown` コメントも「flattened_name 単独キー」前提のため、Cursor 元名化はその前提を部分的に崩す。

### B. `list_all_placed` / `is_enabled` フォールバック（要追記）

```rust
// meta.rs — statusByTarget 欠落時
let prefix = format!("{}_", plugin_name);
deployed.iter().any(|name| name.starts_with(&prefix))
```

`list_all_placed` は全ターゲットの `list_placed` 名を集約する。Cursor が `spec-driven-dev` だけを返すようになると、**meta 無し・Cursor のみ配置**のプラグインはフォールバック判定で enabled とみなされない。

現状の主経路は `statusByTarget` 優先のため致命傷ではないが、次を受入に含める:

- 通常 install 後は `statusByTarget` が書かれることを回帰テストで確認
- フォールバックに頼る古いキャッシュに対する既知制限をドキュメント化
- 将来 #331（prefix 衝突解消）と合わせて「配置物 → plugin 帰属」を meta / managedFiles 側へ寄せる

### C. ディレクトリ名と frontmatter `name` の事前不一致

`list_skill_names` は **ディレクトリ名** を original として返す。frontmatter の `name` は読まない。

配布物が既に `folder=foo` / `name: bar` と壊れている場合、元名配置でも Cursor 認識は失敗する。PLM の責任外でよいが、install 時の **警告（不一致検出）** は任意の品質向上として検討可（本 Issue の必須ではない）。

### D. ドキュメント更新

修正後は次を同期すること（#362 と同型のドキュメント負債を残さない）:

- `docs/concepts/targets.md` — Cursor Skills を `<original_name>/` に変更、衝突・レガシー注記
- `docs/concepts/deployment.md` — 展開先パス例の更新
- 必要なら `docs/commands/install.md` / getting-started のパス例

## 推奨実装手順

1. **モデル拡張**: `Component`（および配置コンテキスト）に `original_name` を追加。`flatten_components` で設定。既存テストの `Component { name, .. }` 初期化を更新
2. **Target API**: Cursor Skill の `placement_location` が `original_name` を使う（他ターゲットは `name` のまま）
3. **衝突ガード**: Cursor Skill 配置前に既存ディレクトリ検査（自管理なら許可、他ならエラー）
4. **レガシー削除**: disable/uninstall/update で旧 `{plugin}_{skill}` パスも削除
5. **テスト**: 下記マトリクス
6. **ドキュメント**: concepts 系を更新
7. **sync**: 既知制限として明記（またはフォローアップ Issue を切る）

## テストマトリクス（受入）

| ID | 内容 |
|----|------|
| T1 | Cursor Skill `placement_location` が `.cursor/skills/<original>/` を返す（Personal / Project） |
| T2 | 他ターゲットは従来どおり `.…/skills/<plugin>_<original>/` |
| T3 | 同名 original の別プラグインを Cursor へ二度目に置くとエラー |
| T4 | 同一プラグイン再 install は成功（上書き） |
| T5 | disable/uninstall が新パスを削除する |
| T6 | 旧フラット化パスのみ残っている状態で uninstall すると旧パスも消える |
| T7 | update 後に新パスへ配置され、旧パスが残らない |
| T8 | frontmatter 無改変（`name: <original>` のままコピーされる） |
| T9 | Codex↔Copilot sync は回帰なし（Cursor を含まないケース） |
| T10 |（任意）Cursor を含む sync は不一致またはスキップされることを文書/テストで固定 |

## 受入基準チェックリスト

### 設計（本レビュー）

- [x] 現象・原因の整理は現状コードと一致
- [x] 「Cursor のみ元名配置 / frontmatter 非書き換え」を承認
- [x] 衝突検出・レガシー掃除を MVP 必須と判定
- [x] sync / `is_enabled` フォールバックへの影響を指摘
- [x] Skills 限定 MVP を承認

### 実装（後続 PR）

- [ ] `original_name` を Component / Placement 経路へ伝播
- [ ] Cursor Skill 配置パスを元名に変更
- [ ] 衝突時エラー
- [ ] 旧フラット化パスのフォールバック削除
- [ ] 上記テストマトリクス
- [ ] concepts ドキュメント更新
- [ ] sync 制限またはフォローアップ Issue

## 関連

- #358 CursorTarget Skills 配置（フラット化導入の起点）
- #331 flatten prefix 衝突による enabled 判定の曖昧さ
- #362 Cursor ドキュメント整合性
- #376 Marketplace 直接 install UX（本 Issue の関連リンク）
