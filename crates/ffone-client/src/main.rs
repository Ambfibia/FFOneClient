#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;

fn main() -> std::process::ExitCode {
    app::entry()
}
