# モジュール構造（035-iter-and-condition-refactoring 後）

## 全体アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CLI Layer (commands/)                        │
│  import.rs ─── install cmd ─── list cmd ─── info cmd ─── etc.      │
└────┬──────────────────┬──────────────────┬──────────────────────────┘
     │                  │                  │
     ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Application Layer (application/)                   │
│                                                                     │
│  ┌──────────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │  plugin_catalog   │  │  plugin_info  │  │  plugin_deployment    │ │
│  │                  │  │              │  │                       │ │
│  │ list_installed() │←─│ find_cands() │  │ load_deployment()    │ │
│  │ list_all_placed()│←─│ build_detail()│  │ cleanup_dirs()       │ │
│  │ list_installed_  │  │ parse_name() │  │                       │ │
│  │  plugins()       │  │              │  │                       │ │
│  └──────┬───────────┘  └──────┬───────┘  └───────────┬───────────┘ │
└─────────┼─────────────────────┼──────────────────────┼─────────────┘
          │                     │                      │
          ▼                     ▼                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Core Domain (install.rs)                       │
│                                                                     │
│  download_plugin() ──→ scan_plugin() ──→ place_plugin()            │
│       │                     │                  │                    │
│       ▼                     ▼                  ▼                    │
│  MarketplacePackage  ScannedPlugin    PlaceResult                  │
│  (domain type)       (MarketplacePkg  (PlaceSuccess/Failure)       │
│                       + components)                                │
└──────┬─────────────────────┬─────────────────────┬─────────────────┘
       │                     │                     │
       ▼                     ▼                     ▼
┌──────────────────┐  ┌───────────────────┐  ┌──────────────────────┐
│   source/         │  │   plugin/          │  │   target/            │
│                  │  │                   │  │                      │
│  PluginSource    │  │ MarketplacePackage│  │  Target trait        │
│  ├ GitHubSource  │  │ RemoteMarketplace │  │  ├ Codex             │
│  ├ MarketplaceSrc│  │   Data            │  │  ├ Copilot           │
│  └ SearchSource  │  │ PluginCache       │  │  ├ Antigravity       │
│                  │  │ PluginManifest    │  │  └ GeminiCli         │
│  parse_source()  │  │ PluginCacheKey    │  │                      │
└──────────────────┘  └───────────────────┘  └──────────────────────┘
```

## データ変換フロー

```
User Input
    │
    ▼
parse_source()
    │
    ├──→ GitHubSource ──────────┐
    ├──→ MarketplaceSource ─────┤
    └──→ SearchSource ──────────┤
                                ▼
                    CachedPackage
                    (name, marketplace, path,
                     manifest, git_ref, sha)
                                │
                                ▼
                        MarketplacePackage
                        (domain logic)
                                │
                                ▼
                        .components()
                                │
                                ▼
                    Vec<Component>
                    (kind, name, source_path)
                                │
                                ▼
                        ScannedPlugin
                        (name, components, marketplace)
                                │
                                ▼
                    ┌───────────────────────┐
                    │  place_plugin()       │
                    │                       │
                    │  for target in targets │
                    │    for comp in comps   │
                    │      placement_loc()   │
                    │      build_deploy()    │
                    │      deploy.execute()  │
                    └───────────────────────┘
                                │
                                ▼
                    PlaceResult (success/failure)
```

## 複雑度ホットスポット（CC = 循環的複雑度）

```
┌─────────────────────────────────────────────────────────────────┐
│                    複雑度が高い関数                              │
├─────────────────────────────┬──────────┬────────────────────────┤
│ 関数                        │ CC (推定) │ 原因                   │
├─────────────────────────────┼──────────┼────────────────────────┤
│ install.rs:place_plugin()   │  10-12   │ 2重ループ+match4分岐   │
│ import.rs:parse_comp_path() │   8-10   │ 7段バリデーション      │
│ search_source.rs:download() │   9-11   │ if/elseチェーン        │
│ cache.rs:list()             │   8-10   │ 2重ループ+条件分岐     │
│ marketplace_pkg:components()│   7-9    │ 5種コンポーネント走査  │
│ plugin_info:parse_plugin_   │   7-9    │ 8エラーケース判定      │
│   name()                    │          │                        │
│ import.rs:filter_components │   7-8    │ HashSet+マッチング     │
│ cache.rs:extract_archive()  │   7-8    │ Zip展開+フィルタ       │
└─────────────────────────────┴──────────┴────────────────────────┘
```

## plugin/ モジュール内部の型関係

```
┌───────────────────────────────────────────────────────────────┐
│  plugin/                                                      │
│                                                               │
│  ┌─────────────────────┐    From     ┌──────────────────────┐│
│  │ CachedPackage       │ ──────────→ │ MarketplacePackage   ││
│  │                     │             │                      ││
│  │ name                │             │ name                 ││
│  │ marketplace         │             │ marketplace          ││
│  │ path                │             │ path                 ││
│  │ manifest ───────────┼─────────────┼→ manifest            ││
│  │ git_ref             │             │                      ││
│  │ commit_sha          │             │ + components()       ││
│  └─────────────────────┘             │ + command_format()   ││
│                                      │ + agent_format()     ││
│                                      │ + skills_dir()       ││
│                                      └──────────────────────┘│
│                                                               │
│  ┌─────────────────────┐    wraps    ┌──────────────────────┐│
│  │ PluginCacheAccess   │ ◄────────── │ PluginCache          ││
│  │ (trait)             │             │ (impl, ~/.plm/cache) ││
│  │                     │             │                      ││
│  │ plugin_path()       │             │ cache_dir/           ││
│  │ is_cached()         │             │ ├ github/            ││
│  │ store_from_archive()│             │ │ └ owner--repo/     ││
│  │ load_manifest()     │             │ └ marketplace/       ││
│  │ list()              │             │   └ plugin-name/     ││
│  │ backup()/restore()  │             │                      ││
│  │ atomic_update()     │             │                      ││
│  └─────────────────────┘             └──────────────────────┘│
│                                                               │
│  ┌─────────────────────┐                                      │
│  │ PluginCacheKey      │  (plugin_catalog.rs 内)              │
│  │                     │                                      │
│  │ marketplace: Opt<S> │  cache.list()の結果を                │
│  │ dir_name: String    │  plugin_path/load_manifest へ渡す    │
│  └─────────────────────┘                                      │
└───────────────────────────────────────────────────────────────┘
```

## install.rs place_plugin() 内部フロー（最大CC）

```
place_plugin(targets, scanned, request)
│
├─ for target in targets ─────────────────────────┐
│   │                                              │
│   ├─ for component in scanned.components ──────┐ │
│   │   │                                        │ │
│   │   ├─ target.supports(comp.kind)?           │ │
│   │   │   └─ No → skip                        │ │
│   │   │                                        │ │
│   │   ├─ target.placement_location(origin,     │ │
│   │   │      comp.kind, scope)?                │ │
│   │   │                                        │ │
│   │   ├─ match comp.kind ──────────────┐       │ │
│   │   │   ├─ Command → set format      │       │ │
│   │   │   ├─ Agent  → set format       │       │ │
│   │   │   ├─ Hook   → set event/script │       │ │
│   │   │   └─ _      → default          │       │ │
│   │   │                    ◄───────────┘       │ │
│   │   │                                        │ │
│   │   ├─ builder.build()                       │ │
│   │   │                                        │ │
│   │   └─ deployment.execute()                  │ │
│   │       ├─ Ok  → PlaceSuccess               │ │
│   │       └─ Err → PlaceFailure               │ │
│   │                                            │ │
│   └────────────────────────────────────────────┘ │
│                                                  │
└──────────────────────────────────────────────────┘
```
