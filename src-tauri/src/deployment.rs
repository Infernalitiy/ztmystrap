//! Roblox deployment: CDN selection, version lookup, package manifests and package maps.

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub const DEFAULT_CHANNEL: &str = "production";
pub const DEFAULT_DOMAIN: &str = "roblox.com";
pub const PLAYER_BINARY: &str = "WindowsPlayer";
const VERSION_STUDIO_HASH: &str = "version-012732894899482c";

/// CDN base URLs with the delay (seconds) before each is tried.
const CDNS: &[(&str, u64)] = &[
    ("https://setup.rbxcdn.com", 0),
    ("https://setup-aws.rbxcdn.com", 2),
    ("https://setup-ak.rbxcdn.com", 2),
    ("https://s3.amazonaws.com/setup.roblox.com", 4),
];

pub fn is_default_channel(channel: &str) -> bool {
    channel.eq_ignore_ascii_case(DEFAULT_CHANNEL) || channel.eq_ignore_ascii_case("live")
}

/// Normalise user input the way Fishstrap does ("live"/"zlive" mean production).
pub fn normalize_channel(channel: &str) -> String {
    let c = channel.trim().to_ascii_lowercase();
    if c.is_empty() || c == "live" || c == "zlive" { DEFAULT_CHANNEL.into() } else { c }
}

/// Race the CDN mirrors and return the first healthy one.
pub async fn pick_cdn(http: &reqwest::Client) -> Result<String> {
    let futs = CDNS.iter().map(|&(url, delay)| {
        let http = http.clone();
        Box::pin(async move {
            tokio::time::sleep(Duration::from_secs(delay)).await;
            let body = http.get(format!("{url}/versionStudio")).send().await?.error_for_status()?.text().await?;
            if body.trim() == VERSION_STUDIO_HASH { Ok(url.to_string()) } else { Err(anyhow!("{url} returned unexpected body")) }
        })
    });
    match futures::future::select_ok(futs).await {
        Ok((url, _rest)) => Ok(url),
        Err(e) => Err(e.context("no Roblox CDN mirror is reachable")),
    }
}

/// Build a CDN URL for a resource like `/version-abc-rbxPkgManifest.txt`.
pub fn cdn_location(cdn: &str, channel: &str, resource: &str) -> String {
    let mut s = cdn.to_string();
    if !is_default_channel(channel) {
        s.push_str("/channel/common");
    }
    s.push_str(resource);
    s
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientVersion {
    pub version: String,
    pub client_version_upload: String,
    #[serde(default)]
    pub bootstrapper_version: String,
}

/// Returned when a channel lookup fails with 401/403/404.
#[derive(Debug)]
pub struct ChannelError(pub u16);
impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            404 => write!(f, "channel does not exist"),
            _ => write!(f, "channel is private or unauthorized (HTTP {})", self.0),
        }
    }
}
impl std::error::Error for ChannelError {}

/// `GET clientsettingscdn.<domain>/v2/client-version/<binary>[/channel/<c>]` with a
/// fallback to `clientsettings.<domain>`.
pub async fn client_version(http: &reqwest::Client, domain: &str, binary: &str, channel: &str) -> Result<ClientVersion> {
    let mut path = format!("v2/client-version/{binary}");
    if !is_default_channel(channel) {
        path.push_str(&format!("/channel/{channel}"));
    }
    let mut last_err = None;
    for host in ["clientsettingscdn", "clientsettings"] {
        let url = format!("https://{host}.{domain}/{path}");
        match http.get(&url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if matches!(status, 401 | 403 | 404) {
                    return Err(ChannelError(status).into());
                }
                match resp.error_for_status() {
                    Ok(r) => return r.json::<ClientVersion>().await.with_context(|| format!("bad JSON from {url}")),
                    Err(e) => last_err = Some(anyhow::Error::from(e)),
                }
            }
            Err(e) => last_err = Some(e.into()),
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow!("client-version lookup failed")))
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Package {
    pub name: String,
    pub signature: String,
    pub packed_size: u64,
    pub size: u64,
}

/// Parse an `rbxPkgManifest.txt`: `v0`, then 4 lines per package.
pub fn parse_manifest(text: &str) -> Result<Vec<Package>> {
    let mut lines = text.lines().map(str::trim);
    if lines.next() != Some("v0") {
        bail!("unexpected package manifest version");
    }
    let mut out = Vec::new();
    loop {
        let Some(name) = lines.next() else { break };
        if name.is_empty() || name == "RobloxPlayerLauncher.exe" {
            break;
        }
        let (Some(sig), Some(packed), Some(size)) = (lines.next(), lines.next(), lines.next()) else {
            bail!("truncated package manifest at {name}");
        };
        out.push(Package { name: name.into(), signature: sig.into(), packed_size: packed.parse()?, size: size.parse()? });
    }
    Ok(out)
}

pub const IGNORED_PACKAGES: &[&str] = &["RobloxPlayerInstaller.exe"];

/// Map a package name to the subdirectory (inside the version dir) it extracts into.
pub fn player_package_dir(name: &str) -> Option<&'static str> {
    Some(match name {
        "RobloxApp.zip" | "Libraries.zip" | "redist.zip" | "WebView2.zip" => "",
        "shaders.zip" => "shaders/",
        "ssl.zip" => "ssl/",
        "WebView2RuntimeInstaller.zip" => "WebView2RuntimeInstaller/",
        "content-avatar.zip" => "content/avatar/",
        "content-configs.zip" => "content/configs/",
        "content-fonts.zip" => "content/fonts/",
        "content-sky.zip" => "content/sky/",
        "content-sounds.zip" => "content/sounds/",
        "content-textures2.zip" => "content/textures/",
        "content-models.zip" => "content/models/",
        "content-textures3.zip" => "PlatformContent/pc/textures/",
        "content-terrain.zip" => "PlatformContent/pc/terrain/",
        "content-platform-fonts.zip" => "PlatformContent/pc/fonts/",
        "content-platform-dictionaries.zip" => "PlatformContent/pc/shared_compression_dictionaries/",
        "extracontent-luapackages.zip" => "ExtraContent/LuaPackages/",
        "extracontent-translations.zip" => "ExtraContent/translations/",
        "extracontent-models.zip" => "ExtraContent/models/",
        "extracontent-textures.zip" => "ExtraContent/textures/",
        "extracontent-places.zip" => "ExtraContent/places/",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest() {
        let m = "v0\nRobloxApp.zip\nabc123\n100\n200\nshaders.zip\ndef456\n10\n20\nRobloxPlayerLauncher.exe\nx\n1\n1\n";
        let p = parse_manifest(m).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0], Package { name: "RobloxApp.zip".into(), signature: "abc123".into(), packed_size: 100, size: 200 });
        assert_eq!(p[1].name, "shaders.zip");
    }

    #[test]
    fn rejects_bad_manifest() {
        assert!(parse_manifest("v1\n").is_err());
        assert!(parse_manifest("v0\nRobloxApp.zip\nabc\n").is_err());
    }

    #[test]
    fn channel_rules() {
        assert_eq!(normalize_channel(" ZLive "), "production");
        assert_eq!(normalize_channel("zCanary"), "zcanary");
        assert_eq!(cdn_location("https://c", "production", "/x"), "https://c/x");
        assert_eq!(cdn_location("https://c", "zcanary", "/x"), "https://c/channel/common/x");
    }

    #[test]
    fn package_dirs() {
        assert_eq!(player_package_dir("content-textures3.zip"), Some("PlatformContent/pc/textures/"));
        assert_eq!(player_package_dir("RobloxApp.zip"), Some(""));
        assert_eq!(player_package_dir("mystery.zip"), None);
    }
}
