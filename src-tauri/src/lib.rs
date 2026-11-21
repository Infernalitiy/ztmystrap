//! ztmystrap: a Roblox bootstrapper. Credits: Bloxstrap and Fishstrap (MIT).

pub mod bootstrapper;
mod commands;
pub mod deployment;
pub mod fastflags;
pub mod global_settings;
pub mod installer;
pub mod launch_args;
pub mod log;
pub mod paths;
pub mod registry;
pub mod settings;
pub mod winutil;

use launch_args::{LaunchArgs, Mode};
use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

#[derive(Clone, Serialize)]
struct ProgressEvent {
    status: String,
    value: Option<f64>,
}

pub fn run() {
    log::init();
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = LaunchArgs::parse(&argv);
    log!("main", "ztmystrap {} args={argv:?} mode={:?}", env!("CARGO_PKG_VERSION"), args.mode);

    if args.mode == Mode::Uninstall {
        let go = args.quiet || winutil::confirm("Uninstall ztmystrap? Roblox installed by ztmystrap will be removed.");
        if go {
            let keep = !args.quiet && winutil::confirm("Keep your settings, FastFlags and mods (for a future reinstall)?");
            if let Err(e) = installer::uninstall(keep) {
                winutil::alert(&format!("Uninstall failed: {e:#}"), winutil::Icon::Error);
            }
        }
        return;
    }

    // Launching Roblox before ztmystrap is installed shows the installer first.
    // (`ZTMYSTRAP_HOME` is a sandbox override used for testing.)
    let bootstrap = args.mode == Mode::Player && (registry::is_installed() || std::env::var_os("ZTMYSTRAP_HOME").is_some());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::install_app,
            commands::import_fishstrap,
            commands::get_settings,
            commands::save_settings,
            commands::get_state,
            commands::save_state,
            commands::get_flags,
            commands::save_flags,
            commands::parse_flag_import,
            commands::flag_catalog,
            commands::refresh_allowlist,
            commands::get_global_settings,
            commands::save_global_settings,
            commands::channel_info,
            commands::launch_roblox,
            commands::open_folder,
            commands::open_url,
        ])
        .setup(move |app| {
            let mode = if bootstrap { "bootstrap" } else { "settings" };
            let mut b = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("ztmystrap")
                .initialization_script(&format!("window.__ZTMY_MODE__ = '{mode}';"))
                .center();
            b = if bootstrap {
                b.inner_size(520.0, 280.0).resizable(false).decorations(false).maximizable(false)
            } else {
                b.inner_size(1080.0, 720.0).min_inner_size(820.0, 560.0)
            };
            let window = b.build()?;

            if bootstrap {
                let handle = app.handle().clone();
                let w = window.clone();
                let report: bootstrapper::Report = Arc::new(move |status: &str, value: Option<f64>| {
                    let _ = w.emit("progress", ProgressEvent { status: status.to_string(), value });
                });
                let args = args.clone();
                tauri::async_runtime::spawn(async move {
                    match bootstrapper::run(args, report).await {
                        Ok(()) => handle.exit(0),
                        Err(e) => {
                            log!("bootstrapper", "failed: {e:#}");
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.emit("bootstrap-error", format!("{e:#}"));
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ztmystrap");
}
