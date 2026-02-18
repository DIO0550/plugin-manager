//! プラグインアクション定義
//!
//! 高レベルユースケース（Install/Uninstall/Enable/Disable）を表す enum。

/// プラグインアクション（高レベルユースケース）
#[derive(Debug, Clone)]
pub enum PluginAction {
    Install {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Uninstall {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Enable {
        plugin_name: String,
        marketplace: Option<String>,
    },
    Disable {
        plugin_name: String,
        marketplace: Option<String>,
    },
}

impl PluginAction {
    /// アクションの種類を文字列で取得
    pub fn kind(&self) -> &'static str {
        match self {
            PluginAction::Install { .. } => "install",
            PluginAction::Uninstall { .. } => "uninstall",
            PluginAction::Enable { .. } => "enable",
            PluginAction::Disable { .. } => "disable",
        }
    }

    /// プラグイン名を取得
    pub fn plugin_name(&self) -> &str {
        match self {
            PluginAction::Install { plugin_name, .. }
            | PluginAction::Uninstall { plugin_name, .. }
            | PluginAction::Enable { plugin_name, .. }
            | PluginAction::Disable { plugin_name, .. } => plugin_name,
        }
    }

    /// マーケットプレイスを取得
    pub fn marketplace(&self) -> Option<&str> {
        match self {
            PluginAction::Install { marketplace, .. }
            | PluginAction::Uninstall { marketplace, .. }
            | PluginAction::Enable { marketplace, .. }
            | PluginAction::Disable { marketplace, .. } => marketplace.as_deref(),
        }
    }

    /// デプロイ系アクションか（Enable/Install）
    pub fn is_deploy(&self) -> bool {
        matches!(
            self,
            PluginAction::Install { .. } | PluginAction::Enable { .. }
        )
    }

    /// 削除系アクションか（Disable/Uninstall）
    pub fn is_remove(&self) -> bool {
        matches!(
            self,
            PluginAction::Uninstall { .. } | PluginAction::Disable { .. }
        )
    }
}

#[cfg(test)]
#[path = "plugin_action_test.rs"]
mod tests;
