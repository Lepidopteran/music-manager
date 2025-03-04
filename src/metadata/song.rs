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
};

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
            tag_type: tag.tag_type().into(),
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

    pub fn write(&mut self) -> Result<(), String> {
        let mut file = File::open(&self.path).map_err(|err| err.to_string())?;

        let mut tagged_file = match read_from(&mut file) {
            Ok(tag) => tag,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag_mut() {
            Some(tag) => tag,
            None => return Err("No tag found".to_string()),
        };

        // TEST: Debug info

        let mut debug_message = Vec::new();

        debug_message.push(format!("#### {:?}", self.tag_type));
        debug_message.push(String::new());
        debug_message.push("##### Original".to_string());
        debug_message.push(String::new());
        debug_message.push("```".to_string());

        for key in &[
            ItemKey::TrackTitle,
            ItemKey::AlbumArtist,
            ItemKey::TrackArtist,
            ItemKey::TrackArtists,
            ItemKey::Genre,
            ItemKey::MusicBrainzReleaseId,
            ItemKey::MusicBrainzReleaseArtistId,
            ItemKey::MusicBrainzReleaseGroupId,
            ItemKey::MusicBrainzRecordingId,
            ItemKey::MusicBrainzTrackId,
            ItemKey::MusicBrainzArtistId,
        ] {
            let value: String = tag
                .get_strings(key)
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
                .join("; ");

            if value.is_empty() {
                debug_message.push(format!("{:<30} None", format!("{key:?}:")));
            } else {
                debug_message.push(format!("{:<30} {value:<10}", format!("{key:?}:")));
            }
        }

        debug_message.push("```".to_string());
        debug_message.push(String::new());

        for (key, value) in &[
            (ItemKey::TrackTitle, self.metadata.title.clone()),
            (ItemKey::TrackArtist, self.metadata.artist.clone()),
            (ItemKey::AlbumTitle, self.metadata.album.clone()),
            (ItemKey::AlbumArtist, self.metadata.album_artist.clone()),
            (ItemKey::TrackNumber, self.metadata.track_number.clone()),
            (ItemKey::DiscNumber, self.metadata.disc_number.clone()),
            (ItemKey::Year, self.metadata.year.clone()),
        ] {
            if let Some(value) = value {
                let item = TagItem::new_checked(
                    self.tag_type.into(),
                    key.clone(),
                    ItemValue::Text(value.to_string()),
                );

                if let Some(item) = item {
                    tag.insert(item);
                }
            }
        }

        if let Some(genres) = &self.metadata.genre {
            tag.remove_key(&ItemKey::Genre);
            for genre in genres.split(",") {
                tag.push(TagItem::new(
                    ItemKey::Genre,
                    ItemValue::Text(genre.trim().to_string()),
                ));
            }
        }

        match self.tag_type {
            TagType::Id3v2 => {
                let id3_tag: Id3v2Tag = tag.clone().into();

                id3_tag
                    .save_to_path(&self.path, WriteOptions::default())
                    .map_err(|err| err.to_string())?
            }
            _ => {
                tag.save_to_path(&self.path, WriteOptions::default())
                    .map_err(|err| err.to_string())?;
            }
        }

        // TEST: Debug info after write
        let mut file = File::open(&self.path).map_err(|err| err.to_string())?;

        let mut tagged_file = match read_from(&mut file) {
            Ok(tag) => tag,
            Err(err) => return Err(err.to_string()),
        };

        let tag = match tagged_file.primary_tag_mut() {
            Some(tag) => tag,
            None => return Err("No tag found".to_string()),
        };

        debug_message.push("##### Written using lofty".to_string());
        debug_message.push(String::new());
        debug_message.push("```".to_string());

        for key in &[
            ItemKey::TrackTitle,
            ItemKey::AlbumArtist,
            ItemKey::TrackArtist,
            ItemKey::TrackArtists,
            ItemKey::Genre,
            ItemKey::MusicBrainzReleaseId,
            ItemKey::MusicBrainzReleaseArtistId,
            ItemKey::MusicBrainzReleaseGroupId,
            ItemKey::MusicBrainzRecordingId,
            ItemKey::MusicBrainzTrackId,
            ItemKey::MusicBrainzArtistId,
        ] {
            let value: String = tag
                .get_strings(key)
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
                .join("; ");

            if value.is_empty() {
                debug_message.push(format!("{:<30} None", format!("{key:?}:")));
            } else {
                debug_message.push(format!("{:<30} {value:<10}", format!("{key:?}:")));
            }
        }

        debug_message.push("```".to_string());
        debug_message.push(String::new());

        use std::io::Write;

        let mut log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("song-metadata-changelog.md")
            .unwrap();

        log_file
            .write_all(debug_message.join("\n").as_bytes())
            .unwrap();

        tracing::debug!("{debug_message:#?}");

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
