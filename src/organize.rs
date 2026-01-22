use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

use super::metadata;

use handlebars::Handlebars;

// TODO: Move these templates to a files
pub const ARTIST_TEMPLATE: &str = "{{artist}}/{{album}}/";
pub const ALBUM_ARTIST_TEMPLATE: &str = "{{albumArtist}}/{{album}}/";

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
    pub metadata: metadata::Metadata,
}

pub fn render_song_path(handlebar: &Handlebars, template: &str, song: &Song) -> Result<PathBuf> {
    Ok(PathBuf::from(handlebar.render_template(template, &song)?))
}
