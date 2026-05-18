mod app;
mod config;
mod fan_control;
mod graph;
mod notifications;
mod profile;
mod sensors;
mod tray;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
