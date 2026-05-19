mod app;
mod config;
mod graph;
mod hardware;
mod services;
mod single_instance;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    if hardware::fan_control::handle_helper_args(std::env::args_os())? {
        return Ok(());
    }

    let Some(_single_instance_guard) = single_instance::acquire()? else {
        eprintln!("NitroSense is already running.");
        return Ok(());
    };

    app::run()
}
