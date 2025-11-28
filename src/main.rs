mod assets;
mod config;
mod core;
mod llm;
mod permissions;

use crate::core::app::App;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logging (simple print for now, can be enhanced later)
    println!("Starting goon.ai...");

    // Initialize App
    let app = App::new()?;

    // Run App
    app.run()?;

    Ok(())
}
