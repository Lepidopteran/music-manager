use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sqlx::query_as;

use crate::{app::AppState, metadata::Song, utils::*};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/songs/", get(get_songs))
        .route("/api/songs/:id", get(get_song))
}

async fn get_song(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<i32>,
) -> Result<Json<Song>, impl IntoResponse> {
    query_as!(Song, "SELECT * FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}

async fn get_songs(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
) -> Result<axum::Json<Vec<Song>>, impl IntoResponse> {
    query_as!(Song, "SELECT * FROM songs")
        .fetch_all(&db)
        .await
        .map(Json)
        .map_err(internal_error)
}
