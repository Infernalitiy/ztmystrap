//! The bootstrapper: resolve channel and version, install/update Roblox, apply
//! modifications (FastFlags etc.), then launch RobloxPlayerBeta.exe.

use crate::deployment::{self, Package};
use crate::launch_args::LaunchArgs;
use crate::settings::{self, RobloxState, Settings, State, channel_change_mode};
use crate::{fastflags, log, paths, registry, winutil};
use anyhow::{Context, Result, bail};
use md5::{Digest, Md5};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

const IDENT: &str = "bootstrapper";
const PARALLEL_DOWNLOADS: usize = 5;
const RETRIES: usize = 5;

/// Progress callback: status text and an optional 0..=1 fraction.
pub type Report = Arc<dyn Fn(&str, Option<f64>) + Send + Sync>;

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("ztmystrap/", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(300))
        .connect_timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("http client")
}

pub fn md5_file(path: &Path) -> Result<String> {
    let mut f = fs::File::open(path)?;
    let mut h = Md5::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

/// Resolve which channel to use, following Fishstrap's channel change modes.
fn resolve_channel(args: &LaunchArgs, s: &Settings) -> String {
    if let Some(c) = &args.channel {
        return deployment::normalize_channel(c);
    }
    let re = regex::Regex::new(r"(?i)channel:([a-zA-Z0-9\-_]+)").unwrap();
    let uri_channel = re.captures(&args.roblox_args).map(|c| deployment::normalize_channel(&c[1]));
    let current = deployment::normalize_channel(&s.channel);
    // A bare launch (no URI) never changes the user's chosen channel.
    if args.roblox_args.is_empty() {
        return current;
    }
    let enrolled = uri_channel.clone().unwrap_or_else(|| deployment::DEFAULT_CHANNEL.into());
    match s.channel_change_mode {
        channel_change_mode::AUTOMATIC => enrolled,
        channel_change_mode::PROMPT => match uri_channel {
            Some(c) if c != current => {
                if winutil::confirm(&format!("Roblox is trying to switch you to the \"{c}\" channel (you are on \"{current}\").\n\nSwitch channels?")) {
                    c
                } else {
                    current
                }
            }
            _ => current,
        },
        _ => current,
    }
}

pub async fn run(args: LaunchArgs, report: Report) -> Result<()> {
    winutil::hold_named_mutex("ztmystrap-Bootstrapper");
    let http = http_client();
    let mut s: Settings = settings::load(&paths::settings_file());
    let mut state: State = settings::load(&paths::state_file());
    let mut rstate: RobloxState = settings::load(&paths::roblox_state_file());

    if s.confirm_launches && !args.no_launch && winutil::is_process_running(paths::PLAYER_EXE) {
        if !winutil::confirm("Roblox is already running. Launching again will close the current instance.\n\nContinue?") {
            return Ok(());
        }
    }

    report("Connecting to Roblox...", None);
    let cdn = deployment::pick_cdn(&http).await?;
    log!(IDENT, "using CDN {cdn}");

    let domain = s.roblox_domain.clone();
    let mut channel = resolve_channel(&args, &s);

    let guid = if let Some(v) = &args.version {
        v.clone()
    } else {
        let cv = match deployment::client_version(&http, &domain, deployment::PLAYER_BINARY, &channel).await {
            Ok(cv) => cv,
            Err(e) if e.downcast_ref::<deployment::ChannelError>().is_some() => {
                let code = e.downcast_ref::<deployment::ChannelError>().unwrap().0;
                log!(IDENT, "channel {channel} unusable ({e}); reverting to production");
                if code != 404 && s.channel_change_mode != channel_change_mode::AUTOMATIC {
                    winutil::alert(&format!("The channel \"{channel}\" is private or restricted. Switching to production."), winutil::Icon::Info);
                }
                channel = deployment::DEFAULT_CHANNEL.into();
                deployment::client_version(&http, &domain, deployment::PLAYER_BINARY, &channel).await?
            }
            Err(e) => return Err(e),
        };
        log!(IDENT, "latest {} on {channel}: {}", cv.version, cv.client_version_upload);
        cv.client_version_upload
    };

    if s.channel != channel {
        s.channel = channel.clone();
        settings::save(&paths::settings_file(), &s)?;
    }
    registry::set_roblox_channel(&domain, (!deployment::is_default_channel(&channel)).then_some(channel.as_str()));

    let version_dir = if s.static_directory { paths::versions().join(deployment::PLAYER_BINARY) } else { paths::versions().join(&guid) };
    let exe = version_dir.join(paths::PLAYER_EXE);

    let needs_install = args.force || state.force_reinstall || rstate.player.version_guid != guid || !exe.exists();
    if needs_install {
        if winutil::is_process_running(paths::PLAYER_EXE) {
            winutil::kill_process(paths::PLAYER_EXE);
        }
        let packages = install(&http, &cdn, &channel, &guid, &version_dir, &report).await?;
        rstate.player = settings::AppState {
            version_guid: guid.clone(),
            package_hashes: packages.iter().map(|p| (p.name.clone(), p.signature.clone())).collect(),
            size: packages.iter().map(|p| p.size).sum::<u64>() / 1024,
        };
        rstate.mod_manifest.clear(); // fresh tree: nothing of ours is in it yet
        settings::save(&paths::roblox_state_file(), &rstate)?;
        if state.force_reinstall {
            state.force_reinstall = false;
            settings::save(&paths::state_file(), &state)?;
        }
        registry::set_estimated_size_kb(rstate.player.size);
        if !s.debug_disable_version_package_cleanup {
            cleanup_old(&version_dir, &rstate.player.package_hashes);
        }
    }

    report("Applying modifications...", None);
    apply_modifications(&version_dir, &s, &mut rstate)?;
    settings::save(&paths::roblox_state_file(), &rstate)?;
    registry::set_player_path(&exe);

    if args.no_launch {
        return Ok(());
    }
    report("Starting Roblox...", None);
    launch(&exe, &version_dir, &args.roblox_args, &s)
}

async fn install(http: &reqwest::Client, cdn: &str, channel: &str, guid: &str, version_dir: &Path, report: &Report) -> Result<Vec<Package>> {
    report("Fetching package manifest...", None);
    let manifest_url = deployment::cdn_location(cdn, channel, &format!("/{guid}-rbxPkgManifest.txt"));
    let text = http.get(&manifest_url).send().await?.error_for_status()?.text().await?;
    let packages: Vec<Package> = deployment::parse_manifest(&text)?
        .into_iter()
        .filter(|p| !deployment::IGNORED_PACKAGES.contains(&p.name.as_str()))
        .collect();
    if !packages.iter().any(|p| p.name == "RobloxApp.zip") {
        bail!("{guid} is not a Roblox Player build");
    }

    fs::create_dir_all(paths::downloads())?;
    let total: u64 = packages.iter().map(|p| p.packed_size).sum::<u64>().max(1);
    let done = Arc::new(AtomicU64::new(0));
    let mut sorted = packages.clone();
    sorted.sort_by(|a, b| b.packed_size.cmp(&a.packed_size));

    report("Downloading Roblox...", Some(0.0));
    let mut set = tokio::task::JoinSet::new();
    let sem = Arc::new(tokio::sync::Semaphore::new(PARALLEL_DOWNLOADS));
    for p in sorted {
        let (http, done, report, sem) = (http.clone(), done.clone(), report.clone(), sem.clone());
        let url = deployment::cdn_location(cdn, channel, &format!("/{guid}-{}", p.name));
        set.spawn(async move {
            let _permit = sem.acquire_owned().await?;
            download_package(&http, &url, &p, &done, total, &report).await.with_context(|| format!("downloading {}", p.name))
        });
    }
    while let Some(r) = set.join_next().await {
        r??;
    }

    report("Installing Roblox...", None);
    let (pkgs, dir) = (packages.clone(), version_dir.to_path_buf());
    let report2 = report.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        if dir.exists() {
            fs::remove_dir_all(&dir).context("removing previous version folder")?;
        }
        fs::create_dir_all(&dir)?;
        let n = pkgs.len() as f64;
        for (i, p) in pkgs.iter().enumerate() {
            report2("Installing Roblox...", Some(i as f64 / n));
            if p.name == "WebView2RuntimeInstaller.zip" {
                continue;
            }
            let Some(sub) = deployment::player_package_dir(&p.name) else {
                log!(IDENT, "no mapping for package {}, skipping", p.name);
                continue;
            };
            extract_zip(&paths::downloads().join(&p.signature), &dir.join(sub), None)?;
        }
        Ok(())
    })
    .await??;
    Ok(packages)
}

async fn download_package(http: &reqwest::Client, url: &str, p: &Package, done: &AtomicU64, total: u64, report: &Report) -> Result<()> {
    let dest = paths::downloads().join(&p.signature);
    let add = |n: u64| {
        let d = done.fetch_add(n, Ordering::Relaxed) + n;
        report("Downloading Roblox...", Some(d as f64 / total as f64));
    };
    if dest.exists() && md5_file(&dest).is_ok_and(|h| h.eq_ignore_ascii_case(&p.signature)) {
        add(p.packed_size);
        return Ok(());
    }
    // Reuse the stock Roblox bootstrapper's download cache if it has this package.
    let stock = paths::roblox_download_cache().join(&p.signature);
    if stock.exists() && md5_file(&stock).is_ok_and(|h| h.eq_ignore_ascii_case(&p.signature)) {
        fs::copy(&stock, &dest)?;
        add(p.packed_size);
        return Ok(());
    }
    let mut last = None;
    for attempt in 1..=RETRIES {
        let mut written = 0u64;
        let r: Result<()> = async {
            let tmp = dest.with_extension("part");
            let mut resp = http.get(url).send().await?.error_for_status()?;
            let mut f = fs::File::create(&tmp)?;
            let mut h = Md5::new();
            while let Some(chunk) = resp.chunk().await? {
                h.update(&chunk);
                f.write_all(&chunk)?;
                written += chunk.len() as u64;
                add(chunk.len() as u64);
            }
            drop(f);
            let got = hex::encode(h.finalize());
            if !got.eq_ignore_ascii_case(&p.signature) {
                let _ = fs::remove_file(&tmp);
                bail!("checksum mismatch (expected {}, got {got})", p.signature);
            }
            fs::rename(&tmp, &dest)?;
            Ok(())
        }
        .await;
        match r {
            Ok(()) => return Ok(()),
            Err(e) => {
                log!(IDENT, "{} attempt {attempt}/{RETRIES} failed: {e:#}", p.name);
                done.fetch_sub(written, Ordering::Relaxed);
                last = Some(e);
            }
        }
    }
    Err(last.unwrap())
}

/// Normalise a zip entry name (Roblox zips use backslashes) and reject traversal.
fn safe_entry_path(name: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for part in name.split(['\\', '/']) {
        match part {
            "" | "." => {}
            ".." => return None,
            p if p.contains(':') => return None,
            p => out.push(p),
        }
    }
    (!out.as_os_str().is_empty()).then_some(out)
}

/// Extract a zip into `dest`. If `only` is given, extract just those entries
/// (relative paths, `/`-separated, compared case-insensitively).
fn extract_zip(zip_path: &Path, dest: &Path, only: Option<&HashSet<String>>) -> Result<usize> {
    let file = fs::File::open(zip_path).with_context(|| format!("opening {}", zip_path.display()))?;
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file))?;
    let mut n = 0;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let Some(rel) = safe_entry_path(entry.name()) else { continue };
        if let Some(only) = only {
            let key = rel.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
            if !only.contains(&key) {
                continue;
            }
        }
        let out = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = fs::File::create(&out)?;
        std::io::copy(&mut entry, &mut f)?;
        n += 1;
    }
    Ok(n)
}

fn cleanup_old(current: &Path, keep: &BTreeMap<String, String>) {
    if let Ok(rd) = fs::read_dir(paths::versions()) {
        for e in rd.flatten() {
            let p = e.path();
            if p != current && p.is_dir() && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("version-")) {
                let _ = fs::remove_dir_all(&p);
            }
        }
    }
    let keep: HashSet<&str> = keep.values().map(String::as_str).collect();
    if let Ok(rd) = fs::read_dir(paths::downloads()) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if !keep.contains(name.as_str()) {
                let _ = fs::remove_file(e.path());
            }
        }
    }
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, base, out);
        } else if let Ok(rel) = p.strip_prefix(base) {
            out.push(rel.to_string_lossy().replace('/', "\\"));
        }
    }
}

/// Mirror `Modifications\` into the version folder and restore any files that
/// were modded previously but have since been removed.
pub fn apply_modifications(version_dir: &Path, s: &Settings, rstate: &mut RobloxState) -> Result<()> {
    const FFLAG_FILE: &str = "ClientSettings\\ClientAppSettings.json";
    let mods = paths::modifications();
    fs::create_dir_all(&mods)?;
    if s.use_fast_flag_manager {
        fastflags::write_client_settings(&fastflags::FlagStore::load(), s.apply_non_allowlisted_flags)?;
    }

    let mut files = Vec::new();
    walk(&mods, &mods, &mut files);
    files.retain(|rel| {
        let l = rel.to_ascii_lowercase();
        !(l.ends_with(".lock") || l.ends_with("readme.txt") || (!s.use_fast_flag_manager && rel.eq_ignore_ascii_case(FFLAG_FILE)))
    });

    for rel in &files {
        let (src, dst) = (mods.join(rel), version_dir.join(rel));
        if dst.exists() && md5_file(&src).ok() == md5_file(&dst).ok() {
            continue;
        }
        if let Some(p) = dst.parent() {
            fs::create_dir_all(p)?;
        }
        fs::copy(&src, &dst).with_context(|| format!("copying mod file {rel}"))?;
    }

    if !files.iter().any(|f| f.eq_ignore_ascii_case("AppSettings.xml")) {
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<Settings>\r\n\t<ContentFolder>content</ContentFolder>\r\n\t<BaseUrl>http://www.{}</BaseUrl>\r\n</Settings>\r\n",
            s.roblox_domain
        );
        fs::write(version_dir.join("AppSettings.xml"), xml)?;
    }

    let current: HashSet<String> = files.iter().map(|f| f.to_ascii_lowercase()).collect();
    for old in rstate.mod_manifest.iter().filter(|o| !current.contains(&o.to_ascii_lowercase())) {
        restore_original(version_dir, old, &rstate.player.package_hashes);
    }
    rstate.mod_manifest = files;
    Ok(())
}

/// Put back Roblox's own copy of a file we previously overwrote, or delete it.
fn restore_original(version_dir: &Path, rel: &str, packages: &BTreeMap<String, String>) {
    let rel_fwd = rel.replace('\\', "/");
    let dst = version_dir.join(rel);
    // Longest matching package directory first.
    let mut candidates: Vec<(&str, &String)> = packages
        .iter()
        .filter_map(|(name, sig)| deployment::player_package_dir(name).map(|d| (d, sig)))
        .filter(|(d, _)| rel_fwd.to_ascii_lowercase().starts_with(&d.to_ascii_lowercase()))
        .collect();
    candidates.sort_by_key(|(d, _)| std::cmp::Reverse(d.len()));
    for (dir, sig) in candidates {
        let inner = rel_fwd[dir.len()..].to_ascii_lowercase();
        let only: HashSet<String> = [inner].into();
        let zip = paths::downloads().join(sig);
        if zip.exists() {
            if let Ok(n) = extract_zip(&zip, &version_dir.join(dir), Some(&only)) {
                if n > 0 {
                    log!(IDENT, "restored {rel} from package");
                    return;
                }
            }
        }
    }
    let _ = fs::remove_file(&dst);
    log!(IDENT, "removed {rel} (not part of any package)");
}

fn launch(exe: &Path, dir: &Path, roblox_args: &str, s: &Settings) -> Result<()> {
    use std::os::windows::process::CommandExt;
    let mut cmd_line = roblox_args.to_string();
    if s.force_roblox_language {
        let re = regex::Regex::new(r"gameLocale:([a-z_]+)").unwrap();
        if let Some(loc) = re.captures(&cmd_line).map(|c| c[1].to_string()) {
            cmd_line = regex::Regex::new(r"robloxLocale:[a-z_]+").unwrap().replace(&cmd_line, format!("robloxLocale:{loc}")).into_owned();
        }
    }
    if !s.roblox_domain.eq_ignore_ascii_case(deployment::DEFAULT_DOMAIN) && cmd_line.is_empty() {
        cmd_line = "roblox://navigation/home".into();
    }
    log!(IDENT, "starting {} {cmd_line}", exe.display());
    let mut cmd = std::process::Command::new(exe);
    cmd.current_dir(dir);
    if !cmd_line.is_empty() {
        cmd.raw_arg(&cmd_line);
    }
    cmd.spawn().context("starting Roblox")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_paths() {
        assert_eq!(safe_entry_path("content\\fonts\\a.ttf"), Some(PathBuf::from("content").join("fonts").join("a.ttf")));
        assert_eq!(safe_entry_path("..\\evil.exe"), None);
        assert_eq!(safe_entry_path("C:\\evil.exe"), None);
        assert_eq!(safe_entry_path("\\"), None);
    }

    #[test]
    fn channel_resolution() {
        let mut s = Settings::default();
        s.channel = "zcanary".into();
        let bare = LaunchArgs { roblox_args: String::new(), ..Default::default() };
        assert_eq!(resolve_channel(&bare, &s), "zcanary", "bare launch keeps channel");
        let uri = LaunchArgs { roblox_args: "roblox-player:1+launchmode:play+channel:zintegration".into(), ..Default::default() };
        assert_eq!(resolve_channel(&uri, &s), "zintegration");
        let no_ch = LaunchArgs { roblox_args: "roblox-player:1+launchmode:play".into(), ..Default::default() };
        assert_eq!(resolve_channel(&no_ch, &s), "production");
        s.channel_change_mode = channel_change_mode::IGNORE;
        assert_eq!(resolve_channel(&uri, &s), "zcanary");
        let forced = LaunchArgs { channel: Some("ZLive".into()), ..Default::default() };
        assert_eq!(resolve_channel(&forced, &s), "production");
    }
}
