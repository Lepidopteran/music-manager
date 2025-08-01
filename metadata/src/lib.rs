mod album;
mod cover_art;
mod song;
mod file;

pub mod tags;
pub use {album::*, cover_art::*, song::*};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Album error: {0}")]
    Album(#[from] AlbumError),
    #[error("Cover art error: {0}")]
    Song(#[from] SongError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Lofty error: {0}")]
    Lofty(#[from] lofty::error::LoftyError),
}

type Result<T, E = Error> = std::result::Result<T, E>;
