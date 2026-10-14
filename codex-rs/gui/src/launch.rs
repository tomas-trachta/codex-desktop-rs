use std::path::PathBuf;

use codex_arg0::Arg0DispatchPaths;
use codex_arg0::Arg0PathEntryGuard;
use codex_utils_absolute_path::canonicalize_existing_preserving_symlinks;

#[derive(Clone, Debug, Hash)]
pub struct LaunchOptions {
    pub codex_self_exe: Option<PathBuf>,
    pub codex_linux_sandbox_exe: Option<PathBuf>,
    pub cwd: PathBuf,
}

impl LaunchOptions {
    pub fn from_environment(arg0_guard: Option<&Arg0PathEntryGuard>) -> Self {
        let codex_self_exe = std::env::current_exe().ok();
        let codex_linux_sandbox_exe = linux_sandbox_exe(arg0_guard, codex_self_exe.clone());

        Self {
            codex_self_exe,
            codex_linux_sandbox_exe,
            cwd: requested_cwd(),
        }
    }

    pub fn dispatch_paths(&self) -> Arg0DispatchPaths {
        Arg0DispatchPaths {
            codex_self_exe: self.codex_self_exe.clone(),
            codex_linux_sandbox_exe: self.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: None,
        }
    }
}

fn linux_sandbox_exe(
    arg0_guard: Option<&Arg0PathEntryGuard>,
    current_exe: Option<PathBuf>,
) -> Option<PathBuf> {
    if !cfg!(target_os = "linux") {
        return None;
    }

    arg0_guard
        .and_then(|guard| guard.paths().codex_linux_sandbox_exe.clone())
        .or(current_exe)
}

fn requested_cwd() -> PathBuf {
    let from_args = std::env::args_os().nth(1).map(PathBuf::from);
    let fallback = || std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cwd = from_args.unwrap_or_else(fallback);
    let absolute = std::path::absolute(&cwd).unwrap_or(cwd);

    canonicalize_existing_preserving_symlinks(&absolute).unwrap_or(absolute)
}
