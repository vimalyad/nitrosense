mod app;
mod config;
mod graph;
mod hardware;
mod services;
mod ui;

use anyhow::Result;

fn main() -> Result<()> {
    app::run()
}
