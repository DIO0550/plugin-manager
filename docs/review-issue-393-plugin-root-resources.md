# レビュー: Issue #393 Plugin 直下の未認識ファイル/フォルダ配置

レビュー日: 2026-07-27  
対象: [#393](https://github.com/DIO0550/plugin-manager/issues/393)  
仕様確定: [#407](https://github.com/DIO0550/plugin-manager/pull/407)（MERGED・docs only）  
関連: [#392](https://github.com/DIO0550/plugin-manager/issues/392) / [#395](https://github.com/DIO0550/plugin-manager/pull/395) / [#339](https://github.com/DIO0550/plugin-manager/issues/339) / [#377](https://github.com/DIO0550/plugin-manager/issues/377)

## 判定

| レイヤ | 状態 | 備考 |
|--------|------|------|
| 配置方針（案 A vs 案 B） | **確定済み** | 案 A（各 Skill 配下へ相対パス複製）。#407 |
| 仕様書 | **充足** | `docs/architecture/file-formats.md`「Plugin 付属リソース」+ concepts 追随 |
| Scan（プラグイン直下の未認識列挙） | **未実装** | `Plugin::build_components` は 5 種 Component のみ |
| Deploy（Skill 配下へ overlay） | **未実装** | `deploy_skill` は Skill ソース dir の `replace_dir` のみ |
| 衝突解決（Skill 優先 + 警告） | **未実装** | 配置警告の共通チャネルも未整備 |
| ライフサイクル一貫性 | **未実装** | install / enable の二重経路、sync の `copy_dir` 非対称に注意 |
| 統合テスト（spec-plugin） | **未実装** | Skill 内付属（#392）のテストのみ存在 |

**結論:** 仕様議論は #407 で閉じている。Issue は **実装チケットとして OPEN のまま維持**すべき。本文の「managedFiles に登録」は案 A 採用後は不要なので、実装前に Issue 本文の更新を推奨する。

---

## 仕様（#407）で確定したこと

1. **案 A** — 各 Skill の配置ディレクトリへ、プラグインルートからの相対パスを保って複製する。
2. **案 B 不採用** — `<plugin>_shared/` 兄弟配置は、Cursor（`original_name`）と他ターゲット（`<plugin>_<skill>`）で相対パスが一意にならない。
3. **`ComponentKind` は拡張しない** — 付属リソースは別枠。
4. **`managedFiles` 個別登録は不要** — Skill ディレクトリに閉じるため、disable/uninstall の `RemoveDir` で足りる。
5. **衝突は Skill 側優先** + 警告。
6. **除外はマニフェスト解決後パス**で判定（リテラル `skills/` 比較だけだとカスタムパスを誤同梱する）。

正本: `docs/architecture/file-formats.md`「Plugin 付属リソース」。

---

## 実装ギャップ（コード根拠）

| 箇所 | 現状 |
|------|------|
| `src/plugin/content/plugin_content.rs` `build_components` | Skill/Agent/Command/Hook + Instruction のみ。ルート直下 `references/` 等は未列挙 |
| `src/scan/` | `list_skill/agent/command/hook/markdown_names` のみ。plugin-root 非 Component API 無し |
| `src/component/deployment.rs` `deploy_skill` | `replace_dir(skill_source, target)` → 任意の `SKILL.md` strip → `Copied`。overlay 無し |
| `src/plugin/lifecycle/intent.rs` | enable Skill は `CopyDir { source: component.path }` → 実装は `replace_dir`。Plugin ルートを見ない |
| `src/sync.rs` `execute_create` | Skill は `copy_dir`（replace ではない）。stale が残る既存非対称 |
| `src/placement_names.rs` | Instruction 名・環境ルートのみ。README/LICENSE/VCS/`.claude-plugin` 等の除外定数は未追加（docs が先走り） |
| `DeploymentOutput` | `Copied` に warnings 無し。`ConversionWarning` は Hook 専用 |

Skill 内付属（#392）は `replace_dir` で実装済み。**Plugin 直下とは別コードパスが必要**で、現状は存在しない。

---

## Issue 本文との差分（実装前に直すとよい点）

| Issue 本文 | #407 仕様 |
|------------|-----------|
| update 時に `managedFiles` へ登録し掃除 | **不要**（Skill dir 内に閉じる） |
| 案 A / 案 B を Issue 内で確定したい | **案 A で確定済み** |
| 除外: `.claude-plugin` / 既定 dir / Instruction / VCS | 仕様は README*/LICENSE*/CI メタ・総量閾値まで拡張 |

---

## 実装チェックリスト（次の PR 向け）

1. **検出** — `scan/` に plugin-root 付属列挙を新設（`ComponentKind` に混ぜない）。除外は `placement_names` 定数 + `PluginManifest` 解決パスの合成。
2. **配置** — `deploy_skill` の `replace_dir` **後**に overlay（先に書くと消える）。衝突パスは skip + warn。
3. **enable** — `intent` の `CopyDir` 経路にも同じ overlay（install と二重経路）。
4. **警告** — Hook 専用 `ConversionWarning` に載せない。配置結果へ警告を集約する共通チャネルを用意する。
5. **閾値** — 付属総量超過時は配置せず警告（仕様どおり）。
6. **テスト** — 検出・除外・衝突・全 `TargetKind` の配置。仮想 `spec-plugin` レイアウトで `references/tdd-guidelines.md` が各 Skill 配下に存在することを保証。
7. **sync** — 最低限「作成時に同梱」。stale 非対称（`copy_dir` vs `replace_dir`）の解消は本 Issue に含めてもよいが、別 Issue でも可。

### 既知の対象外（仕様どおり）

- 相対参照の書き換え（`../../references/...` は救済しない）
- 別 Skill を指す相対参照（`../other-skill/...`）— フラット化で壊れる問題は別起票
- Skill を持たないプラグインへの配置（配置先が無い）

---

## docs 上の注意

- `docs/concepts/components.md` / `deployment.md` は Plugin 付属を「複製される」と**現在形**で書くが、実装は未追随。仕様記述として読む。
- `deployment.md` の展開先パス例は旧 3 階層表記のまま（コードは `{plugin}_{skill}` / Cursor は `original_name`）。#393 本体とは別の docs ずれ。

---

## 推奨アクション

1. **#393 はクローズしない** — 実装・テスト完了まで OPEN。
2. Issue 本文を #407 仕様に合わせて更新（managedFiles / 案 A 確定 / 除外リスト）。
3. 実装 PR は本レビューのチェックリストを受け入れ条件にする。
