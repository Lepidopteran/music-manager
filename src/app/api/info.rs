use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::app::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    name: String,
    system: SystemInfo,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemInfo {
    os: String,
    name: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/info", get(get_app_info))
}

async fn get_app_info() -> Response {
    Json(AppInfo {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        system: SystemInfo {
            os: sysinfo::System::name().expect("Failed to get system name").to_string(),
            name: sysinfo::System::host_name().expect("Failed to get host name").to_string(),
        },
    })
    .into_response()
}
