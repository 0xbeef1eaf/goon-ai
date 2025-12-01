use crate::gui::slint_controller::SlintGuiController;
use crate::server::routes;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub gui_controller: Arc<SlintGuiController>,
}

pub async fn create_app(gui_controller: Arc<SlintGuiController>) -> Router {
    let state = AppState { gui_controller };

    Router::new()
        .merge(routes::api_routes())
        .nest_service("/packs", ServeDir::new("packs")) // Serve pack assets
        .fallback_service(ServeDir::new("web/dist"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}
