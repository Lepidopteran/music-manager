use crate::{
    AppState,
    api::bad_request,
    db::Pool,
    services::organize::{self, PathRenameOptions, PathRenamePreviewResult},
};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::Result,
    routing::get,
};

// TODO: Add ability to use a custom templates for folder structure.

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/albums/{title}/organize",
        get(preview_organize_album_tracks).post(organize_album_tracks),
    )
}

async fn organize_album_tracks(
    Path(title): Path<String>,
    State(AppState {
        file_operation_manager: manager,
        pool: db,
        ..
    }): State<AppState>,
    Query(options): Query<PathRenameOptions>,
) -> Result<()> {
    organize::organize_album_tracks(db.clone(), manager.clone(), title, options)
        .await
        .map_err(bad_request)?;

    Ok(())
}

async fn preview_organize_album_tracks(
    State(pool): State<Pool>,
    Path(title): Path<String>,
    Query(options): Query<PathRenameOptions>,
) -> Result<Json<Vec<PathRenamePreviewResult>>> {
    Ok(Json(
        organize::preview_organize_album_tracks(pool, title, options)
            .await
            .map_err(bad_request)?,
    ))
}
