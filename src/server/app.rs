use crate::gui::WindowSpawnerHandle;
use crate::server::routes;
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub window_spawner: WindowSpawnerHandle,
}

pub async fn create_app(window_spawner: WindowSpawnerHandle) -> Router {
    let state = AppState { window_spawner };

    Router::new()
        .merge(routes::api_routes())
        .nest_service("/packs", ServeDir::new("packs")) // Serve pack assets
        .fallback_service(ServeDir::new("web/dist"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
