//! Minimal append-only file logger (one file per process run in `Logs\`).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

static FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

pub fn init() {
    let dir = crate::paths::logs();
    let _ = fs::create_dir_all(&dir);
    let name = format!("ztmystrap_{}.log", chrono::Utc::now().format("%Y%m%dT%H%M%SZ"));
    let file = OpenOptions::new().create(true).append(true).open(dir.join(name)).ok();
    let _ = FILE.set(Mutex::new(file));
}

pub fn write(ident: &str, msg: &str) {
    let line = format!("{} [{}] {}\n", chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ"), ident, msg);
    #[cfg(debug_assertions)]
    eprint!("{line}");
    if let Some(m) = FILE.get() {
        if let Ok(mut g) = m.lock() {
            if let Some(f) = g.as_mut() {
                let _ = f.write_all(line.as_bytes());
            }
        }
    }
}

#[macro_export]
macro_rules! log {
    ($ident:expr, $($arg:tt)*) => { $crate::log::write($ident, &format!($($arg)*)) };
}
