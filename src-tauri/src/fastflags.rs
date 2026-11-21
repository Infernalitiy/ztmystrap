//! FastFlag store. ztmystrap keeps every flag the user has added (with an on/off toggle)
//! in `FastFlags.json`, and generates Roblox's `ClientAppSettings.json` from the enabled
//! ones on each launch. Flags are split into Roblox-allowlisted and non-allowlisted.

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::{BTreeMap, HashSet}, fs, sync::OnceLock, time::Duration};

const BUNDLED_ALLOWLIST: &str = include_str!("../resources/allowlist.json");
const REMOTE_CONFIG_URL: &str = "https://raw.githubusercontent.com/Infernalitiy/ztmystrap/main/remote-config.json";
const CATALOG_URL: &str = "https://raw.githubusercontent.com/MaximumADHD/Roblox-Client-Tracker/roblox/FVariables.txt";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowlistEntry {
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allowlist {
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub retrieved: String,
    pub flags: Vec<AllowlistEntry>,
}

static ALLOWLIST: OnceLock<std::sync::RwLock<Allowlist>> = OnceLock::new();

fn allowlist_lock() -> &'static std::sync::RwLock<Allowlist> {
    ALLOWLIST.get_or_init(|| {
        let cached = fs::read_to_string(crate::paths::base().join("Allowlist.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
        std::sync::RwLock::new(cached.unwrap_or_else(|| serde_json::from_str(BUNDLED_ALLOWLIST).expect("bundled allowlist is valid")))
    })
}

pub fn allowlist() -> Allowlist {
    allowlist_lock().read().unwrap().clone()
}

pub fn is_allowlisted(name: &str) -> bool {
    allowlist_lock().read().unwrap().flags.iter().any(|f| f.name == name)
}

/// Refresh the allowlist from ztmystrap's remote config (`{"allowlist": {...}}`).
pub async fn refresh_allowlist(http: &reqwest::Client) -> Result<Allowlist> {
    #[derive(Deserialize)]
    struct Remote {
        allowlist: Allowlist,
    }
    let r: Remote = http.get(REMOTE_CONFIG_URL).timeout(Duration::from_secs(10)).send().await?.error_for_status()?.json().await?;
    if r.allowlist.flags.is_empty() {
        bail!("remote allowlist is empty");
    }
    let _ = crate::settings::save(&crate::paths::base().join("Allowlist.json"), &r.allowlist);
    *allowlist_lock().write().unwrap() = r.allowlist.clone();
    Ok(r.allowlist)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct FlagEntry {
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct FlagStore {
    pub flags: BTreeMap<String, FlagEntry>,
}

impl FlagStore {
    pub fn load() -> Self {
        let mut s: FlagStore = crate::settings::load(&crate::paths::fastflags_file());
        // Fishstrap always forces this off; keep parity unless the user set it.
        s.flags.entry("FFlagHandleAltEnterFullscreenManually".into()).or_insert(FlagEntry { value: "False".into(), enabled: true });
        s
    }

    pub fn save(&self) -> Result<()> {
        crate::settings::save(&crate::paths::fastflags_file(), self)
    }

    /// The flags that should be written to `ClientAppSettings.json`.
    pub fn effective(&self, include_non_allowlisted: bool) -> Map<String, Value> {
        self.flags
            .iter()
            .filter(|(n, e)| e.enabled && (include_non_allowlisted || is_allowlisted(n)))
            .map(|(n, e)| (n.clone(), Value::String(e.value.clone())))
            .collect()
    }

    /// Merge flags (enabled) into the store, returning how many were added/changed.
    pub fn merge(&mut self, incoming: BTreeMap<String, String>) -> usize {
        let mut n = 0;
        for (k, v) in incoming {
            let e = FlagEntry { value: v, enabled: true };
            if self.flags.get(&k) != Some(&e) {
                self.flags.insert(k, e);
                n += 1;
            }
        }
        n
    }
}

/// Write the generated `ClientAppSettings.json` into `Modifications\ClientSettings`.
pub fn write_client_settings(store: &FlagStore, include_non_allowlisted: bool) -> Result<()> {
    let path = crate::paths::client_app_settings();
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, serde_json::to_string_pretty(&store.effective(include_non_allowlisted))?)?;
    Ok(())
}

fn value_to_flag_string(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(if *b { "True".into() } else { "False".into() }),
        other => Some(other.to_string()),
    }
}

/// Lenient JSON import, like Fishstrap: tolerates missing braces, `//` and `/* */`
/// comments and trailing commas. Null values are skipped.
pub fn parse_import(text: &str) -> Result<BTreeMap<String, String>> {
    let mut s = strip_comments(text.trim());
    if !s.starts_with('{') {
        s.insert(0, '{');
    }
    if !s.ends_with('}') {
        s.push('}');
    }
    let re = regex::Regex::new(r",(\s*[}\]])").unwrap();
    let s = re.replace_all(&s, "$1");
    let map: Map<String, Value> = serde_json::from_str(&s)?;
    Ok(map.into_iter().filter_map(|(k, v)| value_to_flag_string(&v).map(|v| (k.trim().to_string(), v))).collect())
}

fn strip_comments(s: &str) -> String {
    let (mut out, mut chars, mut in_str) = (String::with_capacity(s.len()), s.chars().peekable(), false);
    while let Some(c) = chars.next() {
        if in_str {
            out.push(c);
            if c == '\\' {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match (c, chars.peek()) {
            ('"', _) => {
                in_str = true;
                out.push(c)
            }
            ('/', Some('/')) => {
                while chars.next().is_some_and(|c| c != '\n') {}
                out.push('\n');
            }
            ('/', Some('*')) => {
                chars.next();
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// Flag names must be alphanumeric/underscore with a known type prefix.
pub fn validate_name(name: &str) -> bool {
    const PREFIXES: &[&str] = &["FFlag", "DFFlag", "SFFlag", "FInt", "DFInt", "SFInt", "FString", "DFString", "SFString", "FLog", "DFLog", "SFLog"];
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && PREFIXES.iter().any(|p| name.starts_with(p))
}

/// Every known client flag name, from the Roblox-Client-Tracker FVariables list.
/// Cached on disk for a day.
pub async fn catalog(http: &reqwest::Client) -> Result<Vec<String>> {
    let cache = crate::paths::flag_catalog_cache();
    let fresh = fs::metadata(&cache).and_then(|m| m.modified()).map(|t| t.elapsed().unwrap_or_default() < Duration::from_secs(86_400)).unwrap_or(false);
    let text = if fresh {
        fs::read_to_string(&cache)?
    } else {
        match async { anyhow::Ok(http.get(CATALOG_URL).send().await?.error_for_status()?.text().await?) }.await {
            Ok(t) => {
                let _ = fs::create_dir_all(crate::paths::base());
                let _ = fs::write(&cache, &t);
                t
            }
            Err(e) if cache.exists() => {
                crate::log!("fastflags::catalog", "refresh failed ({e}); using stale cache");
                fs::read_to_string(&cache)?
            }
            Err(e) => return Err(e),
        }
    };
    let mut seen = HashSet::new();
    Ok(text
        .lines()
        .filter_map(|l| l.split_once("] ").map(|(_, n)| n.trim().to_string()).or_else(|| Some(l.trim().to_string())))
        .filter(|n| validate_name(n) && seen.insert(n.clone()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenient_import() {
        let m = parse_import("\"FIntA\": 5, // comment\n\"FFlagB\": true, /* x */ \"DFStringC\": \"hi\", \"FIntD\": null,").unwrap();
        assert_eq!(m["FIntA"], "5");
        assert_eq!(m["FFlagB"], "True");
        assert_eq!(m["DFStringC"], "hi");
        assert!(!m.contains_key("FIntD"));
    }

    #[test]
    fn comment_markers_inside_strings_survive() {
        let m = parse_import(r#"{"FStringUrl": "https://x//y"}"#).unwrap();
        assert_eq!(m["FStringUrl"], "https://x//y");
    }

    #[test]
    fn effective_respects_toggles_and_allowlist() {
        let mut s = FlagStore::default();
        s.flags.insert("FIntDebugForceMSAASamples".into(), FlagEntry { value: "4".into(), enabled: true });
        s.flags.insert("FFlagNotOnList".into(), FlagEntry { value: "True".into(), enabled: true });
        s.flags.insert("DFFlagDisableDPIScale".into(), FlagEntry { value: "True".into(), enabled: false });
        let all = s.effective(true);
        assert_eq!(all.len(), 2);
        let safe = s.effective(false);
        assert_eq!(safe.len(), 1);
        assert!(safe.contains_key("FIntDebugForceMSAASamples"));
    }

    #[test]
    fn names() {
        assert!(validate_name("DFIntTaskSchedulerTargetFps"));
        assert!(!validate_name("TaskSchedulerTargetFps"));
        assert!(!validate_name("FFlag Bad"));
    }

    #[test]
    fn bundled_allowlist_parses() {
        let a: Allowlist = serde_json::from_str(BUNDLED_ALLOWLIST).unwrap();
        assert!(a.flags.len() >= 18);
    }
}
