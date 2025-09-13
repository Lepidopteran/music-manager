use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use ts_rs::TS;

use crate::app::AppState;

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
struct AppInfo {
    /// The version number of the application
    version: String,
    /// The name of the application
    name: String,
    /// System information that the application is running on
    system: SystemInfo,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
struct SystemInfo {
    /// The name of the operating system, e.g. "Windows", "Ubuntu", "Darwin"
    os: String,
    /// The name of the computer
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
