use anyhow::Result;
use goon_ai::app_loop::orchestrator::Orchestrator;
use goon_ai::config::pack::PackConfig;
use goon_ai::config::settings::Settings;
use goon_ai::gui::tray::{SystemTray, TrayCommand};
use goon_ai::gui::windows::{WindowSpawner, run_event_loop};
use goon_ai::permissions::{PermissionChecker, PermissionResolver, PermissionSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::info;

fn main() -> Result<()> {
    // Create window spawner channel pair
    let (window_handle, window_spawner) = WindowSpawner::create();

    // Create system tray
    let tray = SystemTray::new()?;

    // Create shared run state for communication between tray and LLM thread
    let is_running = Arc::new(AtomicBool::new(false));
    let is_running_for_llm = is_running.clone();

    // Store window handle for LLM loop thread
    let window_handle_for_llm = window_handle.clone();

    info!("Calling run_event_loop...");

    // Run the Slint event loop with window spawner
    run_event_loop(window_spawner)?;

    info!("Slint event loop exited");

    // Spawn LLM loop thread
    let _llm_thread = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            // Load settings and config
            let settings = match Settings::load() {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    tracing::error!("Failed to load settings: {}", e);
                    return;
                }
            };

            let pack_config = match PackConfig::load(&settings.runtime.pack.current) {
                Ok(c) => Arc::new(c),
                Err(e) => {
                    tracing::error!("Failed to load pack config: {}", e);
                    return;
                }
            };

            // Compute permissions using resolver
            let user_perms: PermissionSet = settings.runtime.permissions.clone().into();
            let pack_perms: PermissionSet = pack_config.meta.permissions.clone().into();
            let active_perms = PermissionResolver::resolve(&pack_perms, &user_perms);
            let permissions = Arc::new(PermissionChecker::new(active_perms));

            info!("LLM loop thread initialized, waiting for run signal...");

            // Wait for the run signal before starting
            loop {
                if is_running_for_llm.load(Ordering::Relaxed) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            info!("Starting orchestrator...");

            // Create and run the orchestrator
            let mut orchestrator =
                Orchestrator::new(settings, pack_config, permissions, window_handle_for_llm);

            // Run the orchestrator loop
            // TODO: Add check for is_running to pause/resume
            if let Err(e) = orchestrator.run().await {
                tracing::error!("Orchestrator error: {}", e);
            }
        });
    });

    // Create a timer to poll tray commands
    let timer = slint::Timer::default();
    let tray_cell = std::cell::RefCell::new(tray);
    let is_running_for_tray = is_running.clone();

    // Start the timer to poll tray commands
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(50),
        move || {
            let mut tray = tray_cell.borrow_mut();
            while let Some(cmd) = tray.poll_command() {
                match cmd {
                    TrayCommand::ToggleRunPause => {
                        let running = !tray.is_running();
                        tray.set_running(running);
                        is_running_for_tray.store(running, Ordering::Relaxed);
                        if running {
                            info!("LLM loop started");
                        } else {
                            info!("LLM loop paused");
                        }
                    }
                    TrayCommand::OpenConfig => {
                        info!("Opening configuration window...");
                        // TODO: Open config window
                    }
                    TrayCommand::OpenPackEditor => {
                        info!("Opening pack editor window...");
                        // TODO: Open pack editor window
                    }
                    TrayCommand::Quit => {
                        info!("Quitting application...");
                        let _ = slint::quit_event_loop();
                    }
                }
            }
        },
    );

    Ok(())
}
