//! Editor for Roblox's own `GlobalBasicSettings_13.xml` (the in-game settings menu).
//! Values are edited in place so the rest of the file is left byte-for-byte intact.

use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const TEMPLATE: &str = include_str!("../resources/GlobalBasicSettings_Template.xml");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GlobalSettings {
    pub framerate_cap: i64,
    pub graphics_quality: i64,
    pub transparency: f64,
    pub reduced_motion: bool,
    pub font_size: i64,
    pub mouse_sensitivity: f64,
    pub vr_enabled: bool,
    pub read_only: bool,
}

fn re(tag: &str, name: &str) -> Regex {
    Regex::new(&format!(r#"(<{tag} name="{name}">)([^<]*)(</{tag}>)"#)).unwrap()
}

fn get(xml: &str, tag: &str, name: &str) -> Option<String> {
    re(tag, name).captures(xml).map(|c| c[2].trim().to_string())
}

/// Replace the value, or insert the element at the start of `<Properties>` if absent.
fn set(xml: &str, tag: &str, name: &str, value: &str) -> String {
    let r = re(tag, name);
    if r.is_match(xml) {
        r.replace(xml, |c: &regex::Captures| format!("{}{value}{}", &c[1], &c[3])).into_owned()
    } else {
        xml.replacen("<Properties>", &format!("<Properties>\n\t\t\t<{tag} name=\"{name}\">{value}</{tag}>"), 1)
    }
}

pub fn parse(xml: &str) -> GlobalSettings {
    let num = |t, n, d: f64| get(xml, t, n).and_then(|v| v.parse::<f64>().ok()).unwrap_or(d);
    GlobalSettings {
        framerate_cap: num("int", "FramerateCap", 60.0) as i64,
        graphics_quality: num("token", "SavedQualityLevel", 0.0) as i64,
        transparency: num("float", "PreferredTransparency", 1.0),
        reduced_motion: get(xml, "bool", "ReducedMotion").as_deref() == Some("true"),
        font_size: num("token", "PreferredTextSize", 1.0) as i64,
        mouse_sensitivity: num("float", "MouseSensitivity", 1.0),
        vr_enabled: get(xml, "bool", "VREnabled").as_deref() == Some("true"),
        read_only: false,
    }
}

pub fn apply(xml: &str, s: &GlobalSettings) -> String {
    let fps = if s.framerate_cap < 1 { -1 } else { s.framerate_cap };
    let mut x = set(xml, "int", "FramerateCap", &fps.to_string());
    x = set(&x, "token", "SavedQualityLevel", &s.graphics_quality.clamp(0, 10).to_string());
    let t = format!("{:.3}", s.transparency.clamp(0.0, 1.0));
    x = set(&x, "float", "PreferredTransparency", t.trim_end_matches('0').trim_end_matches('.'));
    x = set(&x, "bool", "ReducedMotion", if s.reduced_motion { "true" } else { "false" });
    x = set(&x, "token", "PreferredTextSize", &s.font_size.clamp(1, 4).to_string());
    x = set(&x, "float", "MouseSensitivity", &s.mouse_sensitivity.clamp(0.0, 10.0).to_string());
    set(&x, "bool", "VREnabled", if s.vr_enabled { "true" } else { "false" })
}

fn read_or_template(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|_| TEMPLATE.to_string())
}

pub fn load() -> GlobalSettings {
    let path = crate::paths::roblox_global_settings();
    let mut s = parse(&read_or_template(&path));
    s.read_only = fs::metadata(&path).map(|m| m.permissions().readonly()).unwrap_or(false);
    s
}

pub fn save(s: &GlobalSettings) -> Result<()> {
    let path = crate::paths::roblox_global_settings();
    if let Some(d) = path.parent() {
        fs::create_dir_all(d)?;
    }
    if path.exists() {
        crate::winutil::set_readonly(&path, false)?;
    }
    fs::write(&path, apply(&read_or_template(&path), s))?;
    if s.read_only {
        crate::winutil::set_readonly(&path, true)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_template() {
        let s = parse(TEMPLATE);
        assert_eq!(s.framerate_cap, 60);
        assert_eq!(s.graphics_quality, 2);
        let mut s2 = s.clone();
        s2.framerate_cap = 0;
        s2.reduced_motion = true;
        s2.transparency = 0.5;
        let out = apply(TEMPLATE, &s2);
        assert!(out.contains(r#"<int name="FramerateCap">-1</int>"#));
        assert!(out.contains(r#"<bool name="ReducedMotion">true</bool>"#));
        assert!(out.contains(r#"<float name="PreferredTransparency">0.5</float>"#));
        assert!(out.contains("<External>nil</External>"), "rest of file untouched");
    }

    #[test]
    fn inserts_missing_element() {
        let xml = "<roblox><Item class=\"UserGameSettings\"><Properties>\n</Properties></Item></roblox>";
        let out = set(xml, "int", "FramerateCap", "144");
        assert_eq!(get(&out, "int", "FramerateCap").as_deref(), Some("144"));
    }
}
