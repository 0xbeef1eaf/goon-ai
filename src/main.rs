use anyhow::Result;
use clap::{Parser, Subcommand};
use goon_ai::core::app::App;
use goon_ai::gui::slint_controller::SlintGuiController;
use std::sync::Arc;
use tracing::info;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the application (default)
    Run,
    /// Configure the application (Web Interface)
    Config,
    /// Run a JavaScript/TypeScript script in the sandbox
    #[command(about = "Execute a script in the goon.ai sandbox")]
    Script {
        /// Path to the script file
        #[arg(value_name = "FILE")]
        path: std::path::PathBuf,
    },
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Config => {
            info!("Starting configuration server at http://localhost:3000");

            // Create Slint GUI controller
            let gui_controller = Arc::new(SlintGuiController::new());

            // Spawn Web Server in a separate thread
            let gui_controller_clone = gui_controller.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    let app = goon_ai::server::app::create_app(gui_controller_clone).await;
                    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

                    // Open browser
                    let _ = open::that("http://localhost:3000");

                    axum::serve(listener, app).await.unwrap();
                });
            });

            // Run Slint Event Loop on main thread
            SlintGuiController::run_event_loop_until_quit()?;
        }
        Commands::Run => {
            info!("Starting goon.ai...");

            // Run App (which handles its own event loop and async runtime)
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            rt.block_on(async {
                // Initialize App
                let app = App::new().unwrap();

                // Run App
                app.run().await.unwrap();
            });
        }
        Commands::Script { path } => {
            info!("Running script: {:?}", path);

            // Run App with script execution mode
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;

            rt.block_on(async {
                // Initialize App with script mode
                let app = App::new().unwrap();

                // Read the script file
                let script_content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                    eprintln!("Failed to read script file: {}", e);
                    std::process::exit(1);
                });

                // Run App with the script
                app.run_script(&script_content).await.unwrap();
            });
        }
    }

    Ok(())
}
