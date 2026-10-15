# Codex GUI

A native desktop client for [OpenAI Codex](https://github.com/openai/codex),
written in Rust with [Iced](https://iced.rs). It looks and works like the
Codex desktop app, but uses roughly 150 MB of memory instead of the gigabytes
an Electron app can take.

The GUI embeds the Codex app-server in-process through
`codex-app-server-client`, the same way the official TUI does. There is no
Electron, no webview and no separate server process. Threads, projects,
configuration and login are shared with the official Codex apps through
`~/.codex`.

This repository is a fork of `openai/codex`, based on the `rust-v0.162.0`
release. The GUI lives in [`codex-rs/gui`](codex-rs/gui).

## Features

- Projects and recent chats in the sidebar, with search. Projects are shared
  with the Codex desktop app, and new ones can be added from a folder picker.
- Streaming replies rendered as Markdown, with selectable and copyable text.
- Codex's work shown as compact steps ("Ran git status", "Edited 2 files
  +12 −3") that expand to the full command output or a coloured diff.
- Model, reasoning-effort and approval-mode pickers in the composer.
- Inline approval cards for commands and file changes.
- Rename, archive and read-only viewing of chats that are open in another
  Codex window.
- The desktop app's keyboard shortcuts.

## Requirements

- Windows 10 or later (Linux and macOS may build, but are untested).
- The Rust toolchain from [`codex-rs/rust-toolchain.toml`](codex-rs/rust-toolchain.toml).
- A Codex login: run `codex login`, or sign in once through the official app.
- About 40 GB of free disk space for the first build.

## Building

From `codex-rs`, build with link-time optimization off and incremental
compilation on. The first build takes 30 minutes or more; after that,
GUI-only changes rebuild in under a minute.

```powershell
$env:CARGO_PROFILE_RELEASE_LTO = "false"
$env:CARGO_PROFILE_RELEASE_INCREMENTAL = "true"
cargo build --release -p codex-gui
cargo build --release -p codex-windows-sandbox --bin codex-command-runner --bin codex-windows-sandbox-setup
```

On Windows, Codex also needs `codex-code-mode-host.exe` next to
`codex-gui.exe`. Building it requires a custom V8 build, so copy it from a
Codex desktop install instead:

```powershell
Copy-Item "$env:LOCALAPPDATA\OpenAI\Codex\bin\*\codex-code-mode-host.exe" target\release\
```

## Running

```powershell
target\release\codex-gui.exe [PROJECT_DIR]
```

`PROJECT_DIR` defaults to the current directory.

## Keeping in sync with the desktop app

The OpenAI backend gates models and request fields by client version. Keep
this fork on the same `rust-vX.Y.Z` release that your Codex desktop app
bundles; you can find that version by running `codex.exe --version` from the
app's `bin` folder. When the desktop app updates, rebase onto the matching
upstream tag.

## Keyboard shortcuts

| Shortcut           | Action                              |
| ------------------ | ----------------------------------- |
| Ctrl+N             | New chat                            |
| Ctrl+O             | Add project                         |
| Ctrl+B             | Toggle sidebar                      |
| Ctrl+1 … Ctrl+9    | Open a recent chat                  |
| Ctrl+Tab / +Shift  | Next / previous chat                |
| Ctrl+Shift+M       | Model picker                        |
| Ctrl+Alt+R         | Rename chat                         |
| Ctrl+Shift+A       | Archive chat                        |
| Enter / Esc        | Approve / decline a pending request |
| ↑ (empty composer) | Restore the previous prompt         |

## Limitations

- Selection works within one message at a time.
- MCP elicitations, `request_user_input` prompts, image attachments, the review
  pane, older-history paging, login and settings screens are not implemented
  yet.

## License

Apache-2.0, like upstream Codex. See [LICENSE](LICENSE) and [NOTICE](NOTICE).
