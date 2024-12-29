use std::{fmt::Debug, path::Path};

use id3::{frame::PictureType as Id3PictureType, Tag as Id3Tag};
use metaflac::{block::PictureType as FlacPictureType, Tag as FlacTag};
use mp4ameta::{ImgFmt, Tag as Mp4Tag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[non_exhaustive]
pub enum CoverArtType {
    Front,
    Back,
    Other,
}

impl TryFrom<&str> for CoverArtType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "front" => Ok(CoverArtType::Front),
            "back" => Ok(CoverArtType::Back),
            "other" => Ok(CoverArtType::Other),
            _ => Err(format!("Invalid cover art type: {}", value))?,
        }
    }
}

impl From<Id3PictureType> for CoverArtType {
    fn from(value: Id3PictureType) -> Self {
        match value {
            Id3PictureType::CoverFront => CoverArtType::Front,
            Id3PictureType::CoverBack => CoverArtType::Back,
            _ => CoverArtType::Other,
        }
    }
}

impl From<FlacPictureType> for CoverArtType {
    fn from(value: FlacPictureType) -> Self {
        match value {
            FlacPictureType::CoverFront => CoverArtType::Front,
            FlacPictureType::CoverBack => CoverArtType::Back,
            _ => CoverArtType::Other,
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct CoverArt {
    pub cover_type: CoverArtType,
    pub mime_type: String,
    pub song_file_type: String,
    pub data: Vec<u8>,
}

impl Debug for CoverArt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoverArt")
            .field("cover_type", &self.cover_type)
            .field("mime_type", &self.mime_type)
            .field("song_file_type", &self.song_file_type)
            .field("data_len", &self.data.len())
            .finish()
    }
}

// TODO: Add opus cover art
/// Get all the cover art from a file
///
/// Returns an empty vector if no cover art is found
pub fn get_cover_art(path: &str) -> Vec<CoverArt> {
    let path = Path::new(path);

    if !path.exists() || !path.is_file() {
        return Vec::new();
    }

    let file_type = match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) => ext,
        None => return Vec::new(),
    };

    match file_type {
        "mp3" | "wav" | "aif" | "aiff" => {
            let tags = match Id3Tag::read_from_path(path) {
                Ok(tag) => tag,
                Err(_) => return Vec::new(),
            };

            tags.pictures()
                .filter(|picture| picture.mime_type.starts_with("image/"))
                .map(|picture| CoverArt {
                    cover_type: picture.picture_type.into(),
                    mime_type: picture.mime_type.clone(),
                    song_file_type: file_type.to_string(),
                    data: picture.data.clone(),
                })
                .collect()
        }
        "flac" => {
            let tags = match FlacTag::read_from_path(path) {
                Ok(tag) => tag,
                Err(_) => return Vec::new(),
            };

            tags.pictures()
                .filter(|picture| picture.mime_type.starts_with("image/"))
                .map(|picture| CoverArt {
                    cover_type: picture.picture_type.into(),
                    mime_type: picture.mime_type.to_string(),
                    song_file_type: file_type.to_string(),
                    data: picture.data.to_vec(),
                })
                .collect()
        }
        "mp4" | "m4a" => {
            let tags = match Mp4Tag::read_from_path(path) {
                Ok(tag) => tag,
                Err(_) => return Vec::new(),
            };

            tags.artworks()
                .enumerate()
                .map(|(index, artwork)| CoverArt {

                    // NOTE: I'm assuming the cover art is always the first artwork in the list.
                    cover_type: match index {
                        0 => CoverArtType::Front,
                        1 => CoverArtType::Back,
                        _ => CoverArtType::Other,
                    },
                    mime_type: match artwork.fmt {
                        ImgFmt::Jpeg => "image/jpeg".to_string(),
                        ImgFmt::Png => "image/png".to_string(),
                        ImgFmt::Bmp => "image/bmp".to_string(),
                    },
                    song_file_type: file_type.to_string(),
                    data: artwork.data.to_vec(),
                })
                .collect()
        }
        _ => Vec::new(),
    }
}
