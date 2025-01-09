use std::{fs::File, io::BufReader, path::Path};

use id3::{Tag as Id3Tag, TagLike};
use lewton::inside_ogg::{read_headers, OggStreamReader};
use metaflac::Tag as FlacTag;
use mp4ameta::Tag as Mp4Tag;
use serde::{Deserialize, Serialize};
use sqlx::prelude::*;

use super::tags::{parse_tags, sanitize_tag, ParserConfig};

#[derive(Deserialize, Serialize, FromRow, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: i64,
    pub path: String,
    pub parent_path: String,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<String>,
    pub disc_number: Option<String>,
    pub year: Option<String>,
}

impl Song {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err("Path does not exist".to_string());
        }

        if !path.is_file() {
            return Err("Path is not a file".to_string());
        }

        let mut file = File::open(path).map_err(|err| err.to_string())?;

        let stem = match path.file_stem() {
            Some(stem) => stem.to_str().map(|stem| stem.to_string()),
            None => return Err("No file stem found".to_string()),
        };

        let file_type = match path.extension() {
            Some(ext) => ext.to_string_lossy().to_string().to_lowercase(),
            None => return Err("No file type found".to_string()),
        };

        // TODO: Remove default parser config for customization
        let config = ParserConfig::new();

        match file_type.as_str() {
            "mp3" | "wav" | "aif" | "aiff" => {
                let tag = Id3Tag::read_from2(file).map_err(|err| err.to_string())?;

                Ok(Self {
                    title: match tag.title() {
                        Some(title) => Some(sanitize_tag(title)),
                        None => stem,
                    },
                    artist: tag
                        .artist()
                        .map(|artist| parse_tags(artist, &config).join(", ")),
                    album: tag.album().map(sanitize_tag),
                    album_artist: tag
                        .album_artist()
                        .map(|album_artist| parse_tags(album_artist, &config).join(", ")),
                    genre: tag
                        .genre()
                        .map(|genre| parse_tags(genre, &config).join(", ")),
                    track_number: tag.track().map(|track| track.to_string()),
                    disc_number: tag.disc().map(|disc| disc.to_string()),
                    year: tag.year().map(|year| year.to_string()),
                    ..Default::default()
                })
            }
            "flac" => {
                let mut reader = BufReader::new(file);
                let tag = FlacTag::read_from(&mut reader).map_err(|err| err.to_string())?;

                let comments = match tag.vorbis_comments() {
                    Some(comments) => comments,
                    None => return Err("No comments found".to_string()),
                };

                Ok(Self {
                    title: match comments.title() {
                        Some(title) => Some(title[0].to_string()),
                        None => stem,
                    },
                    artist: comments.artist().map(|artist| artist[0].to_string()),
                    album: comments.album().map(|album| album[0].to_string()),
                    album_artist: comments
                        .album_artist()
                        .map(|album_artist| parse_tags(&album_artist[0], &config).join(", ")),
                    genre: comments
                        .genre()
                        .map(|genre| parse_tags(&genre[0], &config).join(", ")),
                    track_number: comments.track().map(|track| track.to_string()),
                    disc_number: comments
                        .get("DISCNUMBER")
                        .map(|disc| sanitize_tag(&disc[0])),
                    ..Default::default()
                })
            }
            "mp4" | "m4a" => {
                let tag = Mp4Tag::read_from(&mut file).map_err(|err| err.to_string())?;

                Ok(Self {
                    title: stem,
                    artist: tag
                        .artist()
                        .map(|artist| parse_tags(artist, &config).join(", ")),
                    album: tag.album().map(sanitize_tag),
                    album_artist: tag
                        .album_artist()
                        .map(|album_artist| parse_tags(album_artist, &config).join(", ")),
                    genre: tag
                        .genre()
                        .map(|genre| parse_tags(genre, &config).join(", ")),
                    track_number: tag.track_number().map(|track| track.to_string()),
                    disc_number: tag.disc_number().map(|disc| disc.to_string()),
                    ..Default::default()
                })
            }
            _ => Err("Unsupported file type".to_string()),
        }
    }
}
