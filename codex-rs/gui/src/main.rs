// Release builds exceed rustc's default query depth when laying out the async
// MessageProcessor::handle_initialized_client_request future.
#![recursion_limit = "256"]

mod app;
mod approval;
mod backend;
mod launch;
mod transcript;
mod view;

use launch::LaunchOptions;

fn main() -> iced::Result {
    let arg0_guard = codex_arg0::arg0_dispatch();
    init_tracing();

    let launch = LaunchOptions::from_environment(arg0_guard.as_ref());
    let result = app::run(launch);

    drop(arg0_guard);
    result
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .try_init();
}
