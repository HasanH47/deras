use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tower_http::cors::{Any, CorsLayer};

use crate::commands::add_download_internal;
use crate::models::DownloadTask;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddDownloadPayload {
    pub url: String,
}

#[derive(Serialize)]
pub struct AddDownloadResponse {
    pub success: bool,
    pub task: Option<DownloadTask>,
    pub error: Option<String>,
}

#[derive(Clone)]
struct ServerState {
    app_handle: AppHandle,
}

pub async fn start_local_server(app_handle: AppHandle) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let server_state = ServerState { app_handle };

    let app = Router::new()
        .route("/api/ping", get(|| async { "pong" }))
        .route("/api/add-download", post(handle_add_download))
        .layer(cors)
        .with_state(server_state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 4142));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Local server listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handle_add_download(
    State(state): State<ServerState>,
    Json(payload): Json<AddDownloadPayload>,
) -> (StatusCode, Json<AddDownloadResponse>) {
    let app_state = state.app_handle.state::<AppState>();

    // Determine save path
    let default_save_path = "~/Downloads".to_string();

    match add_download_internal(
        &state.app_handle,
        &app_state,
        payload.url.clone(),
        default_save_path,
    ) {
        Ok(task) => (
            StatusCode::OK,
            Json(AddDownloadResponse {
                success: true,
                task: Some(task),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(AddDownloadResponse {
                success: false,
                task: None,
                error: Some(e),
            }),
        ),
    }
}
