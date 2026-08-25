use sqlx::{query_as, query_scalar};
use ts_rs::TS;

use super::*;

#[derive(Serialize, Debug, FromRow, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "Album", export)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub artist_sort: Option<String>,
    pub artists: Option<String>,
    pub artists_sort: Option<String>,
    pub original_release_date: Option<String>,
    pub label: Option<String>,
    pub barcode: Option<String>,
    pub release_date: Option<String>,
    pub disc_total: Option<String>,
    pub musicbrainz_release_id: Option<String>,
    pub musicbrainz_release_artist_id: Option<String>,
    pub musicbrainz_release_group_id: Option<String>,
    pub script: Option<String>,
    pub language: Option<String>,
    pub replaygain_album_gain: Option<String>,
    pub replaygain_album_peak: Option<String>,
    #[serde(serialize_with = "time::serde::rfc3339::serialize")]
    #[ts(type = "string")]
    pub added_at: OffsetDateTime,
    #[serde(serialize_with = "time::serde::rfc3339::option::serialize")]
    #[ts(type = "string")]
    pub updated_at: Option<OffsetDateTime>,
}

impl Album {
    pub async fn get_songs(&self, connection: &mut Connection) -> Result<Vec<Song>> {
        Ok(
            query_as!(Song, "SELECT * FROM songs WHERE album_id = ?", self.id)
                .fetch_all(&mut *connection)
                .await?,
        )
    }
}

pub async fn get_albums(connection: &mut Connection) -> Result<Vec<Album>> {
    Ok(query_as!(Album, "SELECT * FROM albums")
        .fetch_all(&mut *connection)
        .await?)
}

pub async fn get_album_from_title(connection: &mut Connection, title: &str) -> Result<Album> {
    Ok(
        query_as!(Album, "SELECT * FROM albums WHERE title = ?", title)
            .fetch_one(&mut *connection)
            .await?,
    )
}

pub async fn get_album(connection: &mut Connection, id: &str) -> Result<Album> {
    Ok(query_as!(Album, "SELECT * FROM albums WHERE id = ?", id)
        .fetch_one(&mut *connection)
        .await?)
}
