# レビュー / 実装状況: Issue #393 Plugin 直下の未認識ファイル/フォルダ配置

更新日: 2026-07-28  
対象: [#393](https://github.com/DIO0550/plugin-manager/issues/393)  
仕様確定: [#407](https://github.com/DIO0550/plugin-manager/pull/407)（MERGED・docs only）  
関連: [#392](https://github.com/DIO0550/plugin-manager/issues/392) / [#395](https://github.com/DIO0550/plugin-manager/pull/395) / [#339](https://github.com/DIO0550/plugin-manager/issues/339) / [#377](https://github.com/DIO0550/plugin-manager/issues/377)

## 判定（実装後）

| レイヤ | 状態 | 備考 |
|--------|------|------|
| 配置方針（案 A） | **確定済み** | #407 |
| 仕様書 | **充足** | `file-formats.md`「Plugin 付属リソース」 |
| Scan（プラグイン直下の未認識列挙） | **実装済み** | `scan::list_plugin_attached_resources` |
| Deploy（Skill 配下へ overlay） | **実装済み** | `deploy_skill` の `replace_dir` 後 |
| 衝突解決（Skill 優先 + 警告） | **実装済み** | `AttachedResourceWarning` → `PlaceSuccess.attached_warnings` |
| install / enable | **実装済み** | `ConversionConfig::Skill.plugin_root` / `PluginIntent::with_plugin_root` |
| sync stale 非対称 | **対象外（別途）** | 既存の `copy_dir` 非対称は未解消 |
| 統合テスト | **実装済み** | 仮想 spec-plugin レイアウト + 全 Skill 対応 TargetKind |

**結論:** 案 A の中核（検出・配置・衝突・install/enable）は実装済み。Issue クローズ可。sync の stale 非対称は別 Issue でもよい。

---

## 実装マップ

| 層 | パス |
|----|------|
| 除外定数 | `src/placement_names.rs`（`ATTACHED_*`） |
| 検出 | `src/scan/attached.rs` |
| 除外合成 | `src/plugin/attached.rs` |
| overlay | `src/component/deployment/attached.rs` |
| Skill 配置 | `src/component/deployment.rs` `deploy_skill` |
| install | `src/install.rs`（`plugin_root: Some(...)`） |
| enable | `src/plugin/lifecycle/intent.rs` + `application/lifecycle.rs` |
| 警告表示 | `src/commands/deploy/install.rs` `render_place_success_to_strings` |

---

## 仕様どおりの対象外

- 相対参照の書き換え
- 別 Skill を指す相対参照
- Skill を持たないプラグインへの配置
- sync の `copy_dir` vs `replace_dir` 非対称の解消
