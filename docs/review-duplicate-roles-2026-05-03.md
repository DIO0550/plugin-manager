# 同一役割の trait / struct + 命名レビュー（2026-05-03）

## 対象
- `src/` 配下の `trait` / `struct` 定義を横断確認。
- 観点:
  1. 「責務がほぼ同じなのに、別名・別場所で重複しているか」
  2. 命名の一貫性（型名・モジュール名・文脈依存名の妥当性）

## 結論（要約）
- **明確に統合すべき致命的な重複は見当たらない**。
- 命名も概ね一貫している。
- ただし、次の3点は「役割が近く、将来的に共通化候補」と判断。

## 重複観点の詳細

### 1) Source/Destination の構造的対称性
- `SyncSource` と `SyncDestination` は同期対象の両端を表し、責務が非常に近い構造です。
- ただし意味論（入力側/出力側）が異なるため、現時点で無理に1型へ統合するより、`enum Endpoint { Source, Destination }` のような抽象化を**必要が出た時点で**導入するのが妥当です。
- 該当:
  - `src/sync/endpoint/source.rs`
  - `src/sync/endpoint/destination.rs`

### 2) TUI の画面別 `Model` / `CacheState`
- `tui/manager/screens/*` に画面単位で `Model` や `CacheState` が存在し、名前だけ見ると重複に見えます。
- ただし Feature ごとに状態を閉じ込める設計としては自然で、現時点では「意図的な重複（局所最適）」と評価。
- 改善余地として、共通フィールドが増えてきた場合のみ `core` 側への抽出を検討。
- 該当例:
  - `src/tui/manager/screens/discover.rs`
  - `src/tui/manager/screens/errors.rs`
  - `src/tui/manager/screens/installed/model.rs`
  - `src/tui/manager/screens/marketplaces/model.rs`

### 3) `commands/*` の `Args` 群
- 各サブコマンドで `pub struct Args` を持つ構造は命名が同じです。
- ただしモジュール境界で完全に分離され、CLI定義として定番パターンのため、重複問題とは見なしません。
- ただし将来、共通オプション（例: `--json`, `--target`）が増える場合は埋め込み用の共通 struct を導入すると保守性が上がります。
- 該当:
  - `src/commands/*.rs`

## 命名レビュー

### 良い点
1. **trait 名は役割が明確**
   - `HostClient`, `FileSystem`, `PackageSource`, `Target`, `PackageCacheAccess` と、抽象境界が用途ベースで読みやすい。
2. **Feature配下の型名が文脈に沿っている**
   - `MarketplaceRegistry`, `PluginManifest`, `TargetRegistry` など、機能領域が型名から推測しやすい。
3. **値オブジェクト系の命名が具体的**
   - `ScopedPath`, `TargetId`, `PluginSourcePath` など意味が明確。

### 気になった点（軽微）
1. **`Model` の汎用名が多い**
   - 画面ローカルでは妥当だが、将来ファイル横断で参照が増えると `Model` だけでは識別しづらくなる。
   - 対応案: 公開境界に出る時だけ `InstalledScreenModel` のような具体名を検討。
2. **`Result` 系の型名が文脈依存**
   - `SyncResult`, `ConvertResult`, `OperationResult`, `UpdateResult` などが併存。
   - 現在はモジュール分離で問題ないが、横断利用が増える場合は接頭辞を強めると可読性向上。
3. **`Args` の統一は良いが、共通引数の再利用余地あり**
   - 重複ではないが、同じ意味の引数を複数コマンドで持つなら共通パーツ化可能。

### 命名に関する総合判定
- 現状の命名は、**Rustの一般的慣習（型: CamelCase / 関数・モジュール: snake_case）とFeature分割の方針に整合**しており、大きな問題はなし。
- 改名コストに見合う改善対象は現時点では少なく、まずは現状維持が妥当。

## 推奨アクション（優先度順）
1. **現状維持**（過剰な抽象化・過剰な改名は避ける）。
2. `sync/endpoint` でロジック重複が増えたら共通抽象を導入。
3. `tui` 画面間で同一フィールドが3画面以上に増えたら `core` に共通 state を切り出す。
4. `commands` で共通CLIオプションが増えたら共通 `Args` 部品を導入。
5. 画面モデルが公開境界へ出る場合のみ `Model` を具体名へ段階的に改名。

## 判定
- 現在のコードベースでは、**「無駄に同じような役割の trait / struct が乱立している」状態ではない**。
- 命名も実用上は十分に一貫しており、改善は将来の拡張タイミングで段階的に行うのが適切。
