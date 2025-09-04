use std::{
    collections::{BTreeMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
    time::SystemTime,
};

use lofty::{
    config::WriteOptions,
    id3::v2::Id3v2Tag,
    probe::Probe,
    tag::{ItemKey as LoftyKey, ItemValue, TagItem},
};
use lofty::{prelude::*, read_from, tag::Tag};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use ts_rs::TS;
use super::{
    file::SongFileType,
    item::{ItemKey, TagType},
    Result,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename = "SongMetadata")]
#[ts(export)]
pub struct Metadata {
    #[serde(flatten)]
    fields: BTreeMap<ItemKey, String>,
    #[serde(default = "BTreeMap::new")]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    unknown: BTreeMap<String, String>,
}

impl Metadata {
    pub fn new(items: BTreeMap<ItemKey, String>, unknown: BTreeMap<String, String>) -> Self {
        Self {
            fields: items,
            unknown,
        }
    }

    pub fn insert(&mut self, key: ItemKey, value: String) {
        self.fields.insert(key, value);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ItemKey, &String)> {
        self.fields.iter()
    }

    pub fn get(&self, key: &ItemKey) -> Option<&String> {
        self.fields.get(key)
    }

    pub fn get_unknown(&self, key: &String) -> Option<&String> {
        self.unknown.get(key)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SongError {
    #[error("No Tag(s) found")]
    NoTag,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SongFile {
    /// The path to the file
    path: PathBuf,
    /// The type of metadata format used
    tag_type: TagType,
    /// The type of file
    file_type: SongFileType,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339")]
    /// System time when the file was created
    created: OffsetDateTime,
    #[ts(type = "Date")]
    #[serde(with = "time::serde::rfc3339")]
    /// System time when the file was last modified
    last_modified: OffsetDateTime,
    /// The size of the file
    size: u64,
    /// Metadata contained in the file
    metadata: Option<Metadata>,
}

impl SongFile {
    /// Opens a song file and returns a [`SongFile`]
    pub fn open(path: &Path) -> Result<Self> {
        let path_metadata = path.metadata()?;
        let tagged_file = Probe::open(path)?.read()?;
        let tag = tagged_file.primary_tag().or(tagged_file.first_tag());

        let tag_type = match tag {
            Some(tag) => tag.tag_type().into(),
            None => tagged_file.file_type().primary_tag_type().into(),
        };

        Ok(Self {
            path: path.to_path_buf(),
            size: path_metadata.len(),
            file_type: SongFileType::from(tagged_file.file_type()),
            metadata: read_metadata_from_path(path).ok(),
            last_modified: OffsetDateTime::from(path_metadata.modified()?),
            created: OffsetDateTime::from(path_metadata.created()?),
            tag_type,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn metadata(&self) -> &Option<Metadata> {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Option<Metadata> {
        &mut self.metadata
    }

    pub fn remove_metadata(&mut self) {
        self.metadata = None;
    }

    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = Some(metadata);
    }

    pub fn reload_metadata(&mut self) -> Result<()> {
        self.set_metadata(read_metadata_from_path(&self.path)?);
        Ok(())
    }

    pub fn write(&mut self) -> Result<()> {
        let mut file = File::open(&self.path)?;
        let mut tagged_file = read_from(&mut file)?;
        let tag_type = self.tag_type.into();
        let tag = match tagged_file.primary_tag_mut() {
            Some(tag) => tag,
            None => &mut Tag::new(tag_type),
        };

        tag.clear();
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata.iter() {
                let key = key.clone().into();

                let split: Vec<_> = value.split(';').map(|s| s.trim()).collect();

                if split.len() == 1 {
                    let item =
                        TagItem::new_checked(tag_type, key, ItemValue::Text(value.to_string()));

                    if let Some(item) = item {
                        tag.insert(item);
                    }
                } else {
                    for value in split {
                        let item = TagItem::new_checked(
                            tag_type,
                            key.clone(),
                            ItemValue::Text(value.to_string()),
                        );

                        if let Some(item) = item {
                            tag.push(item);
                        }
                    }
                }
            }
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

    pub fn last_modified(&self) -> OffsetDateTime {
        self.last_modified
    }

    pub fn created(&self) -> OffsetDateTime {
        self.created
    }
}

pub fn read_metadata_from_path(path: &Path) -> Result<Metadata> {
    let tagged_file = Probe::open(path)?.read()?;

    let tag = match tagged_file.primary_tag() {
        Some(tag) => tag,
        None => tagged_file.first_tag().ok_or(SongError::NoTag)?,
    };

    let keys = tag
        .items()
        .map(|item| item.clone().into_key())
        .collect::<HashSet<lofty::tag::ItemKey>>();

    let items = keys
        .iter()
        .filter_map(|key| match key {
            LoftyKey::Unknown(_) => None,
            _ => {
                let value = (
                    key.clone().into(),
                    tag.get_strings(key)
                        .map(|string| string.trim().replace("\0", "").to_string())
                        .collect::<Vec<String>>()
                        .join("; "),
                );

                log::trace!("{key:?}: {value:?}");
                Some(value)
            }
        })
        .collect::<BTreeMap<ItemKey, String>>();

    let unknown = keys
        .iter()
        .filter_map(|key| match key {
            LoftyKey::Unknown(field_name) => {
                let value = (
                    field_name.to_string(),
                    tag.get_strings(key)
                        .map(|string| string.trim().replace("\0", "").to_string())
                        .collect::<Vec<String>>()
                        .join("; "),
                );

                log::trace!("{key:?}: {value:?}");
                Some(value)
            }
            _ => None,
        })
        .collect::<BTreeMap<String, String>>();

    Ok(Metadata::new(items, unknown))
}
