//! Small Win32 helpers: processes, named mutexes, message boxes.

use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn is_process_running(image: &str) -> bool {
    Command::new("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH", "/FO", "CSV"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains(&image.to_ascii_lowercase()))
        .unwrap_or(false)
}

pub fn kill_process(image: &str) {
    let _ = Command::new("taskkill").args(["/F", "/IM", image]).creation_flags(CREATE_NO_WINDOW).output();
}

/// Acquire a named mutex for the rest of the process lifetime, blocking until free.
pub fn hold_named_mutex(name: &str) {
    use windows_sys::Win32::System::Threading::{CreateMutexW, INFINITE, WaitForSingleObject};
    let name = wide(name);
    unsafe {
        let h = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        if !h.is_null() {
            WaitForSingleObject(h, INFINITE);
            // Intentionally leaked: released when the process exits.
        }
    }
}

pub enum Icon {
    Info,
    Warning,
    Error,
    Question,
}

fn message_box(text: &str, icon: Icon, yes_no: bool) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let flags = match icon {
        Icon::Info => MB_ICONINFORMATION,
        Icon::Warning => MB_ICONWARNING,
        Icon::Error => MB_ICONERROR,
        Icon::Question => MB_ICONQUESTION,
    } | if yes_no { MB_YESNO } else { MB_OK }
        | MB_TOPMOST;
    let (t, c) = (wide(text), wide("ztmystrap"));
    let r = unsafe { MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), flags) };
    r == IDYES || (!yes_no && r == IDOK)
}

pub fn alert(text: &str, icon: Icon) {
    message_box(text, icon, false);
}

pub fn confirm(text: &str) -> bool {
    message_box(text, Icon::Question, true)
}

/// Toggle the read-only attribute on a file.
pub fn set_readonly(path: &std::path::Path, ro: bool) -> std::io::Result<()> {
    let mut p = std::fs::metadata(path)?.permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    p.set_readonly(ro);
    std::fs::set_permissions(path, p)
}

/// Spawn a detached `cmd` that deletes `target` after this process exits.
pub fn delete_after_exit(target: &std::path::Path, is_dir: bool) {
    let t = target.display().to_string();
    let del = if is_dir { format!("rmdir /s /q \"{t}\"") } else { format!("del /f /q \"{t}\"") };
    let _ = Command::new("cmd").args(["/c", &format!("timeout /t 3 /nobreak >nul & {del}")]).creation_flags(CREATE_NO_WINDOW).spawn();
}
