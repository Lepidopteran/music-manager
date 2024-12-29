use super::{get_cover_art, CoverArt, Song};
use std::{fs::read_dir, path::Path};

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Album {
    pub title: String,
    pub tracks: Vec<Song>,
    pub barcode: Option<String>,
    pub catalog_number: Option<String>,
    pub comment: Option<String>,
    pub country: Option<String>,
    pub artist: Option<String>,
    pub label: Option<String>,
    pub date: Option<time::Date>,
}

impl Album {
    // pub fn cover_art(&self) -> Option<CoverArt> {
    //     let path = self
    //         .tracks
    //         .first()
    //         .map(|song| &song.path.clone())
    //         .and_then(|path| Path::new(path).parent().map(|path| path.to_path_buf()));
    //
    //     let path = path?;
    //
    //     let mut cover_art: Option<CoverArt> = None;
    //
    //     for entry in read_dir(&path).ok()? {
    //         let entry = entry.ok()?;
    //
    //         if !entry.file_type().ok()?.is_file() {
    //             continue;
    //         }
    //
    //         let stem = entry
    //             .path()
    //             .file_stem()?
    //             .to_string_lossy()
    //             .to_string()
    //             .to_lowercase();
    //
    //         let file_type = entry
    //             .path()
    //             .extension()?
    //             .to_string_lossy()
    //             .to_string()
    //             .to_lowercase();
    //
    //         if !["jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp"]
    //             .iter()
    //             .any(|name| file_type.ends_with(name))
    //         {
    //             continue;
    //         }
    //
    //         if !["cover", "folder", "albumart"]
    //             .iter()
    //             .any(|name| stem.starts_with(name))
    //         {
    //             continue;
    //         }
    //
    //
    //
    //     }
    // }
}

impl TryFrom<Vec<Song>> for Album {
    type Error = String;

    fn try_from(tracks: Vec<Song>) -> Result<Self, Self::Error> {
        let (first, rest) = tracks.split_first().ok_or("No tracks found")?;
        let title = match &first.album {
            Some(title) => title.clone(),
            None => return Err("No album found".to_string()),
        };

        if rest.is_empty() {
            return Ok(Self {
                title,
                tracks: tracks.to_vec(),
                artist: first.album_artist.clone(),
                ..Default::default()
            });
        }

        if !rest.iter().all(|song| song.album == first.album) {
            return Err("All tracks must be from the same album".to_string());
        }

        Ok(Self {
            title,
            tracks: tracks.to_vec(),
            artist: first.album_artist.clone(),
            ..Default::default()
        })
    }
}
