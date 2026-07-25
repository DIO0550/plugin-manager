# Task: 配置リテラル集約 (#339)

> 方針: 表示用 `plural()` と配置用パス断片を分離し、#338 LAYOUT / `placed/` の上に定数層を載せる

---

## Phase A: 棚卸し固定（コード変更なし）

- [ ] [exploration-report.md](./exploration-report.md) のリテラル一覧が現行 HEAD と一致することを確認
- [ ] [hearing-notes.md](./hearing-notes.md) の責務境界（非依存 vs 依存）をレビュー承認
- [ ] Phase D の API 方針（`Target` trait vs `pub(crate)` `TargetKind` テーブル）を仮決め
- [ ] `cargo test` でベースライン green を確認

---

## Phase B: ターゲット非依存定数

> **TDD**: Red → Green → Refactor

- [ ] RED: `ComponentKind::skill_manifest()` / `file_suffix()` の単体テスト（`kind_test.rs`）
  - Skill → `"SKILL.md"`
  - Agent → `Some(".agent.md")`、Command → `Some(".prompt.md")`、他 → `None`（設計どおり）
- [ ] GREEN: API 実装
- [ ] `plural()` に表示専用の doc comment を追加
- [ ] （任意）`default_subdir()` を追加し `plural()` と同一値を返すか、doc で「プラグイン内相対は plural を使う」と明記
- [ ] `scan/constants.rs` を上記への re-export に変更（または薄いラッパ）
- [ ] `cargo test component::` green

---

## Phase C: ヘルパ / deployment / scan 消費切替

- [ ] RED: `filter_skill_dir` が `skill_manifest()` と同じ文字列を使うことを固定するテスト（既存テスト維持 + 必要なら定数一致アサート）
- [ ] GREEN: `filter.rs` の `"SKILL.md"` を定数参照へ
- [ ] `placement_helpers.rs` の `"skills"` / `"agents"` / `".agent.md"` を定数参照へ
- [ ] `component/deployment.rs` の `"SKILL.md"` を定数参照へ
- [ ] `scan/components.rs` 等は re-export 経由のまま動作確認
- [ ] `cargo test target::placed` / 関連モジュール green

---

## Phase D: Target 依存パス + cleanup / placement

### Instruction

- [ ] RED: 「全 Target の instruction ファイル名集合」と `is_instruction_file` が一致する不変条件テスト
- [ ] GREEN: `instruction_filename()`（または同等テーブル）を追加し、各 env LAYOUT と接続
- [ ] REFACTOR: `scan/placement.rs` の静的 `INSTRUCTION_FILE_NAMES` をその集合から構築
- [ ] Antigravity（Instruction 非サポート）が `None` / 集合外であることを確認

### Env root + cleanup

- [ ] RED: `cleanup_specs` が返す base パスが、各 Target の LAYOUT ルートと一致するテスト
- [ ] GREEN: LAYOUT / `TargetKind` テーブルから roots と kind_subdir を供給
- [ ] REFACTOR: `cleanup.rs` のベアリテラル `".codex"` / `".github"` / `"prompts"` 等を除去
- [ ] Copilot Personal（Agent/Hook のみ）など scope 差分が現行どおりか既存 cleanup テストで確認

### Env list_placed / placement_location ベアリテラル

- [ ] Antigravity / Gemini CLI の `"skills"` を定数・subdir API へ
- [ ] Codex の `"skills"` / `"agents"` / `"hooks"` を同様に
- [ ] Copilot の `"prompts"` を **Target 依存 subdir** として明示（`plural()` に置換しない）
- [ ] Cursor の `"commands"` / `"skills"` / `"agents"` / `"hooks"` を同様に
- [ ] `cargo test target::` / `plugin::cache` green

---

## Phase E: wire / import / description

- [ ] `commands/info/wire.rs` / `list/wire.rs` のキー配列を `ComponentKind::plural()` ベースへ
- [ ] `commands/deploy/import.rs` の kind 文字列マッチを `plural()` 逆引きへ
- [ ] JSON 出力のスナップショット / 既存テストで値不変を確認
- [ ] （任意）`Scope::description()` を現行ターゲット含む文言へ更新 + テスト

---

## Phase F: docs / 掃除

- [ ] `scan/constants.rs` 削除 or re-export 残置を最終判断
- [ ] `docs/architecture/core-design.md` に定数層の短い節を追加
- [ ] `docs/roadmap.md` のリファクタ表に #339 完了を反映
- [ ] 本番コードで配置リテラルの残留を `rg` で監査（許容リスト: テストフィクスチャ、doc comment）
- [ ] 本計画ディレクトリを実装完了時に `docs/old/placement-literals-refactor/` へ退避

---

## 完了チェックリスト

- [ ] UC-1〜UC-5（requirements）を満たす
- [ ] `cargo fmt` / `cargo test` /（可能なら）`cargo clippy` 通過
- [ ] 振る舞い不変（パス・cleanup・JSON）を確認済み
