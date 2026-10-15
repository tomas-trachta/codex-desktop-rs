use pretty_assertions::assert_eq;

use super::display_command;

#[test]
fn unwraps_powershell_command() {
    let command = r#""C:\\Program Files\\PowerShell\\7\\pwsh.exe" -Command 'git status'"#;

    assert_eq!(display_command(command), "git status");
}

#[test]
fn drops_powershell_encoding_preamble() {
    let command = "\"C:\\Program Files\\PowerShell\\7\\pwsh.exe\" -NoProfile -Command try { [Console]::OutputEncoding=[System.Text.Encoding]::UTF8 } catch {}\nGet-Location; rg --files";

    assert_eq!(display_command(command), "Get-Location; rg --files");
}

#[test]
fn unwraps_bash_login_command() {
    assert_eq!(
        display_command("/bin/bash -lc 'cargo test -p codex-gui'"),
        "cargo test -p codex-gui"
    );
}

#[test]
fn keeps_plain_commands() {
    assert_eq!(display_command("git diff --stat"), "git diff --stat");
}
