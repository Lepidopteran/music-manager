use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use sqlx::query_as;
use ts_rs::TS;

use crate::{app::AppState, db::Song, utils::*};

#[derive(serde::Serialize, TS)]
#[ts(rename = "Album", export)]
pub struct Album {
    pub title: String,
    pub artist: Option<String>,
    pub tracks: Vec<Song>,
}

impl From<Vec<Song>> for Album {
    fn from(tracks: Vec<Song>) -> Self {
        let title = tracks[0].album.clone().expect("Album not found");
        let artist = tracks[0].album_artist.clone();
        Album {
            title,
            artist,
            tracks,
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/albums/{title}", get(get_album))
        .route("/api/albums/", get(get_albums))
}

async fn get_album(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(title): Path<String>,
) -> Result<Json<Album>, impl IntoResponse> {
    let tracks = match query_as!(Song, "SELECT * FROM songs WHERE album = ?", title)
        .fetch_all(&db)
        .await
    {
        Ok(tracks) => tracks,
        Err(err) => {
            tracing::error!("{}", err);
            return Err(internal_error(err));
        }
    };

    if tracks.is_empty() {
        return Err((StatusCode::NOT_FOUND, "Album not found".to_string()));
    }

    let album = Album::from(tracks);

    Ok(Json(album))
}

async fn get_albums(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
) -> Result<Json<Vec<Album>>, impl IntoResponse> {
    let tracks = match query_as!(Song, "SELECT * FROM songs WHERE album IS NOT NULL")
        .fetch_all(&db)
        .await
    {
        Ok(tracks) => tracks,
        Err(err) => {
            tracing::error!("{}", err);
            return Err(internal_error(err));
        }
    };

    if tracks.is_empty() {
        return Err((StatusCode::NOT_FOUND, "No albums found".to_string()));
    }

    let mut album_map: HashMap<String, Vec<Song>> = HashMap::new();

    for track in tracks {
        album_map
            .entry(track.album.clone().expect("Album not found"))
            .or_default()
            .push(track);
    }

    let albums: Vec<Album> = album_map
        .into_values()
        .map(Album::from)
        .collect();

    if albums.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to construct albums".to_string(),
        ));
    }

    Ok(Json(albums))
}
