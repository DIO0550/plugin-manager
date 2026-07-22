# Issue #392 レビュー: Skill ディレクトリ内の未認識ファイル/フォルダを Skill と一緒に配置することを仕様化する

| 項目 | 内容 |
|------|------|
| Issue | [#392 [feature/scan] Skill ディレクトリ内の未認識ファイル/フォルダを Skill と一緒に配置することを仕様化する](https://github.com/DIO0550/plugin-manager/issues/392) |
| レビュー日 | 2026-07-22 |
| 対象ブランチ | `main`（`872de50` — #344 PLM_HOME 統一 / #391 マージ後） |
| 関連 | [#393](https://github.com/DIO0550/plugin-manager/issues/393)（Plugin 直下リソース — **別 Issue**）、[#339](https://github.com/DIO0550/plugin-manager/issues/339)（リテラル集約） |

## サマリー

現状の `deploy_skill()` は `fs.replace_dir(source, target)` により Skill ディレクトリを丸ごとコピーしており、`references/` / `assets/` / 直下の補助 md なども**結果として同梱されている**。Issue が指摘する通り、これは仕様として明文化されておらず、保証テストも薄い。提案（仕様追記 + 回帰テスト固定）は実装変更をほぼ伴わない仕様固定であり、**方針として承認**する。

実装前に、用語定義・ドキュメント追記箇所・テスト粒度・Cursor 再帰走査との関係を Issue / 実装メモへ明示することを推奨する。

**結論: 方針承認。ブロッカー級の設計変更は不要。下記の明確化を受け入れ条件に含めること。**

## 現状コードとの一致確認

| 経路 | 挙動 | 根拠 |
|------|------|------|
| Skill デプロイ（install / import） | `replace_dir` で Skill ディレクトリ全体をコピー後、必要なら `target_path/SKILL.md` のみ frontmatter strip | `src/component/deployment.rs` `deploy_skill` |
| Skill ライフサイクル（enable 等） | `FileOperation::CopyDir` → 同じく `replace_dir`（変換なし） | `src/plugin/lifecycle/intent.rs` |
| Skill スキャン | 直下に正確な `SKILL.md` があるディレクトリのみ採用。採用後は配下に潜らない | `src/scan/components.rs` `collect_skills_recursive` |
| 既存テスト（部分） | 直下 `helper.py` のコピー、Codex strip 後も `helper.py` 残存、`assets/inner/SKILL.md` の二重スキャン防止 | `deployment_test.rs` / `scan/components/tests.rs` |
| 既存ドキュメント（部分） | Codex 向け変換節に「ディレクトリ内の他ファイルはそのままコピー」と一文あり | `docs/architecture/file-formats.md` Skill → Codex |

Issue の現状分析はコードと一致する。挙動自体は既に正しいが、**契約としての固定**が欠けている。

## 設計判断のレビュー

### 1. 仕様化（`file-formats.md` 追記）

#### 結論: **承認。追記文言は「付属リソース」の定義を明示すること**

提案文言の骨子は妥当。次を仕様に含める:

```text
Skill ディレクトリ（SKILL.md を直下に持つディレクトリ）内のエントリのうち、
PLM が別 Component（Skill / Agent / Command / Instruction / Hook）として
スキャン・登録しないファイル・フォルダは、当該 Skill の付属リソースとして扱い、
deploy 時に Skill 本体と同じ相対構造でターゲットへコピーする。
```

明確化が必要な点:

| 論点 | 推奨定義 |
|------|----------|
| 「未認識」の単位 | **プラグインルートのスキャン規約**ではなく、**Skill ディレクトリ内の全エントリ**が付属リソース候補。Skill 採用後は配下を再スキャンしないため、配下の `SKILL.md` も「別 Skill」にはならない |
| フォルダ名の制約 | なし（`references/` / `assets/` / `templates/` / `examples/` / `docs/` / 任意名）。ホワイトリスト方式は採用しない |
| 直下ファイル | `SKILL.md` 以外の md / スクリプト / バイナリも付属リソース |
| stale 掃除 | `replace_dir` によりターゲット側の余剰ファイルは削除される（更新時に消えた付属ファイルも消える）ことを仕様に書く |
| #393 との境界 | **Skill ディレクトリ内**のみが本 Issue。`plugins/<plugin>/references/` など Plugin 直下は #393 |

追記先は Issue 提案どおり `docs/architecture/file-formats.md` を主とし、併せて次も例示を足すと負債が残らない:

- `docs/concepts/components.md`（Skills の特徴に「付属リソース同梱」）
- `docs/concepts/deployment.md`（展開例に `references/` / `assets/` を含める）

### 2. スキャン保証テスト（二重検出防止を任意名でも固定）

#### 結論: **承認。既存 `assets` ケースのパラメタ拡張で十分**

既存:

```text
test_list_skill_names_does_not_descend_into_skill
  skills/skill1/SKILL.md
  skills/skill1/assets/inner/SKILL.md  → skill1 のみ
```

追加推奨:

| ID | 内容 |
|----|------|
| S1 | `references/` / `templates/` / `docs/` / `examples/` / 任意名 `foo-bar/` 配下の `SKILL.md` も二重スキャンされない（テーブル駆動でよい） |
| S2 | Skill 直下の任意 `.md`（`notes.md` 等）は Skill として採用されない（現状どおりディレクトリ＋`SKILL.md` のみ） |
| S3 |（任意）ネスト skills（`skills/bar/baz/SKILL.md`）と、baz 内付属フォルダの組み合わせでも baz のみ |

スキャン層の変更は不要。テスト追加のみで契約を固定する。

### 3. デプロイ保証テスト（付属リソースの同構造コピー）

#### 結論: **承認。ただし「全ターゲット」は配置パス統合ではなく `deploy_skill` + `ConversionConfig::Skill` の差で足りる**

`deploy_skill` はターゲット非依存の `replace_dir` であり、ターゲット差分は frontmatter strip の有無だけである。

| ターゲット | ディレクトリコピー | frontmatter strip |
|------------|-------------------|-------------------|
| Codex | ○（共通） | ○（許可フィールドのみ） |
| Gemini CLI | ○ | ○（`name`/`description` のみ） |
| Copilot / Antigravity / Cursor | ○ | なし（無改変） |

推奨テスト:

| ID | 内容 |
|----|------|
| D1 | source に `SKILL.md` + `references/a.md` + `assets/templates/x.html` + 直下 `notes.md` を置き、deploy 後に target 側へ同相対パスで存在 |
| D2 | Codex / Gemini CLI で strip 後も付属ファイルの内容・パスが不変 |
| D3 | Copilot / Antigravity / Cursor（`ConversionConfig::Skill`）でも同構造コピー |
| D4 | 既存ターゲットに古い付属ファイルがある状態で、source から消した付属が `replace_dir` で消える（stale 掃除） |
| D5 |（任意）`PluginIntent` の `CopyDir` 経路でも同構造コピー（enable 再配置の回帰） |

Issue 本文の「全ターゲットで確認」は、**各 Target の `placement_location` 統合テストまで必須としない**（配置パス自体は本 Issue の対象外）。`ComponentDeployment` に各 `TargetKind` の `ConversionConfig::Skill` を渡すユニットテストで契約を固定すれば十分。実ターゲットパスまでの E2E は任意の補強。

既存の `test_execute_copies_directory_for_skill`（`helper.py`）は薄い契約として残し、上記を明示的な付属リソースケースとして追加する。

### 4. frontmatter 変換の副作用固定

#### 結論: **承認。コード上すでに `target_path/SKILL.md` のみ触る**

```rust
let manifest = self.target_path.join("SKILL.md");
// ... strip ... atomic_write(&manifest, &stripped)
```

回帰テストで次を固定する:

| ID | 内容 |
|----|------|
| F1 | 付属 md（例: `references/guide.md`）に非対応 frontmatter があっても内容が変わらない |
| F2 | strip 対象は常に `target_path/SKILL.md` のみ（パス結合のリテラル依存をテストで固定） |

`strip_skill_frontmatter_fields` 単体テスト群は既に充実しているため、本 Issue では **deployment 結合テスト**側に寄せる。

## Issue 設計で不足している横断影響

### A. Cursor の skills ルート再帰走査（要追記・既知制限で可）

PLM スキャンは Skill 採用後に潜らないが、Cursor は skills ルートを再帰走査して `SKILL.md` を発見する（`docs/concepts/targets.md`）。

したがって、配布物が次のような構造だと:

```text
skills/spec-driven-dev/
├── SKILL.md
└── assets/inner/SKILL.md   # 付属としてコピーされる
```

PLM は 1 Skill として配置するが、**Cursor 実行時にはネスト側も別 Skill として見える可能性**がある。これは PLM のコピー契約とは別レイヤの問題であり、本 Issue のブロッカーではない。

推奨:

- 仕様に「PLM は Skill 配下のネスト `SKILL.md` を別 Component にしない。ターゲット実行時の再帰発見はターゲット仕様に依存し、PLM は変換・除外しない」と明記
- 必要ならフォローアップ（ネスト `SKILL.md` の警告）を別 Issue に切る

### B. 二つのデプロイ経路

| 経路 | 付属リソース | frontmatter strip |
|------|--------------|-------------------|
| `ComponentDeployment`（install / import） | ○ `replace_dir` | ○ |
| `PluginIntent`（enable 等） | ○ `replace_dir` | ×（現状） |

本 Issue の主題は付属リソース同梱なので、両経路とも `replace_dir` である点で契約は満たす。strip の経路差は既存課題であり、本 Issue のスコープ外でよい（触るなら別 Issue）。

### C. symlink

スキャンは symlink を辿らない。一方 `copy_dir_recursive` は `is_dir()` / `std::fs::copy` ベースで、symlink の扱いがスキャンと完全一致しない可能性がある。一般配布 Skill では稀なので本 Issue 必須ではないが、「symlink は保証外」と一文あると安全。

### D. #393 との混同防止

| | #392（本 Issue） | #393 |
|--|------------------|------|
| 対象 | `skills/<skill>/` 内 | `plugins/<plugin>/` 直下 |
| 現状 | `replace_dir` で既にコピーされる | 無視される（新機能が必要） |
| 作業量 | 仕様 + テスト中心 | 検出・配置戦略・ライフサイクル新設 |

本 Issue の受け入れ条件に「Plugin 直下は扱わない（#393）」を明示すること。

### E. #339 との関係

「未認識」判定の真実源を `ComponentKind` / scan constants に寄せる話は #339。本 Issue では **現行のスキャン結果（Skill として採用されたディレクトリの中身全部）** を契約にすれば足り、リテラル集約を待たなくてよい。

## 推奨実装手順

1. **仕様追記**: `file-formats.md` に「Skill 付属リソース」節を追加（定義・境界・stale 掃除・#393 除外・Cursor 既知制限）
2. **concepts 同期**: `components.md` / `deployment.md` に短い例を追加
3. **スキャンテスト**: 任意名フォルダでの非二重検出をテーブル駆動で追加
4. **デプロイテスト**: 同構造コピー + stale 掃除 + strip 非干渉（各 `TargetKind` の `ConversionConfig::Skill`）
5. **（任意）** `PluginIntent` CopyDir の薄い回帰
6. **Rust 実装変更は原則不要**（契約が崩れている箇所が見つかった場合のみ修正）

## 受入基準チェックリスト

### 設計（本レビュー）

- [x] 現状は `replace_dir` により付属リソースが既に同梱されていることを確認
- [x] 仕様化 + 保証テスト中心の方針を承認
- [x] 「未認識」の定義と #393 境界の明確化を要求
- [x] Cursor 再帰走査は既知制限として文書化するよう指摘
- [x] 全ターゲット E2E は必須とせず、`deploy_skill` + `TargetKind` 差のユニットで足りると判断

### 実装（後続 PR）

- [ ] `docs/architecture/file-formats.md` に Skill 付属リソース節を追加
- [ ] `docs/concepts/components.md` / `deployment.md` の例示更新
- [ ] スキャン: 任意名フォルダ内 `SKILL.md` の非二重検出テスト
- [ ] デプロイ: 同構造コピー・stale 掃除・付属 md の frontmatter 非改変テスト
- [ ] #393 / Cursor 既知制限を仕様または Issue 本文に明記

## 関連

- #393 Plugin 直下の未認識ファイル/フォルダ配置（後続・別スコープ）
- #339 配置ディレクトリ名・ファイル名リテラルの集約
- #377 Cursor Skill の original_name 配置（配置パスは本 Issue 対象外、コピー契約とは直交）
