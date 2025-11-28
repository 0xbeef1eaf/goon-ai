mod config;
mod core;
mod permissions;

use anyhow::Result;
use crate::core::app::App;

fn main() -> Result<()> {
    // Initialize logging (simple print for now, can be enhanced later)
    println!("Starting goon.ai...");

    // Initialize App
    let app = App::new()?;

    // Run App
    app.run()?;

    Ok(())
}
