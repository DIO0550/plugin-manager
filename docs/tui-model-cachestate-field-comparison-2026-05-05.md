# TUI manager Model / CacheState フィールド比較レビュー

## 結論

- 3画面以上で重複する強い候補は、`state: ListState` と選択IDパターンに限られる。
- `Model` / `CacheState` の型名重複は画面ローカルな責務名であり、直ちに共通化対象とはしない。
- `Errors` は `CacheState` を持たず、`Model` も実質 unit-like であるため、形を揃える目的の空 `CacheState` 追加は不要。
- `installed` / `marketplaces` は enum variant ごとに状態の意味と寿命が異なるため、単一の共通構造へ寄せる判断は時期尚早。
- 今回はレビュー成果物のみを追加し、Rust ファイルは変更しない。

## 対象

| 画面 | 対象ファイル | 対象型 | 備考 |
|---|---|---|---|
| Discover | `src/tui/manager/screens/discover.rs` | `Model`, `CacheState` | `selected_id`, `ListState` を持つが、現状は UI 未実装寄り |
| Errors | `src/tui/manager/screens/errors.rs` | `Model` | `CacheState` なし。表示データは `DataStore.last_error` |
| Installed | `src/tui/manager/screens/installed/model.rs` | `Model`, `CacheState`, variants | `marked_ids`, `update_statuses`, 詳細/コンポーネント系 variant あり |
| Marketplaces | `src/tui/manager/screens/marketplaces/model.rs` | `Model`, `CacheState`, variants | `operation_status`, `error_message`, browse/install flow あり |

## 画面別フィールド比較

| 画面 | 型/variant | フィールド | Rust 型 | 意味 | CacheState 対象 | 根拠 |
|---|---|---|---|---|---|---|
| Discover | `CacheState` | `selected_id` | `Option<PluginId>` | タブ切替時に保持する選択中プラグインID | yes | `discover.rs:12-16` |
| Discover | `Model` | `selected_id` | `Option<PluginId>` | 画面内の選択中プラグインID。現状の view/update では実利用が薄い | yes | `discover.rs:18-21`, `discover.rs:43-54` |
| Discover | `Model` | `state` | `ListState` | ratatui のリスト選択状態。キャッシュ復元時は default に戻る | no | `discover.rs:20-21`, `discover.rs:43-47` |
| Errors | `Model` | なし | - | `DataStore.last_error` を表示するためローカルフィールドを持たない | no | `errors.rs:12-15`, `errors.rs:89-93` |
| Installed | `CacheState` | `selected_plugin_id` | `Option<PluginId>` | キャッシュ復元用の選択プラグインID | yes | `installed/model.rs:27-31`, `installed/model.rs:172-177` |
| Installed | `CacheState` | `marked_ids` | `HashSet<PluginId>` | バッチ更新対象としてマークしたプラグインID集合 | yes | `installed/model.rs:29-31`, `installed/model.rs:191-197` |
| Installed | `PluginList` | `selected_id` | `Option<PluginId>` | 一覧で選択中のプラグインID。`CacheState.selected_plugin_id` に変換される | yes | `installed/model.rs:108-112`, `installed/model.rs:210-217` |
| Installed | `PluginList` | `state` | `ListState` | 一覧の選択インデックス | no | `installed/model.rs:108-110`, `installed/model.rs:188-203` |
| Installed | `PluginList` | `marked_ids` | `HashSet<PluginId>` | マーク済みプラグインID集合 | yes | `installed/model.rs:108-112`, `installed/model.rs:210-217` |
| Installed | `PluginList` | `update_statuses` | `HashMap<PluginId, UpdateStatusDisplay>` | バッチ更新・個別更新の表示ステータス。復元時は空になる | no | `installed/model.rs:108-112`, `installed/model.rs:199-203` |
| Installed | `PluginDetail` | `plugin_id` | `PluginId` | 詳細画面の対象プラグインID。cache では選択IDとして保存される | yes | `installed/model.rs:115-117`, `installed/model.rs:218-225` |
| Installed | `PluginDetail` | `state` | `ListState` | 詳細画面のアクションメニュー選択状態 | no | `installed/model.rs:115-117` |
| Installed | `PluginDetail` | `saved_marked_ids` | `HashSet<PluginId>` | 一覧から遷移した時点のマーク状態退避 | yes | `installed/model.rs:118-121`, `installed/model.rs:218-225` |
| Installed | `PluginDetail` | `saved_update_statuses` | `HashMap<PluginId, UpdateStatusDisplay>` | 一覧から遷移した時点の更新表示状態退避。cache には含めない | no | `installed/model.rs:118-121` |
| Installed | `ComponentTypes` | `plugin_id` | `PluginId` | コンポーネント種別選択の対象プラグインID | yes | `installed/model.rs:123-131`, `installed/model.rs:226-230` |
| Installed | `ComponentTypes` | `selected_kind_idx` | `usize` | コンポーネント種別の選択位置 | no | `installed/model.rs:123-127` |
| Installed | `ComponentTypes` | `state` | `ListState` | 種別リストの選択状態 | no | `installed/model.rs:123-127` |
| Installed | `ComponentTypes` | `saved_marked_ids` | `HashSet<PluginId>` | PluginList から遷移チェーンで引き継いだマーク状態 | yes | `installed/model.rs:128-131`, `installed/model.rs:226-230` |
| Installed | `ComponentTypes` | `saved_update_statuses` | `HashMap<PluginId, UpdateStatusDisplay>` | PluginList から遷移チェーンで引き継いだ更新表示状態。cache には含めない | no | `installed/model.rs:128-131` |
| Installed | `ComponentList` | `plugin_id` | `PluginId` | コンポーネント一覧の対象プラグインID | yes | `installed/model.rs:133-142` |
| Installed | `ComponentList` | `kind` | `ComponentKind` | 表示するコンポーネント種別 | no | `installed/model.rs:133-138` |
| Installed | `ComponentList` | `selected_idx` | `usize` | コンポーネント一覧の選択位置 | no | `installed/model.rs:133-138` |
| Installed | `ComponentList` | `state` | `ListState` | コンポーネント一覧の選択状態 | no | `installed/model.rs:133-138` |
| Installed | `ComponentList` | `saved_marked_ids` | `HashSet<PluginId>` | PluginList から遷移チェーンで引き継いだマーク状態 | yes | `installed/model.rs:139-142` |
| Installed | `ComponentList` | `saved_update_statuses` | `HashMap<PluginId, UpdateStatusDisplay>` | PluginList から遷移チェーンで引き継いだ更新表示状態。cache には含めない | no | `installed/model.rs:139-142` |
| Marketplaces | `CacheState` | `selected_id` | `Option<String>` | タブ切替時に保持する選択中 marketplace 名 | yes | `marketplaces/model.rs:19-23` |
| Marketplaces | `MarketList` | `selected_id` | `Option<String>` | 一覧で選択中の marketplace 名。`+ Add new` では `None` になり得る | yes | `marketplaces/model.rs:115-120`, `marketplaces/model.rs:210-223` |
| Marketplaces | `MarketList` | `state` | `ListState` | 一覧の選択インデックス | no | `marketplaces/model.rs:115-120`, `marketplaces/model.rs:222-229` |
| Marketplaces | `MarketList` | `operation_status` | `Option<OperationStatus>` | update/remove など非同期操作の実行中状態 | no | `marketplaces/model.rs:12-17`, `marketplaces/model.rs:115-120` |
| Marketplaces | `MarketList` | `error_message` | `Option<String>` | 一覧操作のローカルエラー表示 | no | `marketplaces/model.rs:115-120` |
| Marketplaces | `MarketDetail` | `marketplace_name` | `String` | 詳細画面の対象 marketplace 名。cache では選択IDとして保存される | yes | `marketplaces/model.rs:122-129`, `marketplaces/model.rs:239-243` |
| Marketplaces | `MarketDetail` | `state` | `ListState` | アクションメニュー選択状態 | no | `marketplaces/model.rs:122-125` |
| Marketplaces | `MarketDetail` | `error_message` | `Option<String>` | 詳細画面の操作エラー表示 | no | `marketplaces/model.rs:122-125` |
| Marketplaces | `MarketDetail` | `browse_plugins` | `Option<Vec<BrowsePlugin>>` | ブラウズ状態の再入時復元用。意味はコードコメントに基づく | no | `marketplaces/model.rs:126-129` |
| Marketplaces | `MarketDetail` | `browse_selected` | `Option<HashSet<String>>` | ブラウズ選択状態の再入時復元用。意味はコードコメントに基づく | no | `marketplaces/model.rs:126-129` |
| Marketplaces | `PluginList` | `marketplace_name` | `String` | marketplace 内プラグイン一覧の対象名 | yes | `marketplaces/model.rs:131-137`, `marketplaces/model.rs:244-248` |
| Marketplaces | `PluginList` | `selected_idx` | `usize` | marketplace 内プラグイン一覧の選択位置 | no | `marketplaces/model.rs:131-135` |
| Marketplaces | `PluginList` | `state` | `ListState` | marketplace 内プラグイン一覧の選択状態 | no | `marketplaces/model.rs:131-135` |
| Marketplaces | `PluginList` | `plugins` | `Vec<(String, Option<String>)>` | ディスクI/O回避用のキャッシュ済みプラグインリスト | no | `marketplaces/model.rs:131-137` |
| Marketplaces | `AddFormModel::Source` | `source_input` / `error_message` | `String`, `Option<String>` | marketplace 追加フォームの source 入力状態 | no | `marketplaces/model.rs:66-72`, `marketplaces/model.rs:249` |
| Marketplaces | `AddFormModel::Name` | `source` / `name_input` / `default_name` / `error_message` | `String`, `Option<String>` | marketplace 追加フォームの name 入力状態 | no | `marketplaces/model.rs:73-79`, `marketplaces/model.rs:249` |
| Marketplaces | `AddFormModel::Confirm` | `source` / `name` / `error_message` | `String`, `Option<String>` | marketplace 追加フォームの確認状態 | no | `marketplaces/model.rs:80-85`, `marketplaces/model.rs:249` |
| Marketplaces | browse/install flow | `selected_plugins` | `HashSet<String>` | インストール対象として選択したプラグイン名集合 | no | `marketplaces/model.rs:141-165` |
| Marketplaces | browse/install flow | `highlighted_idx` | `usize` | ブラウズ/ターゲット/スコープ選択のハイライト位置 | no | `marketplaces/model.rs:141-165` |
| Marketplaces | `Installing` | `current_idx` / `total` | `usize` | インストール進捗表示用の位置と総数 | no | `marketplaces/model.rs:167-175` |
| Marketplaces | `InstallResult` | `summary` | `InstallSummary` | インストール結果表示用の集計 | no | `marketplaces/model.rs:97-110`, `marketplaces/model.rs:177-181` |

## 3画面以上の重複候補

| 候補 | 出現画面 | 同一性 | 共通化判断 | 理由 |
|---|---|---|---|---|
| `Model` 型名 | Discover / Errors / Installed / Marketplaces | 名前のみ | 共通化しない | 各画面モジュール内のローカル状態名であり、Feature 単位の閉じ込めとして自然。`Errors` はフィールドなし、`Installed` / `Marketplaces` は enum variant 構造で責務が異なる |
| `CacheState` 型名 | Discover / Installed / Marketplaces | タブ切替時の軽量状態という役割は近い | trait/helper は将来候補 | `ScreenCache` も3画面分だけ保持し、`Errors` は対象外。中身は `PluginId` / marketplace 名 / `marked_ids` で異なる |
| `state: ListState` | Discover / Installed / Marketplaces | 型と用途が近い | 小さな helper 候補 | ratatui の選択状態という意味は近い。ただし選択対象、stale ID 検証、復元ルールは画面別 |
| 選択IDパターン | Discover / Installed / Marketplaces | 意味は近いが型と検証先が異なる | 抽象化は慎重 | Discover/Installed は `PluginId`、Marketplaces は marketplace 名 `String`。Installed は `find_plugin`、Marketplaces は `find_marketplace` で検証する |

## 3画面以上ではない類似項目

| 候補 | 出現範囲 | 判定 | 理由 |
|---|---|---|---|
| `marked_ids` / `selected_plugins` | Installed / Marketplaces | 強い重複候補ではない | どちらも集合だが、前者はバッチ更新対象の `PluginId`、後者は marketplace からインストールするプラグイン名 |
| `update_statuses` / `operation_status` | Installed / Marketplaces | 強い重複候補ではない | 前者はプラグイン単位の表示結果、後者は marketplace 操作の実行中状態 |
| `error_message` | Marketplaces 内の複数 variant | 画面内重複 | Errors 画面は `DataStore.last_error` を表示し、ローカル `error_message` を持たない |
| `selected_idx` / `highlighted_idx` | Installed / Marketplaces の一部 variant | 画面内または2画面以下の類似 | 位置情報として近いが、対象リストやフローが異なる |
| `saved_marked_ids` / `saved_update_statuses` | Installed の sub screens | 画面内限定 | PluginList から詳細/コンポーネント画面へ遷移するための退避状態 |
| `browse_plugins` / `browse_selected` | Marketplaces の `MarketDetail` | 画面内限定 | ブラウズ画面への再入時復元用で、他画面に対応する構造がない |

## 共通化を急がない理由

| 差異 | 詳細 | リスク |
|---|---|---|
| Errors の扱い | `ScreenCache` は `installed` / `discover` / `marketplaces` のみを持ち、`switch_tab()` でも Errors は「キャッシュ不要」として扱う | 空 `CacheState` を追加すると、存在しない画面状態に責務を持たせる |
| 選択IDの検証先 | Installed は `DataStore::find_plugin` / `plugin_index`、Marketplaces は `find_marketplace` / `marketplace_index` を使う | 共通化で stale ID 検証を誤ると、存在しない項目を復元する |
| `ListState` の限界 | `ListState` は選択インデックスだけを持ち、選択対象の実体は別フィールドや `DataStore` にある | `ListState` だけ抽出しても、選択IDやフィルタ同期は解決しない |
| variant 固有状態 | Installed は詳細/コンポーネント遷移、Marketplaces は browse/install flow とフォームを持つ | 汎用構造に寄せると、画面固有の意味と寿命が読みにくくなる |
| 操作状態の意味差 | `update_statuses` は結果表示、`operation_status` は非同期操作の排他・実行中状態 | 名前や型の近さだけで統合すると、状態遷移の責務が混ざる |

## 推奨アクション

| 優先度 | アクション | 内容 |
|---|---|---|
| P3 | 現状維持 | 今回はレビュー文書のみ追加し、Rust ファイルは変更しない |
| P3 | 将来調査 | `ListState + selected key` helper の粒度を、実装重複が増えた時点で別 Issue として検討する |
| P3 | 将来調査 | `CacheState` trait を検討する場合も、Errors は対象外にできる設計を前提にする |
| P3 | 却下案明記 | Errors に空 `CacheState` を追加して4画面の形だけ揃える案は不要 |

## 手動検証

- 対象4画面の `Model` / `CacheState` / 関連 variant 状態を比較表に反映した。
- `Errors` は `CacheState` を持たず、`Model` が実質 unit-like であることを明記した。
- 3画面以上の候補と、2画面以下または画面内限定の類似項目を分けて記載した。
- `installed` / `marketplaces` は enum variant ごとにフィールド構成が異なることを明記した。
- 意味がコードコメントや遷移からの推定に留まる項目は、根拠または推定元を併記した。
- Rust ファイル変更は不要であり、今回の成果物は `docs/` 配下のレビュー文書のみである。
