//! Ghostty GTK: present the exact terminal surface through its D-Bus action.
//!
//! Recent Ghostty builds export `GHOSTTY_SURFACE_ID` to commands running in a
//! surface and accept that ID through the `present-surface` GAction. Older
//! releases expose no stable external surface handle and are skipped.

use std::ffi::OsString;

use super::{FocusCommand, HerdrClient, TerminalAdapter};

pub(super) struct Ghostty;

impl TerminalAdapter for Ghostty {
    fn bundle_id(&self) -> &'static str {
        "com.mitchellh.ghostty"
    }

    fn focus_command(&self, client: &HerdrClient) -> Option<FocusCommand> {
        if client.env("TERM_PROGRAM") != Some("ghostty") || client.env("KITTY_WINDOW_ID").is_some()
        {
            return None;
        }

        let surface_id = client.env("GHOSTTY_SURFACE_ID")?.strip_prefix("0x")?;
        if surface_id.len() != 16 || !surface_id.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let surface_id = u64::from_str_radix(surface_id, 16).ok()?;
        if surface_id == 0 {
            return None;
        }

        Some(FocusCommand::new(
            "gdbus",
            [
                OsString::from("call"),
                OsString::from("--session"),
                OsString::from("--dest"),
                OsString::from("com.mitchellh.ghostty"),
                OsString::from("--object-path"),
                OsString::from("/com/mitchellh/ghostty"),
                OsString::from("--method"),
                OsString::from("org.gtk.Actions.Activate"),
                OsString::from("present-surface"),
                OsString::from(format!("[<uint64 0x{surface_id:016x}>]")),
                OsString::from("{}"),
            ],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client(pairs: &[(&str, &str)]) -> HerdrClient {
        HerdrClient::from_pairs(pairs)
    }

    #[test]
    fn presents_the_exact_ghostty_surface_over_dbus() {
        assert_eq!(
            Ghostty.focus_command(&client(&[
                ("TERM_PROGRAM", "ghostty"),
                ("GHOSTTY_SURFACE_ID", "0x0123456789abcdef"),
            ])),
            Some(FocusCommand::new(
                "gdbus",
                [
                    "call",
                    "--session",
                    "--dest",
                    "com.mitchellh.ghostty",
                    "--object-path",
                    "/com/mitchellh/ghostty",
                    "--method",
                    "org.gtk.Actions.Activate",
                    "present-surface",
                    "[<uint64 0x0123456789abcdef>]",
                    "{}",
                ],
            ))
        );
    }

    #[test]
    fn requires_a_valid_ghostty_surface_id() {
        for pairs in [
            &[("TERM_PROGRAM", "ghostty")][..],
            &[
                ("TERM_PROGRAM", "xterm-256color"),
                ("GHOSTTY_SURFACE_ID", "0x0123456789abcdef"),
            ],
            &[("TERM_PROGRAM", "ghostty"), ("GHOSTTY_SURFACE_ID", "0x0")],
            &[
                ("TERM_PROGRAM", "ghostty"),
                ("GHOSTTY_SURFACE_ID", "0x0000000000000000"),
            ],
            &[
                ("TERM_PROGRAM", "ghostty"),
                ("GHOSTTY_SURFACE_ID", "not-an-id"),
            ],
        ] {
            assert_eq!(Ghostty.focus_command(&client(pairs)), None);
        }
    }
}
