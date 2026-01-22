use std::{collections::BTreeMap, path::PathBuf};

use handlebars::Handlebars;
use serde::Serialize;
use ts_rs::TS;

use super::metadata;
use metadata::Metadata;

pub const ALBUM_ARTIST_TEMPLATE: &str = "{{albumArtist}}/{{album}}/{{title}}.{{fileType}}";

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum OrganizeError {
    #[error("Handlebars render error: {0}")]
    Handlebars(#[from] handlebars::RenderError),
}

pub type Result<T, E = OrganizeError> = std::result::Result<T, E>;

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "OrganizableSong")]
pub struct Song {
    pub file_name: String,
    pub file_type: String,
    #[serde(flatten)]
    pub metadata: Metadata,
}

pub fn render_song_path(handlebar: &Handlebars, template: &str, song: Song) -> Result<PathBuf> {
    let rendered = handlebar.render_template(
        template,
        &Song {
            metadata: sanitize_metadata(&song.metadata),
            ..song
        },
    )?;
    Ok(PathBuf::from(&rendered))
}

pub fn sanitize_metadata(metadata: &Metadata) -> Metadata {
    Metadata::new(
        metadata
            .fields()
            .iter()
            .fold(BTreeMap::new(), |mut sanitized_known, (key, value)| {
                sanitized_known.insert(key.clone(), sanitize_filename::sanitize(value));
                sanitized_known
            }),
        metadata.unknown_fields().iter().fold(
            BTreeMap::new(),
            |mut sanitized_unknown, (key, value)| {
                sanitized_unknown.insert(key.clone(), sanitize_filename::sanitize(value));
                sanitized_unknown
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use handlebars::Handlebars;
    use std::collections::BTreeMap;

    use metadata::item::ItemKey;

    #[test]
    fn test_render_song_path() {
        let handlebars = Handlebars::new();

        let metadata = Metadata::new(
            BTreeMap::from([
                (ItemKey::Title, "test_title".to_string()),
                (ItemKey::Artist, "test_artist".to_string()),
                (ItemKey::Album, "test_album".to_string()),
                (ItemKey::AlbumArtist, "test_album_artist".to_string()),
                (ItemKey::Genre, "test_genre".to_string()),
                (ItemKey::TrackNumber, "test_track_number".to_string()),
                (ItemKey::DiscNumber, "test_disc_number".to_string()),
            ]),
            BTreeMap::new(),
        );

        let song = Song {
            file_name: "test_file".to_string(),
            file_type: "mp3".to_string(),
            metadata,
        };

        let result = render_song_path(&handlebars, "{{fileName}}.{{fileType}}", song);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from("test_file.mp3"));
    }

    #[test]
    fn test_render_song_path_with_dirty_metadata() {
        let handlebars = Handlebars::new();

        let metadata = Metadata::new(
            BTreeMap::from([
                (ItemKey::Title, "title/with:illegal*chars".to_string()),
                (ItemKey::Artist, "artist\\name\"".to_string()),
                (ItemKey::Album, "album<name>".to_string()),
                (ItemKey::AlbumArtist, "album:artist|name".to_string()),
                (ItemKey::Genre, "genre*illegal?chars".to_string()),
                (ItemKey::TrackNumber, "track:#1".to_string()),
                (ItemKey::DiscNumber, "disc:disk?No.2".to_string()),
            ]),
            BTreeMap::new(),
        );

        let song = Song {
            file_name: "file".to_string(),
            file_type: "wav".to_string(),
            metadata,
        };

        let result = render_song_path(
            &handlebars,
            "{{albumArtist}}/{{album}}/{{title}}.{{fileType}}",
            song,
        );
        assert!(result.is_ok());

        let rendered_path = result.unwrap();
        assert_eq!(
            rendered_path,
            PathBuf::from("albumartistname/albumname/titlewithillegalchars.wav")
        );
    }
}
