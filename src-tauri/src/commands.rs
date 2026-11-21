//! Tauri commands exposed to the Svelte frontend.

use crate::settings::{self, Settings, State};
use crate::{bootstrapper, deployment, fastflags, global_settings, installer, paths, registry};
use serde::Serialize;
use std::os::windows::process::CommandExt;

type CmdResult<T> = Result<T, String>;

fn err<E: std::fmt::Display>(e: E) -> String {
    format!("{e:#}")
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    version: &'static str,
    installed: bool,
    running_from_install: bool,
    fishstrap_present: bool,
    handler_is_ours: bool,
    imported_from_fishstrap: bool,
    base_dir: String,
}

#[tauri::command]
pub fn app_info() -> AppInfo {
    let state: State = settings::load(&paths::state_file());
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        installed: registry::is_installed(),
        running_from_install: installer::running_from_install(),
        fishstrap_present: paths::fishstrap_base().join("Settings.json").exists(),
        handler_is_ours: registry::player_handler().is_some_and(|h| h.to_ascii_lowercase().contains("ztmystrap.exe")),
        imported_from_fishstrap: state.imported_from_fishstrap,
        base_dir: paths::base().display().to_string(),
    }
}

#[tauri::command]
pub fn install_app() -> CmdResult<()> {
    installer::install().map_err(err)
}

#[tauri::command]
pub fn import_fishstrap() -> CmdResult<installer::ImportResult> {
    installer::import_fishstrap().map_err(err)
}

#[tauri::command]
pub fn get_settings() -> Settings {
    settings::load(&paths::settings_file())
}

#[tauri::command]
pub fn save_settings(value: Settings) -> CmdResult<()> {
    settings::save(&paths::settings_file(), &value).map_err(err)
}

#[tauri::command]
pub fn get_state() -> State {
    settings::load(&paths::state_file())
}

#[tauri::command]
pub fn save_state(value: State) -> CmdResult<()> {
    settings::save(&paths::state_file(), &value).map_err(err)
}

#[derive(Serialize)]
pub struct FlagsPayload {
    store: fastflags::FlagStore,
    allowlist: fastflags::Allowlist,
}

#[tauri::command]
pub fn get_flags() -> FlagsPayload {
    FlagsPayload { store: fastflags::FlagStore::load(), allowlist: fastflags::allowlist() }
}

#[tauri::command]
pub fn save_flags(store: fastflags::FlagStore) -> CmdResult<()> {
    store.save().map_err(err)?;
    let s: Settings = settings::load(&paths::settings_file());
    fastflags::write_client_settings(&store, s.apply_non_allowlisted_flags).map_err(err)
}

#[tauri::command]
pub fn parse_flag_import(text: String) -> CmdResult<std::collections::BTreeMap<String, String>> {
    fastflags::parse_import(&text).map_err(err)
}

#[tauri::command]
pub async fn flag_catalog() -> CmdResult<Vec<String>> {
    fastflags::catalog(&bootstrapper::http_client()).await.map_err(err)
}

#[tauri::command]
pub async fn refresh_allowlist() -> CmdResult<fastflags::Allowlist> {
    fastflags::refresh_allowlist(&bootstrapper::http_client()).await.map_err(err)
}

#[tauri::command]
pub fn get_global_settings() -> global_settings::GlobalSettings {
    global_settings::load()
}

#[tauri::command]
pub fn save_global_settings(value: global_settings::GlobalSettings) -> CmdResult<()> {
    global_settings::save(&value).map_err(err)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelInfo {
    channel: String,
    version: String,
    guid: String,
}

#[tauri::command]
pub async fn channel_info(channel: String, domain: String) -> CmdResult<ChannelInfo> {
    let channel = deployment::normalize_channel(&channel);
    let cv = deployment::client_version(&bootstrapper::http_client(), &domain, deployment::PLAYER_BINARY, &channel).await.map_err(err)?;
    Ok(ChannelInfo { channel, version: cv.version, guid: cv.client_version_upload })
}

#[tauri::command]
pub fn launch_roblox() -> CmdResult<()> {
    let exe = if paths::installed_exe().exists() { paths::installed_exe() } else { std::env::current_exe().map_err(err)? };
    std::process::Command::new(exe).arg("-player").spawn().map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn open_folder(which: String) -> CmdResult<()> {
    let p = match which.as_str() {
        "base" => paths::base(),
        "mods" => paths::modifications(),
        "logs" => paths::logs(),
        "versions" => paths::versions(),
        "roblox" => paths::roblox_data(),
        _ => return Err("unknown folder".into()),
    };
    let _ = std::fs::create_dir_all(&p);
    std::process::Command::new("explorer").arg(&p).creation_flags(0x0800_0000).spawn().map_err(err)?;
    Ok(())
}

#[tauri::command]
pub fn open_url(url: String) -> CmdResult<()> {
    if !(url.starts_with("https://")) {
        return Err("only https links".into());
    }
    std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", &url]).spawn().map_err(err)?;
    Ok(())
}
