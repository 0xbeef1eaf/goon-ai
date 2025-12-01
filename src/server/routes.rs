use crate::assets::registry::AssetRegistry;
use crate::config::pack::{Asset, Mood, PackConfig};
use crate::config::settings::Settings;
use crate::permissions::{Permission, PermissionChecker, PermissionSet};
use crate::runtime::GoonRuntime;
use crate::runtime::runtime::RuntimeContext;
use crate::sdk::system::LogCollector;
use crate::server::app::AppState;
use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/api/settings", get(get_settings).post(save_settings))
        .route("/api/packs", get(list_packs).post(create_pack))
        .route("/api/packs/{name}", get(get_pack).post(save_pack))
        .route("/api/packs/{name}/assets/{type}", post(upload_asset))
        .route("/api/run", post(run_code))
        .route("/api/sdk", get(get_sdk_definitions))
}

async fn get_sdk_definitions() -> String {
    crate::sdk::generate_typescript_definitions(&["all".to_string()])
}

#[derive(Deserialize)]
struct CreatePackRequest {
    name: String,
}

async fn create_pack(Json(req): Json<CreatePackRequest>) -> Json<Result<String, String>> {
    let pack_name = req.name.trim();
    if pack_name.is_empty() {
        return Json(Err("Pack name cannot be empty".to_string()));
    }

    // Sanitize pack name (basic check)
    if pack_name.contains('/') || pack_name.contains('\\') || pack_name.contains("..") {
        return Json(Err("Invalid pack name".to_string()));
    }

    let pack_dir = std::path::Path::new("packs").join(pack_name);
    if pack_dir.exists() {
        return Json(Err("Pack already exists".to_string()));
    }

    if let Err(e) = fs::create_dir_all(&pack_dir) {
        return Json(Err(format!("Failed to create pack directory: {}", e)));
    }

    // Create subdirectories
    for subdir in &["image", "video", "audio", "hypno", "wallpaper"] {
        if let Err(e) = fs::create_dir_all(pack_dir.join(subdir)) {
            return Json(Err(format!("Failed to create {} directory: {}", subdir, e)));
        }
    }

    let config = PackConfig::new(pack_name);
    if let Err(e) = config.save(pack_name) {
        return Json(Err(format!("Failed to save pack config: {}", e)));
    }

    Json(Ok("Pack created successfully".to_string()))
}

async fn upload_asset(
    Path((pack_name, asset_type)): Path<(String, String)>,
    mut multipart: Multipart,
) -> Json<Result<String, String>> {
    let mut pack_config = match PackConfig::load(&pack_name) {
        Ok(c) => c,
        Err(e) => return Json(Err(format!("Failed to load pack: {}", e))),
    };

    let pack_dir = std::path::Path::new("packs").join(&pack_name);
    let asset_dir = pack_dir.join(&asset_type);

    if !asset_dir.exists()
        && let Err(e) = fs::create_dir_all(&asset_dir)
    {
        return Json(Err(format!("Failed to create asset directory: {}", e)));
    }

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(name) = field.file_name() {
            name.to_string()
        } else {
            continue;
        };

        let data = if let Ok(bytes) = field.bytes().await {
            bytes
        } else {
            continue;
        };

        let file_path = asset_dir.join(&file_name);
        if let Ok(mut file) = fs::File::create(&file_path) {
            if let Err(e) = file.write_all(&data) {
                return Json(Err(format!("Failed to write file: {}", e)));
            }
        } else {
            return Json(Err("Failed to create file".to_string()));
        }

        // Update pack config
        let relative_path = format!("{}/{}", asset_type, file_name);
        let new_asset = Asset {
            path: relative_path,
            tags: vec![],
        };

        match asset_type.as_str() {
            "image" => {
                if let Some(assets) = &mut pack_config.assets.image {
                    assets.push(new_asset);
                } else {
                    pack_config.assets.image = Some(vec![new_asset]);
                }
            }
            "video" => {
                if let Some(assets) = &mut pack_config.assets.video {
                    assets.push(new_asset);
                } else {
                    pack_config.assets.video = Some(vec![new_asset]);
                }
            }
            "audio" => {
                if let Some(assets) = &mut pack_config.assets.audio {
                    assets.push(new_asset);
                } else {
                    pack_config.assets.audio = Some(vec![new_asset]);
                }
            }
            "wallpaper" => {
                if let Some(assets) = &mut pack_config.assets.wallpaper {
                    assets.push(new_asset);
                } else {
                    pack_config.assets.wallpaper = Some(vec![new_asset]);
                }
            }
            _ => {}
        }
    }

    if let Err(e) = pack_config.save(&pack_name) {
        return Json(Err(format!("Failed to save pack config: {}", e)));
    }

    Json(Ok("Upload successful".to_string()))
}

#[derive(Deserialize)]
struct RunRequest {
    code: String,
}

#[derive(Serialize)]
struct RunResponse {
    logs: Vec<String>,
    error: Option<String>,
}

async fn run_code(State(state): State<AppState>, Json(req): Json<RunRequest>) -> Json<RunResponse> {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();
    let code = req.code.clone();
    let gui_controller = state.gui_controller.clone();

    let (tx, rx) = tokio::sync::oneshot::channel();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let log_collector = LogCollector {
                logs: logs_clone.clone(),
            };

            // Setup runtime
            let mut set = PermissionSet::new();
            // Grant all permissions for sandbox
            set.add(Permission::Image);
            set.add(Permission::Video);
            set.add(Permission::Audio);
            set.add(Permission::Prompt);
            set.add(Permission::Wallpaper);
            set.add(Permission::Website);
            set.add(Permission::Hypno);

            let permissions = PermissionChecker::new(set);
            let registry = Arc::new(AssetRegistry::new()); // Empty registry for now
            let mood = Mood {
                name: "Sandbox".to_string(),
                description: "Sandbox environment".to_string(),
                tags: vec![],
                prompt: None,
            };

            let context = RuntimeContext {
                permissions,
                gui_controller,
                registry,
                mood,
                max_audio_concurrent: 5,
                max_video_concurrent: 2,
            };

            let mut runtime = GoonRuntime::new(context);

            // Inject LogCollector
            {
                let op_state = runtime.js_runtime.op_state();
                let mut op_state = op_state.borrow_mut();
                op_state.put(log_collector);
            }

            let result = runtime.execute_script(&code).await;
            let _ = tx.send(result);
        });
    });

    let result = rx
        .await
        .unwrap_or_else(|_| Err(anyhow::anyhow!("Execution panicked")));

    let final_logs = logs.lock().unwrap().clone();
    let error = result.err().map(|e| e.to_string());

    Json(RunResponse {
        logs: final_logs,
        error,
    })
}

async fn get_settings() -> Json<Settings> {
    // Try to load, if fails create default structure (though Settings::load handles defaults mostly)
    let settings = Settings::load().unwrap_or_else(|_| {
        // If we can't load, we might want to return a default, but Settings doesn't implement Default easily yet.
        // For now, let's panic or return what we can.
        // Actually Settings::load() is what we want.
        panic!("Failed to load settings");
    });
    Json(settings)
}

async fn save_settings(Json(settings): Json<Settings>) -> Json<String> {
    settings.save().expect("Failed to save settings");
    Json("OK".to_string())
}

async fn list_packs() -> Json<Vec<String>> {
    let mut packs = Vec::new();
    if let Ok(entries) = fs::read_dir("packs") {
        for entry in entries {
            if let Ok(entry) = entry
                && entry.path().is_dir()
                && let Some(name) = entry.file_name().to_str()
            {
                packs.push(name.to_string());
            }
        }
    }
    Json(packs)
}

async fn get_pack(Path(name): Path<String>) -> Json<PackConfig> {
    let config = PackConfig::load(&name).expect("Failed to load pack config");
    Json(config)
}

async fn save_pack(Path(name): Path<String>, Json(config): Json<PackConfig>) -> Json<String> {
    config.save(&name).expect("Failed to save pack config");
    Json("OK".to_string())
}
