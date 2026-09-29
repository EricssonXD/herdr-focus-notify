#[cfg(target_os = "linux")]
use std::fs;
use std::io;
use std::path::Path;
#[cfg(not(target_os = "linux"))]
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::executable::find_executable;
#[cfg(not(target_os = "linux"))]
use crate::executable::home_dir;
use crate::util::{notification_group_id, shell_quote};

#[cfg(target_os = "linux")]
pub(crate) fn resolve_notifier_bin() -> Result<String, String> {
    find_executable("gdbus", Vec::new())
        .ok_or_else(|| "gdbus is required for Linux desktop notifications".to_string())
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn resolve_notifier_bin() -> Result<String, String> {
    find_executable("alerter", alerter_candidate_paths()).ok_or_else(|| {
        "no alerter notifier found; install alerter with `brew install vjeantet/tap/alerter`"
            .to_string()
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn resolve_dbus_monitor_bin() -> Result<String, String> {
    find_executable("dbus-monitor", Vec::new())
        .ok_or_else(|| "dbus-monitor is required for clickable Linux notifications".to_string())
}

#[cfg(target_os = "linux")]
pub(crate) fn notification_id_path(pane_id: &str) -> std::path::PathBuf {
    crate::state::plugin_state_dir().join(format!(
        "{}.notification-id",
        notification_group_id(pane_id)
    ))
}

pub(crate) fn send_notification(script_path: &Path, foreground: bool) -> io::Result<()> {
    if foreground {
        run_script_foreground(script_path)
    } else {
        spawn_detached_script(script_path)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn remove_notification(pane_id: &str, notifier_bin: &str) -> io::Result<()> {
    let id_path = notification_id_path(pane_id);
    let id = fs::read_to_string(&id_path)
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok());
    let Some(id) = id else {
        return Ok(());
    };
    let id = id.to_string();
    let result = Command::new(notifier_bin)
        .args([
            "call",
            "--session",
            "--dest",
            "org.freedesktop.Notifications",
            "--object-path",
            "/org/freedesktop/Notifications",
            "--method",
            "org.freedesktop.Notifications.CloseNotification",
            &id,
        ])
        .status();
    let _ = fs::remove_file(id_path);
    match result {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(io::Error::other(format!(
            "notification removal exited with {status}"
        ))),
        Err(err) => Err(err),
    }
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn remove_notification(pane_id: &str, notifier_bin: &str) -> io::Result<()> {
    let group = notification_group_id(pane_id);
    match Command::new(notifier_bin)
        .arg("--remove")
        .arg(group)
        .status()
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(io::Error::other(format!(
            "notification removal exited with {status}"
        ))),
        Err(err) => Err(err),
    }
}

#[cfg(not(target_os = "linux"))]
fn alerter_candidate_paths() -> Vec<PathBuf> {
    let mut paths = vec![
        PathBuf::from("/opt/homebrew/bin/alerter"),
        PathBuf::from("/usr/local/bin/alerter"),
    ];
    if let Some(home) = home_dir() {
        paths.push(home.join(".local/bin/alerter"));
    }
    paths
}

fn run_script_foreground(script_path: &Path) -> io::Result<()> {
    match Command::new("sh").arg(script_path).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(io::Error::other(format!(
            "notification script exited with {status}"
        ))),
        Err(err) => Err(err),
    }
}

fn spawn_detached_script(script_path: &Path) -> io::Result<()> {
    let script_str = script_path.to_string_lossy();
    let cmd = format!(
        "nohup sh {} >/dev/null 2>&1 &",
        shell_quote(script_str.as_ref())
    );

    Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(all(test, not(target_os = "linux")))]
mod tests {
    use super::*;

    #[test]
    fn alerter_candidates_include_common_homebrew_paths() {
        let paths = alerter_candidate_paths();

        assert!(paths.contains(&PathBuf::from("/opt/homebrew/bin/alerter")));
        assert!(paths.contains(&PathBuf::from("/usr/local/bin/alerter")));
    }
}
