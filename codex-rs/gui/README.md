# codex-gui

A native desktop client for Codex built with [Iced](https://iced.rs). It embeds
the app-server in-process through `codex-app-server-client`, the same way the
TUI and `codex exec` do, so there is no Electron, no webview and no separate
server process.

## Running

```sh
cargo run --release -p codex-gui -- [WORKSPACE_DIR]
```

`WORKSPACE_DIR` defaults to the current directory. Configuration, auth and
thread history come from `CODEX_HOME` (usually `~/.codex`), so log in with
`codex login` first if you have not already.

Set `RUST_LOG=codex_gui=debug` to see backend logs on stderr.

## Layout

The package has two targets. Keep code that drives the embedded app-server in
the library: it is expensive to optimize, and the split lets UI-only edits
rebuild in seconds.

| Target | Module                    | Responsibility                                                     |
| ------ | ------------------------- | ------------------------------------------------------------------ |
| lib    | `launch`                  | arg0 helper paths and the workspace directory                      |
| lib    | `backend`                 | owns the embedded app-server on its own Tokio runtime thread       |
| lib    | `backend::requests`       | typed `thread/*` and `turn/*` requests                             |
| lib    | `approval`                | command and file-change approval requests and their responses     |
| bin    | `app`                     | Iced state, messages and update logic                              |
| bin    | `shortcuts`               | keyboard shortcuts, matched by physical key for digits             |
| bin    | `transcript`              | folds thread history and streamed notifications into display rows |
| bin    | `view`                    | sidebar, header, transcript, composer and approval card            |
| bin    | `approval_modes`          | approval-mode presets shared with the TUI                          |
| bin    | `view::markdown`          | Markdown rendering with light code blocks and inline code chips   |
| bin    | `view::flat_markdown`     | flattens a reply into one selectable text block                    |
| bin    | `view::command_display`   | turns shell-wrapped commands into readable step labels             |

## Keyboard shortcuts

| Shortcut              | Action                            |
| --------------------- | --------------------------------- |
| Ctrl+N                | New chat                          |
| Ctrl+O                | Add project                       |
| Ctrl+B                | Toggle sidebar                    |
| Ctrl+1 … Ctrl+9       | Open recent chat                  |
| Ctrl+Tab / +Shift     | Next / previous chat              |
| Ctrl+Shift+M          | Model picker                      |
| Ctrl+Alt+R            | Rename chat                       |
| Ctrl+Shift+A          | Archive chat                      |
| Enter / Esc           | Approve / decline a pending request |
| ↑ (empty composer)    | Restore the previous prompt       |

## MVP scope

Supported: projects and recents in the sidebar with search, opening and
resuming threads (read-only when another Codex window owns them), new chats,
renaming and archiving, streamed agent messages rendered as Markdown,
collapsible work steps with command output and coloured diffs, "Worked for"
turn summaries, interrupting a turn, command/file-change approvals, model,
reasoning-effort and approval-mode pickers, adding projects, and selectable
transcript text.

Not yet supported, and rejected or ignored for now: `request_user_input`
prompts, MCP elicitations, permission requests, dynamic tools, image
attachments, the review pane, older-history paging, login UI and settings.

The release build needs `codex-command-runner.exe`,
`codex-windows-sandbox-setup.exe` and `codex-code-mode-host.exe` next to
`codex-gui.exe` on Windows. Build the first two with
`cargo build --release -p codex-windows-sandbox --bin codex-command-runner --bin codex-windows-sandbox-setup`.
Copy the code-mode host from a Codex desktop install, because building it needs
a custom V8 build.
