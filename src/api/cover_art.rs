use std::{io::Cursor, path::PathBuf};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{self, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
};
use sqlx::query_scalar;

use crate::{
    AppState,
    metadata::{CoverArt, CoverArtType, get_cover_art},
};

use super::*;

#[derive(serde::Serialize)]
struct CoverArtMetadata {
    cover_type: CoverArtType,
    image: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/api/songs/{song_id}/cover-art",
            get(get_song_cover_art_metadata),
        )
        .route(
            "/api/songs/{song_id}/cover-art/{cover_type}",
            get(get_song_cover_art),
        )
        .route(
            "/api/songs/{song_id}/cover-art/{cover_type}/{index}",
            get(get_song_cover_art),
        )
        .route(
            "/api/albums/{album}/cover-art",
            get(get_album_cover_art_metadata),
        )
        .route(
            "/api/albums/{album}/cover-art/{cover_type}",
            get(get_album_cover_art),
        )
        .route(
            "/api/albums/{album}/cover-art/{cover_type}/{index}",
            get(get_album_cover_art),
        )
}

async fn get_song_cover_art(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path((song_id, cover_type)): Path<(String, String)>,
    uri: Uri,
) -> Result<Response, impl IntoResponse> {
    let cover_type = cover_type
        .to_lowercase()
        .split(".")
        .next()
        .unwrap_or(&cover_type)
        .to_string();

    let ext = match uri.path().split('.').next_back() {
        Some(ext) => ext,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Could not determine extension".to_string(),
            ));
        }
    };

    let mime = match mime_guess::from_ext(ext).first() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Could not determine mime type".to_string(),
            ));
        }
    };

    let path = match query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
    {
        Ok(song) => song,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    };

    let cover_art = get_cover_art(&PathBuf::from(path))
        .map_err(internal_error)?
        .into_iter()
        .find(|cover_art| {
            let cover_type = CoverArtType::try_from(cover_type.as_str());

            cover_type == Ok(cover_art.cover_type)
        });

    match cover_art {
        Some(cover_art) => match convert_cover_art(cover_art, ext) {
            Some(cover_art) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, mime.essence_str())
                .body(Body::from(cover_art))
                .unwrap()),
            None => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert cover art".into(),
            )),
        },
        None => Err((StatusCode::NOT_FOUND, "Cover art not found".into())),
    }
}

async fn get_song_cover_art_metadata(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(song_id): Path<String>,
) -> Result<Json<Vec<CoverArtMetadata>>, impl IntoResponse> {
    let path = match query_scalar!("SELECT path FROM songs WHERE id = ?", song_id)
        .fetch_one(&db)
        .await
        .map_err(internal_error)
    {
        Ok(song) => song,
        Err(err) => return Err(err),
    };

    let cover_art = get_cover_art(&PathBuf::from(path))
        .map_err(internal_error)?
        .into_iter()
        .enumerate()
        .map(|(index, cover_art)| CoverArtMetadata {
            cover_type: cover_art.cover_type,
            image: map_type_to_path(&cover_art.cover_type, index),
        })
        .collect();

    Ok(Json(cover_art))
}

async fn get_album_cover_art(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path((album, cover_type)): Path<(String, String)>,
    uri: Uri,
) -> Result<Response, impl IntoResponse> {
    let cover_type = cover_type
        .to_lowercase()
        .split(".")
        .next()
        .unwrap_or(&cover_type)
        .to_string();

    let ext = match uri.path().split('.').next_back() {
        Some(ext) => ext,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Could not determine extension".to_string(),
            ));
        }
    };

    let mime = match mime_guess::from_ext(ext).first() {
        Some(mime) => mime,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Could not determine mime type".to_string(),
            ));
        }
    };

    let paths = match query_scalar!("SELECT path FROM songs WHERE album = ?", album)
        .fetch_all(&db)
        .await
    {
        Ok(song) => song,
        Err(err) => return Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string())),
    };

    let mut cover_art = None;

    for path in paths {
        let art = get_cover_art(&PathBuf::from(path))
            .map_err(internal_error)?
            .into_iter()
            .find(|cover_art| {
                let cover_type = CoverArtType::try_from(cover_type.as_str());

                cover_type == Ok(cover_art.cover_type)
            });

        if let Some(art) = art {
            cover_art = Some(art);
            break;
        }
    }

    match cover_art {
        Some(cover_art) => match convert_cover_art(cover_art, ext) {
            Some(cover_art) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header(http::header::CONTENT_TYPE, mime.essence_str())
                .header(http::header::CACHE_CONTROL, "public, max-age=6000")
                .body(Body::from(cover_art))
                .unwrap()),
            None => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to convert cover art".into(),
            )),
        },
        None => Err((StatusCode::NOT_FOUND, "Cover art not found".into())),
    }
}

async fn get_album_cover_art_metadata(
    State(db): State<sqlx::Pool<sqlx::Sqlite>>,
    Path(album): Path<String>,
) -> Result<Json<Vec<CoverArtMetadata>>, impl IntoResponse> {
    let path = match query_scalar!("SELECT path FROM songs WHERE album = ?", album)
        .fetch_one(&db)
        .await
        .map_err(internal_error)
    {
        Ok(song) => song,
        Err(err) => return Err(err),
    };

    let cover_art = get_cover_art(&PathBuf::from(path))
        .map_err(internal_error)?
        .into_iter()
        .enumerate()
        .map(|(index, cover_art)| CoverArtMetadata {
            cover_type: cover_art.cover_type,
            image: map_type_to_path(&cover_art.cover_type, index),
        })
        .collect();

    Ok(Json(cover_art))
}

fn convert_cover_art(cover_art: CoverArt, extension: &str) -> Option<Vec<u8>> {
    use image::{ImageFormat, load_from_memory};

    let format = ImageFormat::from_extension(extension)?;
    match load_from_memory(&cover_art.data) {
        Ok(cover) => {
            let mut buffer: Vec<u8> = Vec::new();

            if cover
                .to_rgb8()
                .write_to(&mut Cursor::new(&mut buffer), format)
                .is_ok()
            {
                Some(buffer)
            } else {
                None
            }
        }
        Err(err) => {
            tracing::error!("{}", err);
            None
        }
    }
}

fn map_type_to_path(cover_type: &CoverArtType, index: usize) -> String {
    match cover_type {
        CoverArtType::Front => format!("/front/{index}.jpg"),
        CoverArtType::Back => format!("/back/{index}.jpg"),
        CoverArtType::Other => format!("/other/{index}.jpg"),
    }
}
