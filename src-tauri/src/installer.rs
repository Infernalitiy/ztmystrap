//! Installing / uninstalling ztmystrap itself, and importing from Fishstrap.

use crate::settings::{self, Settings, State};
use crate::{fastflags, log, paths, registry, winutil};
use anyhow::{Context, Result};
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

const IDENT: &str = "installer";

fn shortcut_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        v.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\ztmystrap.lnk"));
    }
    if let Some(home) = std::env::var_os("USERPROFILE") {
        v.push(PathBuf::from(home).join(r"Desktop\ztmystrap.lnk"));
    }
    v
}

fn create_shortcut(lnk: &Path, target: &Path) {
    let ps = format!(
        "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Save()",
        lnk.display().to_string().replace('\'', "''"),
        target.display().to_string().replace('\'', "''"),
        target.parent().unwrap().display().to_string().replace('\'', "''"),
    );
    let _ = std::process::Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &ps]).creation_flags(0x0800_0000).output();
}

pub fn running_from_install() -> bool {
    std::env::current_exe().ok().and_then(|p| p.canonicalize().ok()) == paths::installed_exe().canonicalize().ok()
}

/// Copy this exe into `%LOCALAPPDATA%\ztmystrap`, register it and create shortcuts.
pub fn install() -> Result<()> {
    let base = paths::base();
    fs::create_dir_all(&base)?;
    let target = paths::installed_exe();
    if !running_from_install() {
        let me = std::env::current_exe()?;
        fs::copy(&me, &target).with_context(|| format!("copying to {}", target.display()))?;
    }
    registry::write_uninstall_entry(&target, &base)?;
    registry::register_player(&target)?;
    for lnk in shortcut_paths() {
        create_shortcut(&lnk, &target);
    }
    if !paths::settings_file().exists() {
        settings::save(&paths::settings_file(), &Settings::default())?;
    }
    log!(IDENT, "installed to {}", base.display());
    Ok(())
}

pub fn uninstall(keep_data: bool) -> Result<()> {
    if winutil::is_process_running(paths::PLAYER_EXE) {
        winutil::kill_process(paths::PLAYER_EXE);
    }
    registry::restore_player_handler()?;
    for lnk in shortcut_paths() {
        let _ = fs::remove_file(lnk);
    }
    let base = paths::base();
    for d in ["Versions", "Downloads"] {
        let _ = fs::remove_dir_all(base.join(d));
    }
    let _ = fs::remove_file(paths::state_file());
    let _ = fs::remove_file(paths::roblox_state_file());
    registry::remove_uninstall_entry();
    if keep_data {
        winutil::delete_after_exit(&paths::installed_exe(), false);
    } else {
        winutil::delete_after_exit(&base, true);
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path, skip: &dyn Fn(&Path) -> bool) -> Result<usize> {
    let mut n = 0;
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)?.flatten() {
        let (p, d) = (e.path(), dst.join(e.file_name()));
        if skip(&p) {
            continue;
        }
        if p.is_dir() {
            n += copy_dir(&p, &d, skip)?;
        } else {
            fs::copy(&p, &d)?;
            n += 1;
        }
    }
    Ok(n)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub settings: bool,
    pub flags: usize,
    pub mod_files: usize,
}

/// Import Settings.json, FastFlags and Modifications from an existing Fishstrap install.
pub fn import_fishstrap() -> Result<ImportResult> {
    let src = paths::fishstrap_base();
    let mut res = ImportResult { settings: false, flags: 0, mod_files: 0 };

    let fs_settings = src.join("Settings.json");
    if fs_settings.exists() {
        let mut s: Settings = settings::load(&fs_settings);
        if s.bootstrapper_title.eq_ignore_ascii_case("Fishstrap") {
            s.bootstrapper_title = "ztmystrap".into();
        }
        settings::save(&paths::settings_file(), &s)?;
        res.settings = true;
    }

    let fs_flags = src.join(r"Modifications\ClientSettings\ClientAppSettings.json");
    if let Ok(text) = fs::read_to_string(&fs_flags) {
        let mut store = fastflags::FlagStore::load();
        res.flags = store.merge(fastflags::parse_import(&text)?);
        store.save()?;
    }

    let fs_mods = src.join("Modifications");
    if fs_mods.is_dir() {
        let skip_flags = fs_mods.join("ClientSettings").join("ClientAppSettings.json");
        res.mod_files = copy_dir(&fs_mods, &paths::modifications(), &|p| p == skip_flags)?;
    }

    let mut state: State = settings::load(&paths::state_file());
    state.imported_from_fishstrap = true;
    settings::save(&paths::state_file(), &state)?;
    log!(IDENT, "imported from Fishstrap: settings={} flags={} mods={}", res.settings, res.flags, res.mod_files);
    Ok(res)
}
