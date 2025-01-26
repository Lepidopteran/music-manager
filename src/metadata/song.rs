use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use lofty::probe::Probe;
use lofty::{prelude::*, tag::Tag};
use serde::{Deserialize, Serialize};

use super::{
    file::SongFileType,
    tags::{sanitize_tag, TagType},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongFile {
    path: PathBuf,
    tag_type: Option<TagType>,
    file_type: SongFileType,
    metadata: SongMetadata,
    created: SystemTime,
    last_modified: SystemTime,
    size: u64,
}

impl SongFile {
    pub fn open(path: &Path) -> Result<Self, String> {
        check_path(path)?;

        let path_metadata = path.metadata().map_err(|err| err.to_string())?;

        let tagged_file = match Probe::open(path) {
            Ok(tag) => tag.read().map_err(|err| err.to_string())?,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => return Err("No tag found".to_string()),
        };

        Ok(Self {
            path: path.to_path_buf(),
            size: path_metadata.len(),
            tag_type: Some(tag.tag_type().into()),
            file_type: SongFileType::from(tagged_file.file_type()),
            last_modified: path_metadata.modified().map_err(|err| err.to_string())?,
            created: path_metadata.created().map_err(|err| err.to_string())?,
            metadata: SongMetadata::from(tag),
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn metadata(&self) -> &SongMetadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut SongMetadata {
        &mut self.metadata
    }

    pub fn read(&mut self) -> Result<(), String> {
        self.metadata = SongMetadata::from_path(&self.path).map_err(|err| err.to_string())?;
        Ok(())
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn last_modified(&self) -> SystemTime {
        self.last_modified
    }

    pub fn created(&self) -> SystemTime {
        self.created
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub year: Option<String>,
}

impl SongMetadata {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        check_path(path)?;

        let tagged_file = match Probe::open(path) {
            Ok(tag) => tag.read().map_err(|err| err.to_string())?,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or("No tags found".to_string())?,
        };

        Ok(Self::from(tag))
    }
}

impl From<&Tag> for SongMetadata {
    fn from(tag: &Tag) -> Self {
        Self {
            title: tag.title().as_deref().map(sanitize_tag),
            artist: tag.artist().as_deref().map(sanitize_tag),
            album: tag.album().as_deref().map(sanitize_tag),
            album_artist: tag.get_string(&ItemKey::AlbumArtist).map(sanitize_tag),
            genre: tag
                .get_strings(&ItemKey::Genre)
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
            track_number: tag.track().map(|track| track.to_string()),
            disc_number: tag.disk().map(|disc| disc.to_string()),
            year: tag.year().map(|year| year.to_string()),
        }
    }
}

#[doc(hidden)]
pub fn check_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("Path does not exist".to_string());
    }

    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    Ok(())
}

impl From<SongFile> for SongMetadata {
    fn from(file: SongFile) -> Self {
        file.metadata
    }
}
