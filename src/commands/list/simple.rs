//! シンプル（plugin 名のみ）出力フォーマット

use crate::plugin::InstalledPlugin;

pub(super) fn print_simple(plugins: &[InstalledPlugin], total_count: usize) {
    if plugins.is_empty() {
        super::print_empty_list(total_count);
        return;
    }
    for plugin in plugins {
        println!("{}", plugin.name());
    }
}
