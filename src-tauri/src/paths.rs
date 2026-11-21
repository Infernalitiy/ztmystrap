//! Well-known filesystem locations used by ztmystrap and by Roblox itself.

use std::path::PathBuf;

pub const APP_NAME: &str = "ztmystrap";
pub const EXE_NAME: &str = "ztmystrap.exe";
pub const PLAYER_EXE: &str = "RobloxPlayerBeta.exe";

fn local_appdata() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir())
}

/// `%LOCALAPPDATA%\ztmystrap` (or the `ZTMYSTRAP_HOME` override, used by tests).
pub fn base() -> PathBuf {
    if let Some(p) = std::env::var_os("ZTMYSTRAP_HOME") {
        return PathBuf::from(p);
    }
    local_appdata().join(APP_NAME)
}

pub fn installed_exe() -> PathBuf {
    base().join(EXE_NAME)
}
pub fn settings_file() -> PathBuf {
    base().join("Settings.json")
}
pub fn state_file() -> PathBuf {
    base().join("State.json")
}
pub fn roblox_state_file() -> PathBuf {
    base().join("RobloxState.json")
}
pub fn fastflags_file() -> PathBuf {
    base().join("FastFlags.json")
}
pub fn versions() -> PathBuf {
    base().join("Versions")
}
pub fn downloads() -> PathBuf {
    base().join("Downloads")
}
pub fn logs() -> PathBuf {
    base().join("Logs")
}
pub fn modifications() -> PathBuf {
    base().join("Modifications")
}
/// The generated FFlag file that gets mirrored into the Roblox version folder.
pub fn client_app_settings() -> PathBuf {
    modifications().join("ClientSettings").join("ClientAppSettings.json")
}
pub fn flag_catalog_cache() -> PathBuf {
    base().join("FlagCatalog.txt")
}

/// `%LOCALAPPDATA%\Roblox`
pub fn roblox_data() -> PathBuf {
    local_appdata().join("Roblox")
}
pub fn roblox_global_settings() -> PathBuf {
    roblox_data().join("GlobalBasicSettings_13.xml")
}
pub fn roblox_download_cache() -> PathBuf {
    roblox_data().join("Downloads")
}

/// The existing Fishstrap install, for importing settings.
pub fn fishstrap_base() -> PathBuf {
    local_appdata().join("Fishstrap")
}
