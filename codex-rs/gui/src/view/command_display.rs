const SCRIPT_FLAGS: [&str; 4] = [" -Command ", " -command ", " -lc ", " -c "];
const SHELL_NAMES: [&str; 5] = ["pwsh", "powershell", "bash", "zsh", "sh"];
const ENCODING_PREAMBLE_END: &str = "catch {}";

pub fn display_command(command: &str) -> String {
    let script = shell_script(command).unwrap_or(command);
    let script = strip_matching_quotes(script.trim());
    let script = strip_encoding_preamble(script);

    first_meaningful_line(script).to_string()
}

fn shell_script(command: &str) -> Option<&str> {
    SCRIPT_FLAGS.iter().find_map(|flag| {
        let (program, script) = command.split_once(flag)?;
        is_shell(program).then_some(script)
    })
}

fn is_shell(program: &str) -> bool {
    let executable = executable(program).to_ascii_lowercase();
    let name = executable
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(&executable)
        .trim_end_matches(".exe");

    SHELL_NAMES.contains(&name)
}

fn executable(program: &str) -> &str {
    let program = program.trim();

    match program.strip_prefix('"') {
        Some(rest) => rest.split('"').next().unwrap_or(rest),
        None => program.split_whitespace().next().unwrap_or(program),
    }
}

fn strip_matching_quotes(script: &str) -> &str {
    ['\'', '"']
        .iter()
        .find_map(|quote| {
            script
                .strip_prefix(*quote)
                .and_then(|inner| inner.strip_suffix(*quote))
        })
        .unwrap_or(script)
}

fn strip_encoding_preamble(script: &str) -> &str {
    if !script.trim_start().starts_with("try {") {
        return script;
    }

    script
        .split_once(ENCODING_PREAMBLE_END)
        .map_or(script, |(_, rest)| rest.trim_start())
}

fn first_meaningful_line(script: &str) -> &str {
    script
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(script)
}

#[cfg(test)]
#[path = "command_display_tests.rs"]
mod tests;
