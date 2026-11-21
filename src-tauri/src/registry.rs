//! HKCU registry integration: URI protocol handlers, the uninstall entry, Roblox channel keys.

use anyhow::Result;
use std::path::Path;
use winreg::{RegKey, enums::HKEY_CURRENT_USER};

pub const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\ztmystrap";
pub const APIS_KEY: &str = r"Software\ztmystrap";
const PLAYER_SCHEMES: &[&str] = &["roblox", "roblox-player"];

fn hkcu() -> RegKey {
    RegKey::predef(HKEY_CURRENT_USER)
}

fn register_protocol(scheme: &str, handler: &Path, param: &str) -> Result<()> {
    let (key, _) = hkcu().create_subkey(format!(r"Software\Classes\{scheme}"))?;
    let handler = handler.display().to_string();
    key.set_value("", &"URL: Roblox Protocol")?;
    key.set_value("URL Protocol", &"")?;
    key.create_subkey("DefaultIcon")?.0.set_value("", &handler)?;
    key.create_subkey(r"shell\open\command")?.0.set_value("", &format!("\"{handler}\" {param}"))?;
    Ok(())
}

/// Point `roblox:` and `roblox-player:` at ztmystrap.
pub fn register_player(exe: &Path) -> Result<()> {
    for s in PLAYER_SCHEMES {
        register_protocol(s, exe, "-player \"%1\"")?;
    }
    Ok(())
}

/// Current command for `roblox-player:`, if any.
pub fn player_handler() -> Option<String> {
    hkcu().open_subkey(r"Software\Classes\roblox-player\shell\open\command").ok()?.get_value("").ok()
}

fn install_location(uninstall_name: &str) -> Option<String> {
    hkcu()
        .open_subkey(format!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{uninstall_name}"))
        .ok()?
        .get_value::<String, _>("InstallLocation")
        .ok()
        .filter(|s| !s.is_empty())
}

/// Hand the player protocol back: to Fishstrap if installed, else the stock Roblox
/// launcher, else remove the handler entirely.
pub fn restore_player_handler() -> Result<()> {
    if let Some(dir) = install_location("Fishstrap") {
        let exe = Path::new(&dir).join("Fishstrap.exe");
        if exe.exists() {
            for s in PLAYER_SCHEMES {
                register_protocol(s, &exe, "-player \"%1\"")?;
            }
            return Ok(());
        }
    }
    if let Some(dir) = install_location("roblox-player") {
        let exe = Path::new(&dir).join(crate::paths::PLAYER_EXE);
        for s in PLAYER_SCHEMES {
            register_protocol(s, &exe, "%1")?;
        }
        return Ok(());
    }
    for s in PLAYER_SCHEMES {
        let _ = hkcu().delete_subkey_all(format!(r"Software\Classes\{s}"));
    }
    Ok(())
}

pub fn write_uninstall_entry(exe: &Path, base: &Path) -> Result<()> {
    let (k, _) = hkcu().create_subkey(UNINSTALL_KEY)?;
    let exe = exe.display().to_string();
    k.set_value("DisplayIcon", &format!("{exe},0"))?;
    k.set_value("DisplayName", &"ztmystrap")?;
    k.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"))?;
    if k.get_value::<String, _>("InstallDate").is_err() {
        k.set_value("InstallDate", &chrono::Local::now().format("%Y%m%d").to_string())?;
    }
    k.set_value("InstallLocation", &base.display().to_string())?;
    k.set_value("NoRepair", &1u32)?;
    k.set_value("Publisher", &"Infernalitiy")?;
    k.set_value("ModifyPath", &format!("\"{exe}\" -settings"))?;
    k.set_value("QuietUninstallString", &format!("\"{exe}\" -uninstall -quiet"))?;
    k.set_value("UninstallString", &format!("\"{exe}\" -uninstall"))?;
    k.set_value("HelpLink", &"https://github.com/Infernalitiy/ztmystrap/issues")?;
    k.set_value("URLInfoAbout", &"https://github.com/Infernalitiy/ztmystrap")?;
    k.set_value("URLUpdateInfo", &"https://github.com/Infernalitiy/ztmystrap/releases")?;

    let (apis, _) = hkcu().create_subkey(APIS_KEY)?;
    apis.set_value("ApplicationPath", &exe)?;
    apis.set_value("InstallationPath", &base.display().to_string())?;
    Ok(())
}

pub fn set_estimated_size_kb(kb: u64) {
    if let Ok(k) = hkcu().open_subkey_with_flags(UNINSTALL_KEY, winreg::enums::KEY_SET_VALUE) {
        let _ = k.set_value("EstimatedSize", &(kb.min(u32::MAX as u64) as u32));
    }
}

pub fn set_player_path(path: &Path) {
    if let Ok((k, _)) = hkcu().create_subkey(APIS_KEY) {
        let _ = k.set_value("PlayerPath", &path.display().to_string());
    }
}

pub fn remove_uninstall_entry() {
    let _ = hkcu().delete_subkey_all(UNINSTALL_KEY);
    let _ = hkcu().delete_subkey_all(APIS_KEY);
}

pub fn is_installed() -> bool {
    hkcu().open_subkey(UNINSTALL_KEY).is_ok()
}

/// Mirror the enrolled channel into Roblox's own registry key, as Fishstrap does.
pub fn set_roblox_channel(domain: &str, channel: Option<&str>) {
    if let Ok((k, _)) = hkcu().create_subkey(r"SOFTWARE\ROBLOX Corporation\Environments\RobloxPlayer\Channel") {
        let _ = k.set_value(format!("www.{domain}"), &channel.unwrap_or(""));
    }
}
