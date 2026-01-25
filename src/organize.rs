use std::{
    collections::BTreeMap,
    path::{MAIN_SEPARATOR_STR, PathBuf},
};

use handlebars::Handlebars;
use serde::Serialize;

use super::metadata;
use metadata::Metadata;

pub const DEFAULT_TEMPLATE: &str = r#"
{{#if albumArtist}}
  {{albumArtist}}/
{{else}}
  {{artist}}/
{{/if}}

{{#if album}}
  {{album}}/
{{/if}}

{{title}} - {{trackNumber}}
"#;

#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum OrganizeError {
    #[error(transparent)]
    Handlebars(#[from] handlebars::RenderError),
    #[error("Original path has no file name: {0}")]
    NoFileName(PathBuf),
}

pub type Result<T, E = OrganizeError> = std::result::Result<T, E>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    #[serde(skip)]
    pub file_path: PathBuf,
    #[serde(flatten)]
    pub metadata: Metadata,
}

pub fn render_song_path(
    handlebar: &Handlebars,
    template: &str,
    song: &Song,
    rename_original_file: bool,
) -> Result<PathBuf> {
    let mut rendered_path = handlebar
        .render_template(template, &sanitize_metadata(&song.metadata))?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
        .replace(['\\', '/'], MAIN_SEPARATOR_STR);

    rendered_path.push_str(
        &song
            .file_path
            .extension()
            .map(|ext| {
                format!(
                    ".{}",
                    ext.to_str().expect("File extension contains invalid UTF-8")
                )
            })
            .unwrap_or_default(),
    );

    let path = PathBuf::from(rendered_path);
    if !rename_original_file {
        let original_file_name = song
            .file_path
            .file_name()
            .ok_or(OrganizeError::NoFileName(song.file_path.clone()))?;

        Ok(path.with_file_name(original_file_name))
    } else {
        Ok(path)
    }
}

pub fn sanitize_metadata(metadata: &Metadata) -> Metadata {
    Metadata::new(
        metadata
            .fields()
            .iter()
            .fold(BTreeMap::new(), |mut sanitized_known, (key, value)| {
                sanitized_known.insert(key.clone(), sanitize_filename::sanitize(value).to_string());
                sanitized_known
            }),
        metadata.unknown_fields().iter().fold(
            BTreeMap::new(),
            |mut sanitized_unknown, (key, value)| {
                sanitized_unknown
                    .insert(key.clone(), sanitize_filename::sanitize(value).to_string());
                sanitized_unknown
            },
        ),
    )
}

#[cfg(test)]
mod tests {
    use test_log::test;

    use super::*;
    use handlebars::Handlebars;
    use std::collections::BTreeMap;

    use metadata::item::ItemKey;

    #[test]
    fn test_render_song_with_simple_template() {
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
            file_path: PathBuf::from("test_file.mp3"),
            metadata,
        };

        let result = render_song_path(&handlebars, DEFAULT_TEMPLATE, &song, true);

        log::debug!("Result: {:#?}", result);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from("test_album_artist/test_album/test_title - test_track_number.mp3")
        );
    }

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
            file_path: PathBuf::from("test_file.mp3"),
            metadata,
        };

        let result = render_song_path(&handlebars, DEFAULT_TEMPLATE, &song, false);

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            PathBuf::from("test_album_artist/test_album/test_file.mp3")
        );
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
            file_path: PathBuf::from("test_file.wav"),
            metadata,
        };

        let result = render_song_path(
            &handlebars,
            "{{albumArtist}}/{{album}}/{{title}}",
            &song,
            true,
        );
        assert!(result.is_ok());

        let rendered_path = result.unwrap();
        assert_eq!(
            rendered_path,
            PathBuf::from("albumartistname/albumname/titlewithillegalchars.wav")
        );
    }
}
