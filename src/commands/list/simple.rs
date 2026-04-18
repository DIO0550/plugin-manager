//! シンプル（plugin 名のみ）出力フォーマット

use crate::plugin::InstalledPlugin;

pub(super) fn print_simple(plugins: &[InstalledPlugin], total_count: usize) {
    if plugins.is_empty() {
        if total_count == 0 {
            println!("No plugins installed");
        } else {
            println!("No plugins matched");
        }
        return;
    }
    for plugin in plugins {
        println!("{}", plugin.name());
    }
}
