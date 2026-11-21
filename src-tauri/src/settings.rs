//! Persisted JSON models. Field names and enum ordinals match Fishstrap's
//! `Settings.json` / `State.json` / `RobloxState.json` so a Fishstrap install can be
//! imported verbatim. Unknown fields are preserved via `extra`.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fs, path::Path};

/// `ChannelChangeMode` ordinals.
pub mod channel_change_mode {
    pub const AUTOMATIC: u8 = 0;
    pub const PROMPT: u8 = 1;
    pub const IGNORE: u8 = 2;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CustomIntegration {
    pub name: String,
    pub location: String,
    pub launch_args: String,
    pub delay: i32,
    pub pre_launch: bool,
    pub auto_close: bool,
}

impl Default for CustomIntegration {
    fn default() -> Self {
        Self { name: String::new(), location: String::new(), launch_args: String::new(), delay: 0, pre_launch: false, auto_close: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Settings {
    // Behaviour
    pub allow_cookie_access: bool,
    pub confirm_launches: bool,
    pub enable_better_matchmaking: bool,
    pub enable_better_matchmaking_randomization: bool,
    pub force_roblox_language: bool,
    pub background_updates_enabled: bool,
    pub cleaner_options: u8,
    pub cleaner_directories: Vec<String>,
    // Appearance
    pub bootstrapper_style: u8,
    pub bootstrapper_icon: u8,
    pub bootstrapper_title: String,
    pub bootstrapper_icon_custom_location: String,
    pub roblox_icon: u8,
    pub roblox_title: String,
    pub roblox_icon_custom_location: String,
    pub theme: u8,
    pub use_acrylic_background: bool,
    pub acrylic_background_opacity: i32,
    pub locale: String,
    pub selected_custom_theme: Option<String>,
    // Deployment
    pub check_for_updates: bool,
    pub static_directory: bool,
    pub channel: String,
    pub roblox_domain: String,
    pub channel_change_mode: u8,
    pub distributor_type: u8,
    pub web_environment: String,
    pub debug_disable_version_package_cleanup: bool,
    pub force_local_data: bool,
    // FastFlags
    pub use_fast_flag_manager: bool,
    /// ztmystrap: also write flags that are not on Roblox's allowlist.
    pub apply_non_allowlisted_flags: bool,
    // Integrations
    pub enable_roblox_background_app: bool,
    pub roblox_theme: u8,
    pub enable_window_manipulation: bool,
    pub fake_borderless_fullscreen: bool,
    pub enable_activity_tracking: bool,
    #[serde(rename = "UseDiscordRichPresence")]
    pub use_discord_rich_presence: bool,
    #[serde(rename = "RichPresenceStatusDisplayType")]
    pub rich_presence_status_display_type: u8,
    #[serde(rename = "HideRPCButtons")]
    pub hide_rpc_buttons: bool,
    pub allow_discord_invites: bool,
    pub show_account_on_rich_presence: bool,
    pub show_server_details: bool,
    pub custom_integrations: Vec<CustomIntegration>,
    pub use_disable_app_patch: bool,
    #[serde(rename = "WPFSoftwareRender")]
    pub wpf_software_render: bool,
    pub enable_analytics: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            allow_cookie_access: false,
            confirm_launches: true,
            enable_better_matchmaking: false,
            enable_better_matchmaking_randomization: false,
            force_roblox_language: false,
            background_updates_enabled: false,
            cleaner_options: 3,
            cleaner_directories: ["RobloxCache", "RobloxStudioCache", "RobloxLogs", "FishstrapLogs"].map(String::from).to_vec(),
            bootstrapper_style: 9,
            bootstrapper_icon: 1,
            bootstrapper_title: "ztmystrap".into(),
            bootstrapper_icon_custom_location: String::new(),
            roblox_icon: 0,
            roblox_title: "Roblox".into(),
            roblox_icon_custom_location: String::new(),
            theme: 0,
            use_acrylic_background: false,
            acrylic_background_opacity: 165,
            locale: "nil".into(),
            selected_custom_theme: None,
            check_for_updates: true,
            static_directory: false,
            channel: "production".into(),
            roblox_domain: "roblox.com".into(),
            channel_change_mode: channel_change_mode::AUTOMATIC,
            distributor_type: 0,
            web_environment: "Production".into(),
            debug_disable_version_package_cleanup: false,
            force_local_data: false,
            use_fast_flag_manager: true,
            apply_non_allowlisted_flags: true,
            enable_roblox_background_app: false,
            roblox_theme: 0,
            enable_window_manipulation: false,
            fake_borderless_fullscreen: false,
            enable_activity_tracking: true,
            use_discord_rich_presence: true,
            rich_presence_status_display_type: 0,
            hide_rpc_buttons: true,
            allow_discord_invites: false,
            show_account_on_rich_presence: false,
            show_server_details: false,
            custom_integrations: vec![],
            use_disable_app_patch: false,
            wpf_software_render: false,
            enable_analytics: false,
            extra: Map::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct State {
    pub force_reinstall: bool,
    pub ignore_outdated_channel: bool,
    pub last_page: Option<String>,
    pub imported_from_fishstrap: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase", default)]
pub struct AppState {
    pub version_guid: String,
    pub package_hashes: BTreeMap<String, String>,
    /// Installed size in KB.
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct RobloxState {
    pub player: AppState,
    pub studio: AppState,
    /// Relative paths currently mirrored from `Modifications\` into the version dir.
    pub mod_manifest: Vec<String>,
}

/// Load a JSON file, falling back to defaults. A corrupt file is backed up to `.bak`.
pub fn load<T: DeserializeOwned + Default>(path: &Path) -> T {
    let Ok(text) = fs::read_to_string(path) else { return T::default() };
    match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(v) => v,
        Err(e) => {
            crate::log!("settings::load", "failed to parse {}: {e}; backing up", path.display());
            let _ = fs::copy(path, path.with_extension("json.bak"));
            T::default()
        }
    }
}

pub fn save<T: Serialize>(path: &Path, value: &T) -> anyhow::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fishstrap_settings_roundtrip_preserves_unknown_fields() {
        let json = r#"{"Channel":"zcanary","ChannelChangeMode":2,"HideRPCButtons":false,"WPFSoftwareRender":true,"SomethingNew":42}"#;
        let s: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(s.channel, "zcanary");
        assert_eq!(s.channel_change_mode, channel_change_mode::IGNORE);
        assert!(!s.hide_rpc_buttons);
        assert!(s.wpf_software_render);
        assert!(s.confirm_launches, "missing fields take defaults");
        let out = serde_json::to_value(&s).unwrap();
        assert_eq!(out["SomethingNew"], 42);
        assert_eq!(out["HideRPCButtons"], false);
    }
}
