use crate::metadata::item::ItemKey;

use super::{Error, Metadata as SongMetadata};

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub title: String,
    pub tracks: Vec<SongMetadata>,
    pub barcode: Option<String>,
    pub catalog_number: Option<String>,
    pub comment: Option<String>,
    pub country: Option<String>,
    pub artist: Option<String>,
    pub label: Option<String>,
    pub date: Option<String>,
    pub original_date: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum AlbumError {
    #[error("No album found")]
    NoAlbum,
    #[error("No tracks found")]
    NoTracks,
    #[error("All tracks must be from the same album")]
    MixedTracks,
}

impl TryFrom<Vec<SongMetadata>> for Album {
    type Error = Error;

    fn try_from(tracks: Vec<SongMetadata>) -> Result<Self, Self::Error> {
        let (first, rest) = tracks.split_first().ok_or(AlbumError::NoTracks)?;
        let title: String = match &first.get(&ItemKey::Album).cloned() {
            Some(title) => title.to_string(),
            None => return Err(AlbumError::NoAlbum.into()),
        };

        if rest.is_empty() {
            return Ok(Self {
                title,
                tracks: tracks.to_vec(),
                artist: first.get(&ItemKey::AlbumArtist).cloned(),
                ..Default::default()
            });
        }

        if !rest
            .iter()
            .all(|song| song.get(&ItemKey::Album) == first.get(&ItemKey::Album))
        {
            return Err(AlbumError::MixedTracks.into());
        }

        Ok(Self {
            title,
            tracks: tracks.to_vec(),
            artist: first.get(&ItemKey::AlbumArtist).cloned(),
            ..Default::default()
        })
    }
}
