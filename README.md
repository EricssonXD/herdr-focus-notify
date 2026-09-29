# Herdr Focus Notify

English | [简体中文](README.zh-CN.md)

`herdr-focus-notify` is a desktop notification plugin for Herdr on macOS and Linux. It shows a clickable notification when an agent is `blocked` or `done`. Clicking it focuses the matching Herdr pane.

On macOS it avoids alerts when it can confirm that you are already looking at the pane. Linux always alerts on these status changes because frontmost-window detection is not portable.

## Herdr compatibility

For **Herdr 0.9.0, use plugin tag `v0.5.0` or later**. The `v0.4.0` tag predates the client focus changes: clicking a notification may activate the terminal without switching to the target pane. Later versions explicitly project the target pane into attached client views.

The minimum supported Herdr version remains `0.7.5`. Workspace-to-terminal bindings are unchanged.

## Quick start

### 1. Install the requirements

- Herdr `0.7.5` or later
- macOS: [alerter](https://github.com/vjeantet/alerter)
- Linux: `gdbus`, `dbus-monitor`, and a Freedesktop-compatible notification service

On macOS, install alerter with `brew install vjeantet/tap/alerter`. On Debian/Ubuntu, the Linux tools are provided by `libglib2.0-bin` and `dbus`.

### 2. Install the plugin

Install from GitHub:

```bash
herdr plugin install EricssonXD/herdr-focus-notify
```

Or build and link the local checkout:

```bash
cargo build --release
herdr plugin link .
```

### 3. Done — zero configuration

The plugin works with **zero configuration**. On macOS it learns the terminal bound to each workspace and activates it on click. On Linux it uses the desktop notification service directly and focuses the Herdr pane over its socket; kitty is raised when its remote-control adapter is available.

No configuration files are needed. The required notification tools are detected from `PATH`.

## How notifications behave

By default, `blocked` and `done` status changes can produce a notification. macOS suppresses one only when it can confirm that you are already looking at that pane. Linux errs on the side of notifying.

On macOS:

| Your current view | Notification |
|---|---|
| Another app is frontmost | Sent |
| Herdr is frontmost, but a different pane is focused | Sent |
| Herdr is frontmost and the matching pane is focused | Skipped |
| The terminal bound to the pane's workspace is frontmost and the pane is focused | Skipped (you are looking at Herdr) |
| The focused app cannot be determined | Sent, to avoid missing a change |

Clicking a notification sends Herdr's `pane.focus` socket request for the target pane. On macOS, a saved terminal binding activates the terminal first. On Linux, kitty's remote-control adapter raises its window when available; other terminal/window-manager activation is desktop-specific.

## Linux: persistent question notifications

The `pi-ask-user` extension marks the agent `blocked` while waiting for your answer, so the existing blocked-status event triggers this notification. Linux notifications request no expiry and stay resident until you dismiss them or choose **Open**/**Focus**; clicking to focus sends Herdr's pane-focus request. Notification servers can still enforce their own policy. The notification copy is generic and does not expose the question text.

### macOS: multiple terminal windows and tabs

With several windows or tabs open, activating the terminal app alone may bring forward one that is not running Herdr. In supported terminals, a click first selects the window, tab, or split running a Herdr client attached to that session, and raises it together with its OS window. When several clients are attached, the most recently used one is chosen.

| Terminal | Setup |
|---|---|
| iTerm2 | None. The plugin passes the client's `ITERM_SESSION_ID` to iTerm2's built-in reveal URL. |
| kitty | Enable remote control, as shown below. |

```conf
# kitty.conf (restart kitty afterwards)
allow_remote_control socket-only
listen_on unix:/tmp/kitty
```

In other terminals, or in kitty without these settings, the click only activates the terminal app, and macOS decides which window comes forward.

Blocked notifications say that the agent needs your input and prompt you to review and respond. Done notifications say that the agent finished and prompt you to review the result. The plugin does not read or summarize pane contents.

When you manually focus the matching pane in Herdr, its pending notification is removed. On macOS, a notification for an already-active pane also clears shortly after returning to its bound terminal.

## macOS: how it stays quiet

- Notifications only fire for `blocked` and `done` — the two statuses that actually need you.
- A notification is skipped when the pane is already focused **and** the frontmost app is the terminal bound to its workspace (learned automatically).
- If you are elsewhere when the pane becomes active, the notification auto-removes within a few seconds once you switch back to the bound terminal.

The `--test` action sends a real notification so you can verify the pipeline. Linux test notifications also remain until dismissed or clicked.

## Troubleshooting

| Problem | What to check |
|---|---|
| No notification appears | Check that `alerter` is installed on macOS, or `gdbus` and `dbus-monitor` are on `PATH` and a desktop notification service is running on Linux. |
| Click brings the right terminal forward, but not the window or tab running Herdr | Window and tab selection works only in iTerm2 and in kitty with remote control enabled (see [Multiple terminal windows and tabs](#multiple-terminal-windows-and-tabs)). Other terminals only get app-level activation. |
| Click does not bring forward the expected terminal | Use the **Clear saved terminal bindings** plugin action, then focus a pane once in the expected terminal. |
| A workspace has a stale terminal binding | Use the **Clear saved terminal bindings** plugin action, then focus a pane once in the expected terminal. |
| Notifications appear while you are viewing Herdr | You were not in the workspace's bound terminal at that moment; the plugin errs on the side of notifying rather than missing a state change. |
| Need diagnostic information | Run the plugin with `--test` or `--check-pane-visibility <pane_id>` to exercise the pipeline and focus checks directly. |

## Bundled icons

Recognised agent names use bundled local icons, including Codex, Claude Code, Cursor, Gemini, GitHub Copilot, DeepSeek, Qwen, Kimi, OpenCode, OpenHands, Cline, Windsurf, Devin, omp, pi, and v0.

The icons are vendored from `@lobehub/icons-static-png` under the MIT license, except `omp.png` and `pi.png`, which use the official logos of Oh My Pi and the Pi coding agent. See `assets/icons/NOTICE.md`.
