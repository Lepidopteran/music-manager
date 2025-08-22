use std::{
    fs::File,
    path::{Path, PathBuf},
    time::SystemTime,
};

use lofty::{
    config::WriteOptions,
    id3::v2::Id3v2Tag,
    probe::Probe,
    tag::{ItemValue, TagItem},
};
use lofty::{prelude::*, read_from, tag::Tag};
use serde::{Deserialize, Serialize};

use super::{
    file::SongFileType,
    tags::{sanitize_tag, TagType},
    Result,
};

#[derive(Debug, thiserror::Error)]
pub enum SongError {
    #[error("No Tag(s) found")]
    NoTag,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SongFile {
    /// The path to the file
    path: PathBuf,
    /// The type of metadata format used
    tag_type: TagType,
    /// The type of file
    file_type: SongFileType,
    /// Metadata contained in the file
    metadata: SongMetadata,
    /// System time when the file was created
    created: SystemTime,
    /// System time when the file was last modified
    last_modified: SystemTime,
    /// The size of the file
    size: u64,
}

impl SongFile {
    /// Opens a song file and returns a [`SongFile`]
    pub fn open(path: &Path) -> Result<Self> {
        let path_metadata = path.metadata()?;

        let tagged_file = Probe::open(path)?.read()?;

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or(SongError::NoTag)?,
        };

        Ok(Self {
            path: path.to_path_buf(),
            size: path_metadata.len(),
            tag_type: tag.tag_type().into(),
            file_type: SongFileType::from(tagged_file.file_type()),
            last_modified: path_metadata.modified()?,
            created: path_metadata.created()?,
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

    pub fn read(&mut self) -> Result<()> {
        self.metadata = SongMetadata::from_path(&self.path)?;
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        let mut file = File::open(&self.path)?;

        let mut tagged_file = read_from(&mut file)?;

        let tag = match tagged_file.primary_tag_mut() {
            Some(tag) => tag,
            None => return Err(SongError::NoTag.into()),
        };

        for (key, value) in &[
            (ItemKey::TrackTitle, self.metadata.title.clone()),
            (ItemKey::TrackArtist, self.metadata.artist.clone()),
            (ItemKey::AlbumTitle, self.metadata.album.clone()),
            (ItemKey::AlbumArtist, self.metadata.album_artist.clone()),
            (ItemKey::TrackNumber, self.metadata.track_number.clone()),
            (ItemKey::DiscNumber, self.metadata.disc_number.clone()),
            (ItemKey::Year, self.metadata.year.clone()),
        ] {
            let item = TagItem::new_checked(
                self.tag_type.into(),
                key.clone(),
                ItemValue::Text(value.clone().unwrap_or_default()),
            );

            if let Some(item) = item {
                tag.insert(item);
            }
        }

        tag.remove_key(&ItemKey::Genre);
        for genre in self.metadata.genre.clone().unwrap_or_default().split(",") {
            tag.push(TagItem::new(
                ItemKey::Genre,
                ItemValue::Text(genre.trim().to_string()),
            ));
        }

        match self.tag_type {
            TagType::Id3v2 => {
                let id3_tag: Id3v2Tag = tag.clone().into();

                id3_tag.save_to_path(&self.path, WriteOptions::default())?
            }
            _ => {
                tag.save_to_path(&self.path, WriteOptions::default())?;
            }
        }

        Ok(())
    }

    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }

    pub fn tag_type_mut(&mut self) -> &mut TagType {
        &mut self.tag_type
    }

    pub fn file_type(&self) -> SongFileType {
        self.file_type
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

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SongMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub mood: Option<String>,
    pub year: Option<String>,
}

impl SongMetadata {
    pub fn from_path(path: &Path) -> Result<Self> {
        let tagged_file = Probe::open(path)?.read()?;

        let tag = match tagged_file.primary_tag() {
            Some(tag) => tag,
            None => tagged_file.first_tag().ok_or(SongError::NoTag)?,
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
            mood: tag.get_string(&ItemKey::Mood).map(sanitize_tag),
            genre: Some(
                tag.get_strings(&ItemKey::Genre)
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
            track_number: tag.track().map(|track| track.to_string()),
            disc_number: tag.disk().map(|disc| disc.to_string()),
            year: tag.year().map(|year| year.to_string()),
        }
    }
}

impl From<SongFile> for SongMetadata {
    fn from(file: SongFile) -> Self {
        file.metadata
    }
}
