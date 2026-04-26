//! キャッシュからのプラグインロード
//!
//! キャッシュ配下のディレクトリから `PluginManifest` を読み込み、
//! `Plugin` 値を組み立てる責務のみを持つ。アンインストール後の
//! ディレクトリ整理は `plugin::cleanup` を参照。

use crate::plugin::{PackageCacheAccess, Plugin};
use crate::target::PluginOrigin;

/// キャッシュから Plugin を読み込む
///
/// # Arguments
///
/// * `cache` - package cache access used to read the manifest and path
/// * `marketplace` - marketplace name (`None` defaults to `"github"`)
/// * `plugin_name` - id (cache directory name; e.g. `owner--repo` for GitHub)
pub(crate) fn load_plugin(
    cache: &dyn PackageCacheAccess,
    marketplace: Option<&str>,
    plugin_name: &str,
) -> Result<Plugin, String> {
    let manifest = cache
        .load_manifest(marketplace, plugin_name)
        .map_err(|e| format!("Failed to load manifest: {}", e))?;

    let origin = PluginOrigin::from_cached_plugin(marketplace, plugin_name);
    let plugin_path = cache.plugin_path(marketplace, plugin_name);

    Plugin::new(manifest, plugin_path, origin).map_err(|e| format!("Failed to build plugin: {}", e))
}
