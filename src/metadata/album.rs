use super::SongMetadata;

#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub title: String,
    pub tracks: Vec<SongMetadata>,
    pub barcode: Option<String>,
    pub catalog_number: Option<String>,
    pub comment: Option<String>,
    pub country: Option<String>,
    pub artist: Option<String>,
    pub label: Option<String>,
    pub date: Option<time::Date>,
    pub original_date: Option<time::Date>,
}

impl TryFrom<Vec<SongMetadata>> for Album {
    type Error = String;

    fn try_from(tracks: Vec<SongMetadata>) -> Result<Self, Self::Error> {
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
