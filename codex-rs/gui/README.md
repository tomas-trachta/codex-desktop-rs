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

| Module          | Responsibility                                                       |
| --------------- | -------------------------------------------------------------------- |
| `launch`        | arg0 helper paths and the workspace directory                        |
| `backend`       | owns the embedded app-server on its own Tokio runtime thread         |
| `backend::requests` | typed `thread/*` and `turn/*` requests                           |
| `transcript`    | folds thread history and streamed notifications into display entries |
| `approval`      | command and file-change approval requests and their responses       |
| `app`           | Iced state, messages and update logic                               |
| `view`          | sidebar, transcript, composer and approval dialog                   |

## MVP scope

Supported: thread list, opening and resuming threads, new threads, streamed
agent messages rendered as Markdown, reasoning summaries, command output,
file-change summaries, interrupting a turn, and command/file-change approvals.

Not yet supported, and rejected or ignored for now: `request_user_input`
prompts, MCP elicitations, permission requests, dynamic tools, image
attachments, model/approval-mode pickers, diff views, older-history paging,
login UI and settings.
