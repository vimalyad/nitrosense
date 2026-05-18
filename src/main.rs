mod app;
mod config;
mod graph;
mod hardware;
mod services;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    if hardware::fan_control::handle_helper_args(std::env::args_os())? {
        return Ok(());
    }

    app::run()
}
