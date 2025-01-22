use std::{fmt::Debug, path::Path};

use id3::{frame::PictureType as Id3PictureType, Tag as Id3Tag};
use lofty::probe::Probe;
use lofty::tag::TagType;
use lofty::{picture::PictureType, prelude::*};
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

impl From<PictureType> for CoverArtType {
    fn from(value: PictureType) -> Self {
        match value {
            PictureType::CoverFront => CoverArtType::Front,
            PictureType::CoverBack => CoverArtType::Back,
            _ => CoverArtType::Other,
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
    pub data: Vec<u8>,
}

impl Debug for CoverArt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoverArt")
            .field("cover_type", &self.cover_type)
            .field("mime_type", &self.mime_type)
            .field("data_len", &self.data.len())
            .finish()
    }
}

/// Get all the cover art from a file
///
/// Returns an empty vector if no cover art is found
pub fn get_cover_art(path: &str) -> Vec<CoverArt> {
    let path = Path::new(path);

    if !path.exists() || !path.is_file() {
        return Vec::new();
    }

    let tagged_file = match Probe::open(path) {
        Ok(tag) => tag.read().unwrap(),
        Err(_) => return Vec::new(),
    };

    let tag = tagged_file.primary_tag().unwrap();

    if tag.is_empty() {
        return Vec::new();
    }

    let pictures = tag.pictures().iter();

    if pictures.len() == 0 {
        return Vec::new();
    }

    if tag.tag_type() == TagType::Mp4Ilst {
        pictures
            .enumerate()
            .map(|(index, picture)| CoverArt {
                // NOTE: The first picture should be the front cover in most cases
                cover_type: if index == 0 {
                    CoverArtType::Front
                } else {
                    CoverArtType::Other
                },
                mime_type: picture.mime_type().unwrap().as_str().to_string(),
                data: picture.data().to_vec(),
            })
            .collect()
    } else if pictures.len() > 1 {
        pictures
            .map(|picture| CoverArt {
                cover_type: picture.pic_type().into(),
                mime_type: picture.mime_type().unwrap().as_str().to_string(),
                data: picture.data().to_vec(),
            })
            .collect()
    } else {
        pictures
            .filter(|picture| {
                [PictureType::CoverFront, PictureType::Other].contains(&picture.pic_type())
            })
            .map(|picture| CoverArt {
                cover_type: PictureType::CoverFront.into(),
                mime_type: picture.mime_type().unwrap().as_str().to_string(),
                data: picture.data().to_vec(),
            })
            .collect()
    }
}
