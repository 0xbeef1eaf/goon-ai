use anyhow::Result;
use goon_ai::core::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (simple print for now, can be enhanced later)
    println!("Starting goon.ai...");

    // Initialize App
    let app = App::new()?;

    // Run App
    app.run().await?;

    Ok(())
}
